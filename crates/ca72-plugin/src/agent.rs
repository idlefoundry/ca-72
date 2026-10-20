//! Agents (issue #76; decisions.md R-MCP, docs/mcp.md): an AI agent operates the CA-72 as a
//! player does, through the Model Context Protocol. An agent's client starts the bridge,
//! `ca72-mcp`, which speaks MCP on its standard input and output (newline-delimited
//! JSON-RPC 2.0, as the owner's DAW's `daw mcp` does: DAW-v3's `crates/daw-api/src/mcp.rs`,
//! DESIGN 7.6, whose patterns are reused here), finds the CA-72s open on this computer that
//! allow agents ([`discovery`]) and passes each tool's call to the one chosen over a local
//! socket ([`server`]). There the editor carries it out on its own thread, at its next frame,
//! through the same paths its pointer, its keys and its drawer take (`editor/agent.rs`): a
//! parameter set through the host's context as a gesture of its own, a preset chosen as the
//! drawer chooses it, a note played as the on-screen keyboard plays one. Nothing reaches the
//! audio thread but what the editor's own changes do (the parameters' atomics, the local
//! notes' queue).
//!
//! Off unless the player allows it (SETTINGS: AGENTS, or `--agents` for one session): until
//! then no socket exists. Only the standalone application listens in this version; a
//! plug-in in a host is the next step (docs/mcp.md).

pub mod bridge;
pub mod discovery;
pub mod mcp;
pub mod server;
pub mod tools;
pub mod values;

use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};

/// The version of the conversation between the bridge and a CA-72 (`hello`'s `protocol`): a
/// bridge refuses an instance that speaks another.
pub const PROTOCOL: u64 = 1;

/// JSON-RPC's error codes, and ours (-32000 to -32099, the range it leaves to servers).
pub mod codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL: i64 = -32603;
    /// The instance's window did not take the request in its time.
    pub const TIMEOUT: i64 = -32001;
    /// Too many requests waiting, or too many clients.
    pub const BUSY: i64 = -32002;
    /// No CA-72 to talk to (none open with agents allowed, or the one chosen has closed).
    pub const NO_INSTANCE: i64 = -32003;
    /// Refused as the panel would refuse it (DETUNE in SCATTER, a name taken, ...).
    pub const REFUSED: i64 = -32004;
    /// Several CA-72s are open and none is chosen.
    pub const AMBIGUOUS: i64 = -32005;
}

/// An error a request ends in: its code, what to say, and anything that helps put it right
/// (the range a value must be in, the choices a selector has).
#[derive(Clone, Debug, PartialEq)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl ApiError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn params(message: impl Into<String>) -> Self {
        ApiError::new(codes::INVALID_PARAMS, message)
    }

    pub fn refused(message: impl Into<String>) -> Self {
        ApiError::new(codes::REFUSED, message)
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// As JSON-RPC's `error` member.
    pub fn to_json(&self) -> Value {
        let mut e = json!({"code": self.code, "message": self.message});
        if let Some(d) = &self.data {
            e["data"] = d.clone();
        }
        e
    }

    /// From JSON-RPC's `error` member (what an instance answered).
    pub fn from_json(v: &Value) -> Self {
        ApiError {
            code: v
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or(codes::INTERNAL),
            message: v
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_owned(),
            data: v.get("data").cloned(),
        }
    }
}

pub type Reply = Result<Value, ApiError>;

/// `--agents` given to the standalone: agents allowed for this session whatever the setting.
static FORCED: AtomicBool = AtomicBool::new(false);

/// The standalone application's end (`bin/standalone.rs`'s `finish`, decisions.md R-QUIT):
/// this process's record and socket removed if anything left them (its editor's server
/// removes them as the window closes), so that no bridge finds a CA-72 that has gone.
pub fn end_of_app() {
    let dir = discovery::dir();
    let id = server::own_id("standalone");
    let _ = std::fs::remove_file(discovery::record_path(&dir, &id));
    let _ = std::fs::remove_file(discovery::socket_path(&dir, &id));
}

/// Agents allowed for this session (the standalone's `--agents`).
pub fn force(on: bool) {
    FORCED.store(on, Ordering::Relaxed);
}

pub fn forced() -> bool {
    FORCED.load(Ordering::Relaxed)
}

/// An argument of a call: `args[key]`, absent or null as none.
pub fn arg<'a>(args: &'a Value, key: &str) -> Option<&'a Value> {
    args.get(key).filter(|v| !v.is_null())
}

/// A string argument that must be there.
pub fn text_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ApiError> {
    match arg(args, key) {
        Some(Value::String(s)) => Ok(s),
        Some(_) => Err(ApiError::params(format!("`{key}` must be a string"))),
        None => Err(ApiError::params(format!("`{key}` is required"))),
    }
}

/// A boolean argument, `default` when absent.
pub fn flag_arg(args: &Value, key: &str, default: bool) -> Result<bool, ApiError> {
    match arg(args, key) {
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(ApiError::params(format!("`{key}` must be true or false"))),
        None => Ok(default),
    }
}
