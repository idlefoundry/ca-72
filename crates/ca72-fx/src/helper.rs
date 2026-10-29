//! One helper process and its control channel. A reader thread routes replies to the
//! requests waiting for them; end of file on the helper's stdout means the process died,
//! which is reported at once through the `on_exit` callback.
//!
//! DAW-v3's `daw-sandbox/src/helper.rs` (commit cb15558, third_party/daw-v3/VENDORED.md),
//! with its notices reduced to what the standalone keeps: the instances whose plug-in
//! changed its own parameters or state (its editor), whose state is then saved again; the
//! values its editor gave its parameters (a pinned parameter's base, decisions.md
//! R-FX-PARAMS); the instances whose parameters' names or list changed; and the rest as
//! they come (a window the player closed). The helper's log goes to the
//! standalone's stderr with `CA72_FX_LOG` set.

use daw_ipc::protocol::{self, Op, Reply, Request};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Called on the reader thread when the helper's stdout closes.
pub type OnExit = Box<dyn FnOnce(&Arc<HelperProc>) + Send>;

/// Why a request had no answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The helper answered no: the plug-in's or the helper's error.
    Refused(String),
    /// The helper exited (crashed, was killed) before it answered.
    Exited,
    /// It did not answer in time: it is hung (decisions.md R-FX-SAFETY).
    TimedOut(Duration),
    /// It was not running when asked.
    NotRunning,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Refused(e) => f.write_str(e),
            Failure::Exited => f.write_str("the plugin helper exited"),
            Failure::TimedOut(t) => write!(f, "the plugin helper did not answer within {t:?}"),
            Failure::NotRunning => f.write_str("the plugin helper is not running"),
        }
    }
}

/// How many lines of the helper's stderr are kept for crash reports.
const KEEP_LINES: usize = 20;

/// How many parameters' editor values are kept until taken.
const KEEP_EDITS: usize = 1024;

pub struct HelperProc {
    child: Mutex<Child>,
    stdin: Mutex<Option<ChildStdin>>,
    pending: Mutex<HashMap<u64, Sender<Reply>>>,
    /// Notices the helper sent on its own (a plugin window the user closed), until taken.
    notices: Mutex<Vec<serde_json::Value>>,
    /// Instances whose plug-in changed its parameters or its state itself (its editor's
    /// edits, a preset it loaded: DAW-v3 D222, D228, D234), by instance number, until taken.
    edited: Mutex<Vec<u32>>,
    /// The values the plug-ins' editors gave their parameters (instance, parameter, value in
    /// the plug-in's units), the latest of each, until taken (R-FX-PARAMS).
    edits: Mutex<Vec<(u32, u32, f64)>>,
    /// Instances whose parameters' names or list changed, until taken (R-FX-PARAMS).
    relist: Mutex<Vec<u32>>,
    next_id: AtomicU64,
    alive: AtomicBool,
    pid: u32,
    /// The last lines the helper wrote to stderr.
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
}

impl std::fmt::Debug for HelperProc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HelperProc")
            .field("pid", &self.pid)
            .finish()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl HelperProc {
    /// Starts a helper. `on_exit` runs on the reader thread when the helper's stdout
    /// closes (it exited, crashed or was killed).
    pub fn spawn(exe: &Path, on_exit: OnExit) -> Result<Arc<HelperProc>, String> {
        let mut command = Command::new(exe);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        crate::locate::no_console(&mut command);
        let mut child = command
            .spawn()
            .map_err(|e| format!("cannot start the plugin helper {}: {e}", exe.display()))?;
        let stdout = child.stdout.take().ok_or("no helper stdout")?;
        let stderr = child.stderr.take().ok_or("no helper stderr")?;
        let stdin = child.stdin.take();
        let pid = child.id();
        let tail = Arc::new(Mutex::new(VecDeque::new()));
        let h = Arc::new(HelperProc {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            pending: Mutex::new(HashMap::new()),
            notices: Mutex::new(Vec::new()),
            edited: Mutex::new(Vec::new()),
            edits: Mutex::new(Vec::new()),
            relist: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
            alive: AtomicBool::new(true),
            pid,
            stderr_tail: tail.clone(),
        });
        std::thread::Builder::new()
            .name(format!("ca72-fx-{pid}-stderr"))
            .spawn(move || {
                for line in BufReader::new(stderr).lines() {
                    let Ok(line) = line else { break };
                    if std::env::var_os("CA72_FX_LOG").is_some() {
                        eprintln!("[ca72-fx-helper {pid}] {line}");
                    }
                    let mut t = lock(&tail);
                    if t.len() == KEEP_LINES {
                        t.pop_front();
                    }
                    t.push_back(line);
                }
            })
            .map_err(|e| e.to_string())?;
        let reader = h.clone();
        std::thread::Builder::new()
            .name(format!("ca72-fx-{pid}-reader"))
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    match protocol::decode::<Reply>(&line) {
                        Some(Ok(reply))
                            if reply.id == 0
                                && (reply.result["notice"] == protocol::NOTICE_PARAM_EDITS
                                    || reply.result["notice"] == protocol::NOTICE_PARAM_VALUES) =>
                        {
                            if let Some(n) = reply.result["instance"].as_u64() {
                                lock(&reader.edited).push(n as u32);
                                reader.keep_edits(n as u32, &reply.result["edits"]);
                            }
                        }
                        Some(Ok(reply))
                            if reply.id == 0
                                && reply.result["notice"] == protocol::NOTICE_PARAMS_CHANGED =>
                        {
                            if let Some(n) = reply.result["instance"].as_u64() {
                                lock(&reader.relist).push(n as u32);
                            }
                        }
                        Some(Ok(reply)) if reply.id == 0 => {
                            lock(&reader.notices).push(reply.result);
                        }
                        Some(Ok(reply)) => {
                            if let Some(tx) = lock(&reader.pending).remove(&reply.id) {
                                let _ = tx.send(reply);
                            }
                        }
                        Some(Err(_)) | None => {
                            // Not protocol: something the plugin printed.
                            let mut t = lock(&reader.stderr_tail);
                            if t.len() == KEEP_LINES {
                                t.pop_front();
                            }
                            t.push_back(line);
                        }
                    }
                }
                reader.alive.store(false, Ordering::Release);
                // Waiting requests see a disconnected channel.
                lock(&reader.pending).clear();
                on_exit(&reader);
            })
            .map_err(|e| e.to_string())?;
        let hello = h.request(
            Op::Hello {
                version: protocol::VERSION,
            },
            Duration::from_secs(10),
        );
        if let Err(e) = hello {
            h.kill();
            return Err(format!("the plugin helper did not start: {e}"));
        }
        Ok(h)
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    /// Sends a request and waits for its reply.
    pub fn request(&self, op: Op, timeout: Duration) -> Result<serde_json::Value, String> {
        self.ask(op, timeout).map_err(|e| e.to_string())
    }

    /// Sends a request and waits for its reply; why there was none.
    pub fn ask(&self, op: Op, timeout: Duration) -> Result<serde_json::Value, Failure> {
        if !self.is_alive() {
            return Err(Failure::NotRunning);
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = channel();
        lock(&self.pending).insert(id, tx);
        let line = protocol::encode(&Request { id, op });
        {
            let mut stdin = lock(&self.stdin);
            let Some(w) = stdin.as_mut() else {
                lock(&self.pending).remove(&id);
                return Err(Failure::NotRunning);
            };
            if w.write_all(line.as_bytes())
                .and_then(|()| w.flush())
                .is_err()
            {
                lock(&self.pending).remove(&id);
                return Err(Failure::Exited);
            }
        }
        match rx.recv_timeout(timeout) {
            Ok(r) if r.ok => Ok(r.result),
            Ok(r) => Err(Failure::Refused(
                r.error
                    .unwrap_or_else(|| "the plugin helper refused".into()),
            )),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                lock(&self.pending).remove(&id);
                if self.is_alive() {
                    Err(Failure::TimedOut(timeout))
                } else {
                    Err(Failure::Exited)
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(Failure::Exited),
        }
    }

    /// Notices the helper sent on its own since the last call.
    pub fn take_notices(&self) -> Vec<serde_json::Value> {
        std::mem::take(&mut *lock(&self.notices))
    }

    /// The instances whose plug-in changed its own parameters or state since the last call.
    pub fn take_edited(&self) -> Vec<u32> {
        std::mem::take(&mut *lock(&self.edited))
    }

    /// The values the plug-ins' editors gave their parameters since the last call
    /// (instance, parameter, value in the plug-in's units), the latest of each, oldest first.
    pub fn take_param_edits(&self) -> Vec<(u32, u32, f64)> {
        std::mem::take(&mut *lock(&self.edits))
    }

    /// The instances whose parameters' names or list changed since the last call.
    pub fn take_relist(&self) -> Vec<u32> {
        std::mem::take(&mut *lock(&self.relist))
    }

    /// Keeps the values of a `param_edits` notice's edits (DAW-v3 D222: a gesture's ends
    /// carry the value it had then), the latest of each; at most [`KEEP_EDITS`].
    fn keep_edits(&self, instance: u32, edits: &serde_json::Value) {
        let Ok(edits) = serde_json::from_value::<Vec<protocol::ParamEdit>>(edits.clone()) else {
            return;
        };
        let mut kept = lock(&self.edits);
        for e in edits {
            let Some(v) = e.value.filter(|v| v.is_finite()) else {
                continue;
            };
            kept.retain(|k| k.0 != instance || k.1 != e.param);
            if kept.len() == KEEP_EDITS {
                kept.remove(0);
            }
            kept.push((instance, e.param, v));
        }
    }

    /// Kills the process (SIGKILL on Unix, TerminateProcess on Windows).
    pub fn kill(&self) {
        let _ = lock(&self.child).kill();
    }

    /// Waits for the process to exit and describes how it ended.
    pub fn reap(&self) -> String {
        match lock(&self.child).wait() {
            Ok(s) => describe(s),
            Err(e) => format!("unknown exit ({e})"),
        }
    }

    /// Asks the helper to quit, then kills it if it does not exit in time.
    pub fn shutdown(&self, grace: Duration) {
        let _ = self.request(Op::Quit, grace);
        drop(lock(&self.stdin).take());
        let deadline = std::time::Instant::now() + grace;
        loop {
            if let Ok(Some(_)) = lock(&self.child).try_wait() {
                return;
            }
            if std::time::Instant::now() >= deadline {
                self.kill();
                let _ = lock(&self.child).wait();
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    pub fn stderr_tail(&self) -> Vec<String> {
        lock(&self.stderr_tail).iter().cloned().collect()
    }
}

/// How a helper ended, in words.
pub(crate) fn describe(s: std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = s.signal() {
            let name = match sig {
                6 => "SIGABRT",
                9 => "SIGKILL",
                11 => "SIGSEGV",
                15 => "SIGTERM",
                _ => "",
            };
            return if name.is_empty() {
                format!("killed by signal {sig}")
            } else {
                format!("killed by signal {sig} ({name})")
            };
        }
    }
    match s.code() {
        Some(c) => format!("exited with code {c}"),
        None => "exited".into(),
    }
}
