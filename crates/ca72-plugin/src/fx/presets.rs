//! Preset chains use the session’s snapshots and background restore (R-FX-PRESETS).

use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use ca72_fx::{Chain, Restore, SavedChain};

use crate::fx_view::PresetStatus;

use super::locked;

/// The standalone’s preset changes, serialised off the editor and audio threads.
#[derive(Debug)]
pub struct PresetChains {
    tx: mpsc::Sender<Request>,
    status: Arc<Mutex<PresetStatus>>,
}

enum Request {
    Load(Option<String>),
    Observe(Option<String>),
    Lock(bool),
    TurnOn,
    Save(mpsc::Sender<Result<Option<String>, String>>),
}

pub(super) fn section(chain: &SavedChain) -> Result<String, String> {
    // The same record as effects.json, in a TOML section older readers ignore. JSON keeps
    // optional state and safety records in their established form, without another schema.
    let json = serde_json::to_string(chain).map_err(|e| e.to_string())?;
    let mut table = toml::Table::new();
    table.insert("chain".into(), json.into());
    let mut root = toml::Table::new();
    root.insert("effects".into(), table.into());
    toml::to_string(&root).map_err(|e| e.to_string())
}

fn chain_of(section: Option<&str>) -> Result<SavedChain, String> {
    let Some(section) = section else {
        return Ok(SavedChain {
            format: ca72_fx::chain::FORMAT,
            slots: Vec::new(),
        });
    };
    let root: toml::Table = section
        .parse()
        .map_err(|e: toml::de::Error| e.to_string())?;
    let json = root
        .get("effects")
        .and_then(toml::Value::as_table)
        .and_then(|t| t.get("chain"))
        .and_then(toml::Value::as_str)
        .ok_or("The preset’s effects section is not understood")?;
    let chain = SavedChain::load_bytes(json.as_bytes())?;
    if chain.format != ca72_fx::chain::FORMAT || chain.slots.len() > ca72_fx::MAX_SLOTS {
        return Err("The preset’s effects format or chain length is not supported".into());
    }
    Ok(chain)
}

impl PresetChains {
    /// A chain and this computer’s settings; `off` keeps a safe start in force.
    pub fn new(chain: Chain, off: bool, path: std::path::PathBuf) -> Self {
        let is_locked = crate::settings::fx_locked(&path);
        let status = Arc::new(Mutex::new(PresetStatus {
            locked: is_locked,
            ..PresetStatus::default()
        }));
        let (tx, rx) = mpsc::channel();
        let shown = status.clone();
        std::thread::Builder::new()
            .name("ca72-fx-presets".into())
            .spawn(move || {
                let mut is_locked = is_locked;
                let mut off = off;
                let mut preset = None;
                let mut seen = None;
                loop {
                    let request = match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(r) => Some(r),
                        Err(mpsc::RecvTimeoutError::Timeout) => None,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    let revision = chain.changes();
                    if request.is_none() && seen == Some(revision) {
                        continue;
                    }
                    let mut restore = false;
                    let load = matches!(request, Some(Request::Load(_)));
                    match request {
                        Some(Request::Load(raw) | Request::Observe(raw)) => {
                            match chain_of(raw.as_deref()) {
                                Ok(saved) => {
                                    preset = Some(saved);
                                    restore = load && !is_locked;
                                }
                                Err(e) => {
                                    preset = None;
                                    eprintln!("CA-72 effects: {e}");
                                }
                            }
                        }
                        Some(Request::Lock(on)) => {
                            restore = is_locked && !on;
                            is_locked = on;
                            if let Err(e) = crate::settings::keep_fx_locked(&path, on) {
                                eprintln!("CA-72 effects: {e}");
                            }
                        }
                        Some(Request::Save(reply)) => {
                            let _ = reply.send(section(&chain.capture()).map(Some));
                        }
                        Some(Request::TurnOn) => {
                            off = false;
                            chain.turn_on();
                        }
                        None => {}
                    }
                    if let Some(saved) = &preset {
                        if restore && chain.try_saved().as_ref() != Some(saved) {
                            for e in chain.restore_as(
                                saved,
                                if off {
                                    Restore::Off
                                } else {
                                    Restore::Background
                                },
                            ) {
                                eprintln!("CA-72 effects: {e}");
                            }
                        }
                        if let Some(playing) = chain.try_saved() {
                            *locked(&shown) = status_of(is_locked, saved, &playing);
                            seen = Some(revision);
                        }
                    } else {
                        seen = Some(revision);
                        *locked(&shown) = PresetStatus {
                            locked: is_locked,
                            ..PresetStatus::default()
                        };
                    }
                }
            })
            .expect("the effects’ preset thread");
        Self { tx, status }
    }

    /// Queue fresh snapshots in order with preset changes, without waiting for a helper.
    pub fn prepare_save(&self) -> mpsc::Receiver<Result<Option<String>, String>> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(Request::Save(tx));
        rx
    }

    /// Fresh state, by the session’s safe snapshot path. Control threads only.
    pub fn save(&self) -> Result<Option<String>, String> {
        self.prepare_save().recv().map_err(|e| e.to_string())?
    }

    pub fn load(&self, section: Option<String>) {
        let _ = self.tx.send(Request::Load(section));
    }

    /// Remember a recalled session’s preset without replacing its playing chain.
    pub fn observe(&self, section: Option<String>) {
        let _ = self.tx.send(Request::Observe(section));
    }

    pub fn set_locked(&self, on: bool) {
        {
            let mut status = locked(&self.status);
            status.locked = on;
            if !on {
                status.held = None;
            }
        }
        let _ = self.tx.send(Request::Lock(on));
    }

    /// End a safe start, in order with preset changes already requested.
    pub fn turn_on(&self) {
        let _ = self.tx.send(Request::TurnOn);
    }

    pub fn status(&self) -> PresetStatus {
        locked(&self.status).clone()
    }
}

fn status_of(is_locked: bool, preset: &SavedChain, playing: &SavedChain) -> PresetStatus {
    let differs = preset != playing;
    PresetStatus {
        locked: is_locked,
        differs,
        held: if is_locked && differs && !preset.slots.is_empty() {
            Some(std::array::from_fn(|i| {
                preset
                    .slots
                    .get(i)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| "–".into())
            }))
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_signs_require_a_lock_a_difference_and_a_nonempty_preset() {
        let empty = chain_of(None).unwrap();
        let mut preset = empty.clone();
        preset.slots.push(ca72_fx::SavedSlot {
            format: "clap".into(),
            path: "not-installed.clap".into(),
            plugin: "test".into(),
            name: "Preset reverb".into(),
            vendor: "Maker".into(),
            bypass: false,
            state: Some("deadbeef".into()),
            disabled: None,
            set_aside: None,
        });
        assert!(status_of(false, &preset, &empty).held.is_none());
        assert!(status_of(true, &preset, &preset).held.is_none());
        assert!(status_of(true, &empty, &preset).held.is_none());
        let held = status_of(true, &preset, &empty).held.unwrap();
        assert_eq!(held, ["Preset reverb", "–", "–", "–"]);
        assert_eq!(chain_of(Some(&section(&preset).unwrap())).unwrap(), preset);
        assert!(chain_of(Some("[effects]\nchain = 'unknown'\n")).is_err());
    }
}
