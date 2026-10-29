//! The standalone application's effects (docs/fx-host.md, decisions.md R-FX-HOST): the
//! chain's audio side, handed to the synth's processor when the application makes it. The
//! chain itself (`ca72_fx`) knows nothing of the synth, and the synth of the chain only this.

use std::sync::Mutex;

mod presets;
pub use crate::fx_view::PresetStatus;
pub use presets::PresetChains;

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
    presets: PresetChains,
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
    let patch = locked(&PATCH).clone();
    if let Some(p) = patch {
        chain.set_pin_mods(std::sync::Arc::new(PatchMods(p)));
    }
    *locked(&HOST) = Some(std::sync::Arc::new(Host {
        presets: PresetChains::new(chain.clone(), safe.is_some(), crate::settings::file()),
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
    fn load_preset(&self, section: Option<String>) {
        self.presets.load(section);
    }

    fn observe_preset(&self, section: Option<String>) {
        self.presets.observe(section);
    }

    fn save_preset(&self) -> Result<Option<String>, String> {
        self.presets.save()
    }

    fn prepare_save_preset(&self) -> std::sync::mpsc::Receiver<Result<Option<String>, String>> {
        self.presets.prepare_save()
    }

    fn preset_status(&self) -> crate::fx_view::PresetStatus {
        self.presets.status()
    }

    fn set_locked(&self, on: bool) {
        self.presets.set_locked(on);
    }

    fn slots(&self) -> Option<Vec<crate::fx_view::SlotInfo>> {
        use crate::fx_view::{Health, SlotInfo};
        self.chain.try_status().map(|st| {
            st.into_iter()
                .map(|s| {
                    let why = s.disabled.as_ref().map(|d| d.why.as_str());
                    SlotInfo {
                        instance: s.id,
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
                    features: e.desc.features.clone(),
                    key: effect_id(e),
                })
                .collect(),
        }
    }

    fn preferences(&self) -> crate::fx_view::Preferences {
        crate::settings::fx_preferences(&crate::settings::file())
    }

    fn set_order(&self, order: ca72_panel::strip::fx::FxOrder) {
        if let Err(e) = crate::settings::keep_fx_order(&crate::settings::file(), order) {
            eprintln!("CA-72 effects: {e}");
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
        self.chain.add(&found)?;
        remember(&found);
        Ok(())
    }

    fn replace(&self, slot: usize, generation: u64, effect: usize) -> Result<(), String> {
        let found = self.found(generation, effect)?;
        self.chain.replace(slot, &found)?;
        remember(&found);
        Ok(())
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
        self.presets.turn_on();
    }

    fn safe_start(&self) -> Option<crate::fx_view::SafeStart> {
        *locked(&self.safe)
    }

    fn params(&self, slot: usize) -> Result<Vec<crate::fx_view::ParamRow>, String> {
        let pins = self.chain.pins();
        Ok(self
            .chain
            .list_params(slot)?
            .into_iter()
            .filter(ca72_fx::pinnable)
            .map(|p| crate::fx_view::ParamRow {
                pinned: pins
                    .iter()
                    .find(|x| x.slot == slot && x.id == p.id)
                    .map(|x| x.target),
                value: p.value.map_or(0.0, |v| p.normalized(v)),
                plain: p.value.unwrap_or(p.default),
                id: p.id,
                name: p.name,
                short: p.short,
                units: p.units,
                steps: p.steps,
            })
            .collect())
    }

    fn param_text(&self, slot: usize, values: &[(u32, f64)]) -> Vec<Option<String>> {
        self.chain
            .param_text(slot, values)
            .unwrap_or_else(|_| vec![None; values.len()])
    }

    fn pins(&self) -> Option<Vec<crate::fx_view::PinInfo>> {
        let pins = self.chain.try_pins()?;
        publish_pinned(&pins);
        Some(pins.into_iter().map(pin_info).collect())
    }

    fn pin(&self, slot: usize, id: u32) -> Result<usize, String> {
        let t = self.chain.pin(slot, id);
        publish_pinned(&self.chain.pins());
        t
    }

    fn touched(&self, slot: usize) -> Option<Vec<u32>> {
        self.chain.try_touched(slot)
    }

    fn repin(&self, target: usize, id: u32) -> Result<(), String> {
        let result = self.chain.repin(target, id);
        publish_pinned(&self.chain.pins());
        result
    }

    fn unpin(&self, target: usize) -> Result<(), String> {
        let r = self.chain.unpin(target);
        publish_pinned(&self.chain.pins());
        r
    }

    fn set_pin_value(&self, target: usize, travel: f64) -> Result<(), String> {
        self.chain.set_pin_value(target, travel)
    }
}

fn pin_info(p: ca72_fx::PinStatus) -> crate::fx_view::PinInfo {
    crate::fx_view::PinInfo {
        target: p.target,
        slot: p.slot,
        effect: p.effect,
        id: p.id,
        name: p.name,
        short: p.short,
        value: p.value,
        plain: p.base,
        steps: p.steps,
        live: p.live,
        missing: p.missing,
    }
}

/// Which pin targets can take a source now ([`crate::lfo::FX_PINNED`]): those of effects
/// that run and have the parameter.
fn publish_pinned(pins: &[ca72_fx::PinStatus]) {
    let bits = pins
        .iter()
        .filter(|p| p.live)
        .fold(0u32, |b, p| b | 1 << p.target);
    crate::lfo::FX_PINNED.store(bits, std::sync::atomic::Ordering::Release);
}

/// The patch's side of the pins' assignments (decisions.md R-FX-PARAMS): the chain reads what
/// moves each pin from the patch when it is saved, and gives it back when it is restored.
struct PatchMods(std::sync::Arc<crate::lfo::Patch>);

impl ca72_fx::PinMods for PatchMods {
    fn get(&self, target: usize) -> Vec<(String, f32)> {
        crate::lfo::Source::all()
            .filter_map(|s| {
                self.0
                    .depth(s, crate::lfo::Target::Fx(target))
                    .map(|d| (s.id(), d))
            })
            .collect()
    }

    fn set(&self, target: usize, mods: &[(String, f32)]) {
        let t = crate::lfo::Target::Fx(target.min(crate::lfo::FX_PINS - 1));
        for s in crate::lfo::Source::all() {
            self.0.remove(s, t);
        }
        for (id, depth) in mods {
            if let Some(s) = crate::lfo::Source::of_id(id) {
                let _ = self.0.assign(s, t, *depth);
            }
        }
    }

    fn revision(&self) -> u64 {
        u64::from(self.0.fx_changes())
    }
}

static PATCH: Mutex<Option<std::sync::Arc<crate::lfo::Patch>>> = Mutex::new(None);

/// The synth instance's patch, whose assignments move the pins: given to the chain (now, or
/// when it is installed).
pub(crate) fn bind_patch(patch: std::sync::Arc<crate::lfo::Patch>) {
    *locked(&PATCH) = Some(patch.clone());
    if let Some(h) = host() {
        h.chain.set_pin_mods(std::sync::Arc::new(PatchMods(patch)));
        publish_pinned(&h.chain.pins());
    }
}

const _: () = assert!(ca72_fx::MAX_PINS == crate::lfo::FX_PINS);
fn effect_id(found: &ca72_fx::Found) -> crate::fx_view::EffectId {
    crate::fx_view::EffectId {
        format: found.desc.format.clone(),
        path: found.path.clone(),
        id: found.desc.id.clone(),
    }
}

fn remember(found: &ca72_fx::Found) {
    if let Err(e) = crate::settings::keep_fx_choice(&crate::settings::file(), effect_id(found)) {
        eprintln!("CA-72 effects: {e}");
    }
}
