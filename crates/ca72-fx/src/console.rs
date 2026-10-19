//! A plain line-command interface to the chain, for testing (docs/fx-host.md). The chain's
//! view in the editor is not designed yet (the owner, 2026-10-18): this stands in for it,
//! clearly temporary, and drives nothing the view will not drive through [`Chain`].
//!
//! The standalone reads these lines from its standard input with `--fx-console`.

use crate::catalog::{self, Catalog, Kind};
use crate::chain::{Chain, Health, MAX_SLOTS};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The commands, for `help`.
pub const HELP: &str = "\
effects console (temporary, for testing):
  scan                  list the effects found (numbered)
  add N                 add effect N of the last scan at the chain's end
  add-file PATH [ID]    add the effect in a plug-in file (its first, or ID)
  replace S N           put effect N of the last scan in place of effect S
  remove S              take out the chain's effect S (1 to 4)
  move S T              move effect S to place T
  bypass S on|off       bypass effect S, or not
  editor S [close]      open (or close) effect S's editor
  param S ID VALUE      set a parameter of effect S (its plain value)
  params S              list effect S's parameters
  chain                 the chain, its latency, each effect's health
  events                what happened to the effects since the last time
  kill S                kill effect S's helper (to try the recovery)
  retry S               try effect S again (one turned off, missing, or off)
  fresh S               load effect S without its saved settings (kept, set aside)
  aside S               give effect S back its settings set aside
  on                    turn the effects back on after a safe start
  wait                  wait until no effect is loading
  save                  save the chain now
  help                  this";

/// The console's state: the last scan, and the events shown so far.
#[derive(Debug)]
pub struct Console {
    pub chain: Chain,
    helper: PathBuf,
    dirs: Vec<PathBuf>,
    cache: Option<PathBuf>,
    catalog: Catalog,
    events: usize,
}

impl Console {
    pub fn new(
        chain: Chain,
        helper: PathBuf,
        dirs: Vec<PathBuf>,
        cache: Option<PathBuf>,
    ) -> Console {
        Console {
            chain,
            helper,
            dirs,
            cache,
            catalog: Catalog::default(),
            events: 0,
        }
    }

    /// Runs one command; what it says.
    pub fn run(&mut self, line: &str) -> String {
        let words: Vec<&str> = line.split_whitespace().collect();
        let slot = |i: usize| -> Result<usize, String> {
            let s: usize = words
                .get(i)
                .ok_or("which effect? (1 to 4)")?
                .parse()
                .map_err(|_| "an effect is a number, 1 to 4")?;
            if s == 0 || s > MAX_SLOTS {
                return Err("an effect is a number, 1 to 4".into());
            }
            Ok(s - 1)
        };
        let result: Result<String, String> = match words.first().copied() {
            None => Ok(String::new()),
            Some("help") => Ok(HELP.into()),
            Some("scan") => Ok(self.scan()),
            Some("add") => (|| {
                let n: usize = words
                    .get(1)
                    .ok_or("which effect of the scan?")?
                    .parse()
                    .map_err(|_| "a number from the scan")?;
                let found = self
                    .catalog
                    .effects()
                    .nth(n.wrapping_sub(1))
                    .cloned()
                    .ok_or("no such effect in the last scan")?;
                let at = self.chain.add(&found)?;
                Ok(format!("effect {}: {}", at + 1, found.desc.name))
            })(),
            Some("add-file") => (|| {
                let path = Path::new(words.get(1).ok_or("which file?")?);
                let plugins = crate::locate::scan(&self.helper, path, Duration::from_secs(10))?;
                let desc = match words.get(2) {
                    Some(id) => plugins.into_iter().find(|d| d.id == *id),
                    None => plugins
                        .into_iter()
                        .find(|d| catalog::kind(d) != Kind::Instrument),
                }
                .ok_or("no such effect in the file")?;
                let found = catalog::Found {
                    path: path.to_path_buf(),
                    kind: catalog::kind(&desc),
                    desc,
                };
                let at = self.chain.add(&found)?;
                Ok(format!("effect {}: {}", at + 1, found.desc.name))
            })(),
            Some("replace") => (|| {
                let s = slot(1)?;
                let n: usize = words
                    .get(2)
                    .ok_or("replace S N")?
                    .parse()
                    .map_err(|_| "a number from the scan")?;
                let found = self
                    .catalog
                    .effects()
                    .nth(n.wrapping_sub(1))
                    .cloned()
                    .ok_or("no such effect in the last scan")?;
                self.chain.replace(s, &found)?;
                Ok(self.show())
            })(),
            Some("remove") => slot(1)
                .and_then(|s| self.chain.remove(s))
                .map(|()| self.show()),
            Some("move") => slot(1)
                .and_then(|s| Ok((s, slot(2)?)))
                .and_then(|(s, t)| self.chain.move_to(s, t))
                .map(|()| self.show()),
            Some("bypass") => slot(1)
                .and_then(|s| match words.get(2).copied() {
                    Some("on") => self.chain.set_bypass(s, true),
                    Some("off") => self.chain.set_bypass(s, false),
                    _ => Err("bypass S on|off".into()),
                })
                .map(|()| self.show()),
            Some("editor") => slot(1).and_then(|s| {
                if words.get(2) == Some(&"close") {
                    self.chain.close_editor(s).map(|v| v.to_string())
                } else {
                    self.chain.open_editor(s).map(|v| v.to_string())
                }
            }),
            Some("param") => (|| {
                let s = slot(1)?;
                let id: u32 = words
                    .get(2)
                    .and_then(|w| w.parse().ok())
                    .ok_or("param S ID VALUE")?;
                let v: f64 = words
                    .get(3)
                    .and_then(|w| w.parse().ok())
                    .ok_or("param S ID VALUE")?;
                self.chain.set_param(s, id, v)?;
                Ok(format!("effect {}: {id} = {v}", s + 1))
            })(),
            Some("params") => slot(1).map(|s| {
                self.chain
                    .params(s)
                    .iter()
                    .filter(|p| !p.hidden)
                    .map(|p| {
                        format!(
                            "  {} {:?} {}..{} (default {})",
                            p.id, p.name, p.min, p.max, p.default
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }),
            Some("chain") => Ok(self.show()),
            Some("events") => Ok(self.events()),
            Some("kill") => slot(1)
                .and_then(|s| self.chain.kill(s))
                .map(|()| "killed".into()),
            Some("save") => self.chain.save_now().map(|()| "saved".into()),
            Some("retry") => slot(1)
                .and_then(|s| self.chain.retry(s))
                .map(|()| self.show()),
            Some("fresh") => slot(1)
                .and_then(|s| self.chain.load_without_settings(s))
                .map(|()| self.show()),
            Some("aside") => slot(1)
                .and_then(|s| self.chain.try_set_aside(s))
                .map(|()| self.show()),
            Some("on") => {
                self.chain.turn_on();
                Ok(self.show())
            }
            Some("wait") => {
                let done = self.chain.wait_loaded(Duration::from_secs(120));
                Ok(format!(
                    "{}\n{}",
                    if done { "loaded" } else { "still loading" },
                    self.show()
                ))
            }
            Some(w) => Err(format!("`{w}`? (help lists the commands)")),
        };
        result.unwrap_or_else(|e| format!("error: {e}"))
    }

    fn scan(&mut self) -> String {
        self.catalog = catalog::scan(
            &self.helper,
            &self.dirs,
            self.cache.as_deref(),
            Duration::from_secs(10),
        );
        let mut out = Vec::new();
        for (i, f) in self.catalog.effects().enumerate() {
            out.push(format!(
                "{:>3}. {} ({}, {}{}) {}",
                i + 1,
                f.desc.name,
                f.desc.vendor,
                f.desc.format,
                if f.kind == Kind::Unknown {
                    ", kind not said"
                } else {
                    ""
                },
                f.path.display()
            ));
        }
        let instruments = self
            .catalog
            .plugins
            .iter()
            .filter(|f| f.kind == Kind::Instrument)
            .count();
        out.push(format!(
            "{} effects; {instruments} instruments left out; {} files not listed; {} listed afresh",
            self.catalog.effects().count(),
            self.catalog.unlisted.len(),
            self.catalog.scanned
        ));
        for u in &self.catalog.unlisted {
            out.push(format!("  not listed: {}: {}", u.path.display(), u.reason));
        }
        out.join("\n")
    }

    /// The chain, as a few lines.
    pub fn show(&self) -> String {
        let st = self.chain.status();
        if st.is_empty() {
            return "the chain is empty".into();
        }
        let rate = self.chain.sample_rate();
        let mut out: Vec<String> = st
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "{}. {} ({}){}{}{}{} latency {} crashes {} misses {} at frame {}{}",
                    i + 1,
                    s.name,
                    s.format,
                    if s.bypass { " BYPASSED" } else { "" },
                    match s.health {
                        Health::Running => "",
                        Health::Loading => " LOADING",
                        Health::Restarting => " RESTARTING",
                        Health::Disabled => " TURNED OFF",
                        Health::Missing => " MISSING",
                        Health::Off => " OFF (safe start)",
                    },
                    s.disabled
                        .as_ref()
                        .map(|d| format!(" [{}, {} time(s)]", d.why, d.times))
                        .unwrap_or_default(),
                    if s.set_aside {
                        " (its settings set aside)"
                    } else {
                        ""
                    },
                    s.latency,
                    s.crashes,
                    s.misses,
                    s.position,
                    s.error
                        .as_deref()
                        .map(|e| format!(" ({e})"))
                        .unwrap_or_default()
                )
            })
            .collect();
        let latency = self.chain.latency();
        out.push(format!(
            "latency {latency} frames ({:.1} ms at {rate} Hz)",
            f64::from(latency) * 1000.0 / rate
        ));
        out.join("\n")
    }

    fn events(&mut self) -> String {
        let (events, n) = self.chain.events_from(self.events);
        self.events = n;
        if events.is_empty() {
            return "nothing new".into();
        }
        events
            .iter()
            .map(|e| format!("{} {}: {}", e.effect, e.kind, e.detail))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
