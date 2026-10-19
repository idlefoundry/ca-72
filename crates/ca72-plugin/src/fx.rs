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
}

/// The effects found: the last scan's, in the list's order (by name), its number; a scan under
/// way.
#[derive(Default)]
struct Found {
    effects: Vec<ca72_fx::Found>,
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
/// their listings kept in `cache`.
pub fn install_host(
    chain: ca72_fx::Chain,
    helper: std::path::PathBuf,
    dirs: Vec<std::path::PathBuf>,
    cache: Option<std::path::PathBuf>,
) {
    *locked(&HOST) = Some(std::sync::Arc::new(Host {
        chain,
        helper,
        dirs,
        cache,
        found: std::sync::Arc::default(),
    }));
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
                .map(|s| SlotInfo {
                    name: s.name,
                    vendor: s.vendor,
                    format: s.format,
                    bypass: s.bypass,
                    health: match s.health {
                        ca72_fx::Health::Running => Health::Running,
                        ca72_fx::Health::Restarting => Health::Restarting,
                        ca72_fx::Health::Failed => Health::Failed,
                        ca72_fx::Health::Missing => Health::Missing,
                    },
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
                let catalog = ca72_fx::catalog::scan(
                    &helper,
                    &dirs,
                    cache.as_deref(),
                    std::time::Duration::from_secs(10),
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
}
