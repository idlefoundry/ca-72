//! The standalone application's effects (docs/fx-host.md, decisions.md R-FX-HOST): the
//! chain's audio side, handed to the synth's processor when the application makes it. The
//! chain itself (`ca72_fx`) knows nothing of the synth, and the synth of the chain only this.

use std::sync::Mutex;

/// The share of a block's period from its start by which every effect must have returned
/// its block, after the voices' own (`engine::DEADLINE`, 0.75): an effect late past it
/// passes the sound on dry for that block.
pub const DEADLINE: f64 = 0.9;

static RUNNER: Mutex<Option<ca72_fx::Runner>> = Mutex::new(None);

/// The chain's audio side, for the next instance of the synth made: the standalone's (one
/// instance), before it starts.
pub fn install(runner: ca72_fx::Runner) {
    *RUNNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(runner);
}

/// The audio side installed, if any (the plug-ins: none).
pub(crate) fn take_runner() -> Option<ca72_fx::Runner> {
    RUNNER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

/// The chain's control side and what finding effects needs, for the effects' drawer
/// (decisions.md R-FX-DRAWER): the standalone's one chain, and the effects found on this
/// computer, kept for the application's life (the editor closed and opened again finds them
/// still).
pub struct Host {
    chain: ca72_fx::Chain,
    helper: std::path::PathBuf,
    dirs: Vec<std::path::PathBuf>,
    cache: Option<std::path::PathBuf>,
    found: std::sync::Arc<Mutex<Found>>,
    /// Why the effects are off this time, if this is a safe start (decisions.md R-FX-SAFETY):
    /// none once they are turned back on.
    safe: Mutex<Option<crate::fx_view::SafeStart>>,
}

/// The effects found: the last scan's, in the list's order (by name), its number; a scan under
/// way.
#[derive(Default)]
struct Found {
    effects: Vec<ca72_fx::Found>,
    /// Plug-in files the last scan could not list (they crashed or hung their helper, or are
    /// not plug-ins).
    unlisted: usize,
    generation: u64,
    scanning: bool,
    scanned: bool,
}

impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("helper", &self.helper)
            .finish_non_exhaustive()
    }
}

static HOST: Mutex<Option<std::sync::Arc<Host>>> = Mutex::new(None);

fn locked<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The chain's control side, for the editor's drawer: `helper` lists the effects in `dirs`,
/// their listings kept in `cache`; `safe`, why the chain was put back off, if it was (a safe
/// start, decisions.md R-FX-SAFETY).
pub fn install_host(
    chain: ca72_fx::Chain,
    helper: std::path::PathBuf,
    dirs: Vec<std::path::PathBuf>,
    cache: Option<std::path::PathBuf>,
    safe: Option<ca72_fx::launch::SafeStart>,
) {
    *locked(&HOST) = Some(std::sync::Arc::new(Host {
        chain,
        helper,
        dirs,
        cache,
        found: std::sync::Arc::default(),
        safe: Mutex::new(safe.map(|s| match s {
            ca72_fx::launch::SafeStart::DidNotClose => crate::fx_view::SafeStart::DidNotClose,
            ca72_fx::launch::SafeStart::Asked => crate::fx_view::SafeStart::Asked,
        })),
    }));
}

/// How long a failed try of an effect's old settings is said on its slot.
const TRY_FAILED_SHOWN: std::time::Duration = std::time::Duration::from_secs(5);

/// Whether a safe start is asked for as the application opens (decisions.md R-FX-SAFETY):
/// Shift held, or on macOS Option (or Shift). Read once, before the window opens, from the
/// keyboard's state as the system has it. On Linux that is the X server's (XWayland's under
/// Wayland, which knows the keys only while one of its windows has the keyboard): there
/// `--safe-start` is the sure way.
pub fn safe_start_keys_held() -> bool {
    keys::held()
}

#[cfg(target_os = "linux")]
mod keys {
    pub fn held() -> bool {
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::{ConnectionExt as _, KeyButMask};
        let Ok((conn, n)) = x11rb::connect(None) else {
            return false;
        };
        let root = conn.setup().roots[n].root;
        conn.query_pointer(root)
            .ok()
            .and_then(|c| c.reply().ok())
            .is_some_and(|r| r.mask.contains(KeyButMask::SHIFT))
    }
}

#[cfg(target_os = "macos")]
mod keys {
    /// `[NSEvent modifierFlags]`: the keys held now, whichever application is in front.
    // (objc 0.2's `msg_send!` asks after a `cargo-clippy` feature this crate does not have.)
    #[allow(unsafe_code, unexpected_cfgs)]
    pub fn held() -> bool {
        use objc::{class, msg_send, sel, sel_impl};
        const SHIFT: u64 = 1 << 17;
        const OPTION: u64 = 1 << 19;
        // SAFETY: a class method of AppKit's NSEvent, which takes nothing and returns the
        // modifier flags as an NSUInteger; callable from any thread.
        let flags: u64 = unsafe { msg_send![class!(NSEvent), modifierFlags] };
        flags & (SHIFT | OPTION) != 0
    }
}

#[cfg(target_os = "windows")]
mod keys {
    #[allow(unsafe_code)]
    pub fn held() -> bool {
        use winapi::um::winuser::{GetAsyncKeyState, VK_SHIFT};
        // SAFETY: GetAsyncKeyState takes a virtual key code and reads the keyboard's state;
        // the high bit is set while the key is down.
        (unsafe { GetAsyncKeyState(VK_SHIFT) } as u16) & 0x8000 != 0
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod keys {
    pub fn held() -> bool {
        false
    }
}

/// The chain's control side, if the application installed it.
pub(crate) fn host() -> Option<std::sync::Arc<Host>> {
    locked(&HOST).clone()
}

impl Host {
    /// Effect `effect` of scan `generation`, if that is still the list.
    fn found(&self, generation: u64, effect: usize) -> Result<ca72_fx::Found, String> {
        let f = locked(&self.found);
        if f.generation != generation {
            return Err("the effects were listed again meanwhile".into());
        }
        f.effects
            .get(effect)
            .cloned()
            .ok_or_else(|| "no such effect".into())
    }
}

impl crate::fx_view::Effects for Host {
    fn slots(&self) -> Option<Vec<crate::fx_view::SlotInfo>> {
        use crate::fx_view::{Health, SlotInfo};
        self.chain.try_status().map(|st| {
            st.into_iter()
                .map(|s| {
                    let why = s.disabled.as_ref().map(|d| d.why.as_str());
                    SlotInfo {
                        health: match s.health {
                            ca72_fx::Health::Running => Health::Running,
                            ca72_fx::Health::Loading => Health::Loading,
                            ca72_fx::Health::Restarting => Health::Restarting,
                            ca72_fx::Health::Disabled if why == Some("hung") => {
                                Health::NotResponding
                            }
                            ca72_fx::Health::Disabled => Health::Crashed,
                            ca72_fx::Health::Missing => Health::Missing,
                            ca72_fx::Health::Off => Health::Off,
                        },
                        by_settings: why == Some("settings"),
                        has_state: s.has_state,
                        set_aside: s.set_aside,
                        try_failed: s.try_failed.is_some_and(|t| t.elapsed() < TRY_FAILED_SHOWN),
                        name: s.name,
                        vendor: s.vendor,
                        format: s.format,
                        bypass: s.bypass,
                    }
                })
                .collect()
        })
    }

    fn active(&self) -> Option<bool> {
        self.chain.try_active()
    }

    fn catalogue(&self) -> crate::fx_view::Catalogue {
        let f = locked(&self.found);
        crate::fx_view::Catalogue {
            scanning: f.scanning,
            scanned: f.scanned,
            unlisted: f.unlisted,
            generation: f.generation,
            effects: f
                .effects
                .iter()
                .map(|e| crate::fx_view::Listing {
                    name: e.desc.name.clone(),
                    vendor: e.desc.vendor.clone(),
                    format: e.desc.format.clone(),
                })
                .collect(),
        }
    }

    fn scan(&self, again: bool) {
        {
            let mut f = locked(&self.found);
            if f.scanning || (f.scanned && !again) {
                return;
            }
            f.scanning = true;
        }
        let (found, helper, dirs, cache) = (
            self.found.clone(),
            self.helper.clone(),
            self.dirs.clone(),
            self.cache.clone(),
        );
        let scanned = std::thread::Builder::new()
            .name("ca72-fx-scan".into())
            .spawn(move || {
                // SCAN AGAIN tries again the files that crashed or hung their helper when
                // last listed; else they are remembered (R-FX-SAFETY).
                let catalog = ca72_fx::catalog::scan_as(
                    &helper,
                    &dirs,
                    cache.as_deref(),
                    std::time::Duration::from_secs(10),
                    again,
                );
                for u in &catalog.unlisted {
                    eprintln!(
                        "CA-72 effects: not listed: {}: {}",
                        u.path.display(),
                        u.reason
                    );
                }
                let mut effects: Vec<ca72_fx::Found> = catalog.effects().cloned().collect();
                effects.sort_by(|a, b| {
                    let key = |f: &ca72_fx::Found| {
                        (
                            f.desc.name.to_lowercase(),
                            f.desc.vendor.to_lowercase(),
                            f.desc.format.clone(),
                        )
                    };
                    key(a).cmp(&key(b))
                });
                let mut f = locked(&found);
                f.effects = effects;
                f.unlisted = catalog.unlisted.len();
                f.generation += 1;
                f.scanning = false;
                f.scanned = true;
            });
        if scanned.is_err() {
            locked(&self.found).scanning = false;
        }
    }

    fn add(&self, generation: u64, effect: usize) -> Result<(), String> {
        let found = self.found(generation, effect)?;
        self.chain.add(&found).map(|_| ())
    }

    fn replace(&self, slot: usize, generation: u64, effect: usize) -> Result<(), String> {
        let found = self.found(generation, effect)?;
        self.chain.replace(slot, &found)
    }

    fn remove(&self, slot: usize) -> Result<(), String> {
        self.chain.remove(slot)
    }

    fn move_to(&self, from: usize, to: usize) -> Result<(), String> {
        self.chain.move_to(from, to)
    }

    fn set_bypass(&self, slot: usize, on: bool) -> Result<(), String> {
        self.chain.set_bypass(slot, on)
    }

    fn open_editor(&self, slot: usize) -> Result<(), String> {
        self.chain.open_editor(slot).map(|_| ())
    }

    fn retry(&self, slot: usize) -> Result<(), String> {
        self.chain.retry(slot)
    }

    fn without_settings(&self, slot: usize) -> Result<(), String> {
        self.chain.load_without_settings(slot)
    }

    fn old_settings(&self, slot: usize) -> Result<(), String> {
        self.chain.try_set_aside(slot)
    }

    fn turn_on(&self) {
        *locked(&self.safe) = None;
        self.chain.turn_on();
    }

    fn safe_start(&self) -> Option<crate::fx_view::SafeStart> {
        *locked(&self.safe)
    }
}
