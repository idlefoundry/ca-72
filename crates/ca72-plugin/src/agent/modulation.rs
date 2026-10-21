//! What the modulation sources (the four LFOs and the six macros) modulate, for the agents
//! (decisions.md R-LFO, R-MACROS, docs/mcp.md): `modulation.list`, `modulation.set` and
//! `modulation.remove`, carried out on the patch as the LFO page's and the macros' drag and
//! menus are (`lfo::Patch`, patch state: no host gesture), and the macros' typed names. The LFOs'
//! own controls and the macros' knobs are parameters (`lfoN_rate`, `macroN`), set with
//! `parameters.set`.
//!
//! The standalone's effects' pinned parameters are targets too (decisions.md R-FX-PARAMS:
//! `fx1` to `fx16`, while pinned to an effect that runs), and `effects.parameters`,
//! `effects.pin` and `effects.unpin` list an effect's parameters and pin or unpin them.

use serde_json::{Value, json};

use super::{ApiError, Reply, arg, values};
use crate::lfo::{self, Edges, Refused, Source, Target};
use crate::params::Ca72Params;

/// A source as an agent names it: its id (`lfo1`, `macro1`), its label (`LFO 1`, `MACRO 1`), a
/// macro's typed name, or its number (1 to 4, an LFO's).
pub fn source_of(p: &Ca72Params, v: Option<&Value>) -> Result<Source, ApiError> {
    let bad = || {
        ApiError::params(
            "`source` must be an LFO (lfo1 to lfo4, \"LFO 1\" or 1 to 4) or a macro (macro1 to \
             macro6, \"MACRO 1\" or its name)",
        )
        .with_data(json!({"sources": Source::all().map(|s| s.id()).collect::<Vec<_>>()}))
    };
    match v {
        Some(Value::Number(n)) => {
            let n = n
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(bad)?;
            (1..=lfo::LFOS)
                .contains(&n)
                .then(|| Source::Lfo(n - 1))
                .ok_or_else(bad)
        }
        Some(Value::String(s)) => {
            let t = s.trim().to_lowercase().replace(' ', "");
            let named = lfo::name_within(s);
            Source::of_id(&t)
                .or_else(|| {
                    (0..lfo::MACROS)
                        .find(|&n| !named.is_empty() && p.lfo_patch.name(n) == named)
                        .map(Source::Macro)
                })
                .or_else(|| {
                    t.parse::<usize>()
                        .ok()
                        .and_then(|n| n.checked_sub(1))
                        .and_then(|n| (n < lfo::LFOS).then_some(Source::Lfo(n)))
                })
                .ok_or_else(bad)
        }
        _ => Err(ApiError::params("`source` is required")),
    }
}

fn label(s: Source) -> String {
    match s {
        Source::Lfo(n) => format!("LFO {}", n + 1),
        Source::Macro(n) => format!("MACRO {}", n + 1),
    }
}

/// A source's label with a macro's typed name: `MACRO 1 (BRIGHT)`.
fn named(p: &Ca72Params, s: Source) -> String {
    match s.macro_n().map(|n| p.lfo_patch.name(n)) {
        Some(name) if !name.is_empty() => format!("{} ({name})", label(s)),
        _ => label(s),
    }
}

/// The knob an agent names (an id, a panel label or a host name; an oscillator's or an LFO's
/// WAVEFORM, its CONTINUOUS shape; an LFO's RATE, PHASE, RISE or LEVEL: R-LFO's addendum), if it
/// can take a source now, as the panel's drag allows.
fn target_of(p: &Ca72Params, v: Option<&Value>) -> Result<Target, ApiError> {
    let Some(Value::String(key)) = v else {
        return Err(ApiError::params("`target` must be a knob's id or label"));
    };
    let table = values::table(p);
    // (An id in any case: TEMPO has no parameter to be found by, R-LFO.)
    let lower = key.trim().to_ascii_lowercase();
    let id = values::find(&table, key).map(|e| e.id).unwrap_or(&lower);
    lfo::target_of(p, id).ok_or_else(|| {
        let why = match Target::of_id(id) {
            Some(
                Target::Osc1Shape | Target::Osc2Shape | Target::Osc3Shape | Target::LfoShape(_),
            ) => {
                format!("{key} takes a source only through its WAVEFORM while CONTINUOUS")
            }
            Some(Target::LfoRate(_)) => {
                format!("{key} cannot take a source now (its LFO is timed)")
            }
            Some(Target::Fx(_)) => {
                format!("{key} is not pinned to an effect that runs (effects.pin pins one)")
            }
            Some(Target::Tempo) => format!(
                "{key} is the standalone's TEMPO: a plug-in follows its host's tempo (standalone \
                 only)"
            ),
            Some(_) => format!("{key} cannot take a source now (its oscillator is timed)"),
            None if id.ends_with("_waveform")
                && (id.starts_with("osc") || id.starts_with("lfo")) =>
            {
                format!("{key} takes a source only while CONTINUOUS")
            }
            None => format!("{key} is not a knob a source can modulate"),
        };
        ApiError::refused(why).with_data(json!({"targets": targets_now(p)}))
    })
}

/// The knobs that can take a source now, by their controls' ids.
fn targets_now(p: &Ca72Params) -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = Target::ALL
        .iter()
        .map(|t| t.id())
        .filter(|id| lfo::target_of(p, id).is_some())
        .collect();
    for (n, w) in ["osc1_waveform", "osc2_waveform", "osc3_waveform"]
        .into_iter()
        .enumerate()
    {
        if p.continuous(n) {
            ids.push(w);
        }
    }
    // (An LFO's WAVEFORM likewise: R-LFO's addendum.)
    for w in [
        "lfo1_waveform",
        "lfo2_waveform",
        "lfo3_waveform",
        "lfo4_waveform",
    ] {
        if lfo::target_of(p, w).is_some() {
            ids.push(w);
        }
    }
    ids
}

/// Whether an assignment on `t` moves anything now (a shape's oscillator CONTINUOUS, a
/// FREQUENCY's not timed).
fn live(p: &Ca72Params, t: Target) -> bool {
    match t {
        Target::Fx(k) => lfo::FX_PINNED.load(std::sync::atomic::Ordering::Acquire) & (1 << k) != 0,
        Target::Osc1Shape => p.continuous(0),
        Target::Osc2Shape => p.continuous(1),
        Target::Osc3Shape => p.continuous(2),
        Target::Osc2Frequency => !p.timed_now(1),
        Target::Osc3Frequency => !p.timed_now(2),
        Target::LfoShape(n) => p.lfo(n).shape_mode.value() == crate::params::WaveMode::Continuous,
        Target::LfoRate(n) => !p.lfo(n).timed.value(),
        // (The standalone's alone: kept in a plug-in's patch, never applied.)
        Target::Tempo => p.is_standalone(),
        _ => true,
    }
}

/// `modulation.list`: every source with what it modulates (each knob's id, label, depth in %,
/// and whether it takes the LFO's shared copy with MONO off), its EDGES, MONO, POLARITY, REVERSE
/// and ON; the most a
/// source takes; and the knobs that can take one now.
pub fn list(p: &Ca72Params) -> Value {
    // (The pins first: they say which pin targets can take a source now.)
    let pins = pins_now();
    let table = values::table(p);
    let sources: Vec<Value> = Source::all()
        .map(|s| {
            let knobs: Vec<Value> = p
                .lfo_patch
                .assignments(s)
                .into_iter()
                .flatten()
                .map(|a| {
                    let pin = a
                        .target
                        .fx()
                        .and_then(|k| pins.iter().find(|x| x.target == k));
                    let label = match pin {
                        Some(x) => format!("{}: {}", x.effect, x.name),
                        None => values::find(&table, a.target.id())
                            .map_or(a.target.short(), |e| e.label)
                            .to_owned(),
                    };
                    json!({
                        "target": a.target.id(),
                        "label": label,
                        "depth": (f64::from(a.depth) * 1000.0).round() / 10.0,
                        "shared": lfo::takes_shared(p, s, a.target),
                        "inert": !live(p, a.target),
                    })
                })
                .collect();
            let mut v = json!({"source": s.id(), "label": label(s), "knobs": knobs});
            if let Some(n) = s.lfo() {
                v["edges"] = json!(match p.lfo_patch.edges(n) {
                    Edges::Soft => "SOFT",
                    Edges::Sharp => "SHARP",
                });
                v["mono"] = json!(p.lfo(n).mono.value());
                // (R-LFO's corrections: BI or UNI, REVERSE, ON; the parameters
                // `lfoN_polarity`, `lfoN_reverse` and `lfoN_on` set them.)
                let keys = p.lfo_keys(n);
                v["polarity"] = json!(match keys.polarity.value() {
                    lfo::Polarity::Bi => "BI",
                    lfo::Polarity::Uni => "UNI",
                });
                v["reverse"] = json!(keys.reverse.value());
                v["on"] = json!(keys.on.value());
            }
            if let Some(n) = s.macro_n() {
                v["name"] = json!(p.lfo_patch.name(n));
                v["value"] = json!(f64::from(p.macro_param(n).value()).round());
            }
            v
        })
        .collect();
    let mut v = json!({"sources": sources, "most": lfo::MOST, "targets": targets_now(p)});
    if crate::fx_view::installed().is_some() {
        v["pins"] = json!(pins.iter().map(pin_json).collect::<Vec<_>>());
    }
    v
}

/// The effects' pins now (none without the standalone's effects), the targets that can take a
/// source published with them.
fn pins_now() -> Vec<crate::fx_view::PinInfo> {
    crate::fx_view::installed()
        .and_then(|e| e.pins())
        .unwrap_or_default()
}

fn pin_json(x: &crate::fx_view::PinInfo) -> Value {
    json!({
        "target": Target::Fx(x.target).id(),
        "slot": x.slot + 1,
        "effect": x.effect,
        "parameter": x.id,
        "name": x.name,
        "value": (x.value * 1000.0).round() / 10.0,
        "live": x.live,
        "missing": x.missing,
    })
}

/// The standalone's effects, or why there are none.
fn effects() -> Result<std::sync::Arc<dyn crate::fx_view::Effects>, ApiError> {
    crate::fx_view::installed()
        .ok_or_else(|| ApiError::refused("effects are the standalone application's alone"))
}

/// The effect's slot an agent names (1 to 4).
fn slot_of(v: Option<&Value>) -> Result<usize, ApiError> {
    match v.and_then(Value::as_u64) {
        Some(n @ 1..=4) => Ok(n as usize - 1),
        _ => Err(ApiError::params("`slot` is the effect's place, 1 to 4")),
    }
}

/// `effects.parameters`: the parameters of the effect in `slot` that a player may pin, in
/// name order (those holding every word of `find`, if given), each with its ID, names, units,
/// steps, value in % of its travel, the plug-in's own text for it, and its pin target if
/// pinned (R-FX-PARAMS).
pub fn effect_params(_p: &Ca72Params, args: &Value) -> Reply {
    let fx = effects()?;
    let slot = slot_of(arg(args, "slot"))?;
    let rows = fx.params(slot).map_err(ApiError::refused)?;
    let find = arg(args, "find").and_then(Value::as_str).unwrap_or("");
    let shown = crate::fx_view::search_params(&rows, find, false);
    let texts = fx.param_text(
        slot,
        &shown
            .iter()
            .map(|&i| (rows[i].id, rows[i].plain))
            .collect::<Vec<_>>(),
    );
    let list: Vec<Value> = shown
        .iter()
        .zip(texts)
        .map(|(&i, text)| {
            let r = &rows[i];
            json!({
                "parameter": r.id,
                "name": r.name,
                "short": r.short,
                "units": r.units,
                "steps": r.steps,
                "value": (r.value * 1000.0).round() / 10.0,
                "text": text,
                "pinned": r.pinned.map(|t| Target::Fx(t).id()),
            })
        })
        .collect();
    Ok(
        json!({"slot": slot + 1, "parameters": list, "pins": pins_now().iter().map(pin_json).collect::<Vec<_>>()}),
    )
}

/// The parameter of `rows` an agent names: its ID, or its name (any case).
fn param_of(rows: &[crate::fx_view::ParamRow], v: Option<&Value>) -> Result<u32, ApiError> {
    let found = match v {
        Some(Value::Number(n)) => n
            .as_u64()
            .and_then(|n| rows.iter().find(|r| u64::from(r.id) == n)),
        Some(Value::String(s)) => rows.iter().find(|r| {
            r.name.eq_ignore_ascii_case(s.trim()) || r.short.eq_ignore_ascii_case(s.trim())
        }),
        _ => return Err(ApiError::params("`parameter` is its ID or its name")),
    };
    found
        .map(|r| r.id)
        .ok_or_else(|| ApiError::refused("the effect has no such parameter a player may pin"))
}

/// `effects.pin`: the parameter `parameter` of the effect in `slot` pinned (at most four an
/// effect): its target (`fx1` to `fx16`), which the modulation tools then take.
pub fn pin(_p: &Ca72Params, args: &Value) -> Reply {
    let fx = effects()?;
    let slot = slot_of(arg(args, "slot"))?;
    let rows = fx.params(slot).map_err(ApiError::refused)?;
    let id = param_of(&rows, arg(args, "parameter"))?;
    let t = fx.pin(slot, id).map_err(ApiError::refused)?;
    Ok(
        json!({"target": Target::Fx(t).id(), "pins": pins_now().iter().map(pin_json).collect::<Vec<_>>()}),
    )
}

/// `effects.unpin`: pin `target` (`fx1` to `fx16`) unpinned, what moved it let go of.
pub fn unpin(_p: &Ca72Params, args: &Value) -> Reply {
    let fx = effects()?;
    let t = arg(args, "target")
        .and_then(Value::as_str)
        .and_then(|s| Target::of_id(s.trim()))
        .and_then(Target::fx)
        .ok_or_else(|| ApiError::params("`target` is a pin's, fx1 to fx16"))?;
    fx.unpin(t).map_err(ApiError::refused)?;
    Ok(json!({"pins": pins_now().iter().map(pin_json).collect::<Vec<_>>()}))
}

/// `modulation.set`: `source` modulating `target` by `depth` (-100 to 100 % of the knob's
/// travel; added, or its depth changed), and/or an LFO's `edges` (SOFT or SHARP), and/or a
/// macro's `name` (seven characters; "" for none).
pub fn set(p: &Ca72Params, args: &Value) -> Reply {
    let source = source_of(p, arg(args, "source"))?;
    let name = match arg(args, "name") {
        None => None,
        Some(Value::String(n)) if source.macro_n().is_some() => Some(lfo::name_within(n)),
        Some(Value::String(_)) => return Err(ApiError::params("`name` is a macro's")),
        Some(_) => return Err(ApiError::params("`name` must be text")),
    };
    let edges = match arg(args, "edges") {
        None => None,
        Some(Value::String(e)) if e.eq_ignore_ascii_case("soft") => Some(Edges::Soft),
        Some(Value::String(e)) if e.eq_ignore_ascii_case("sharp") => Some(Edges::Sharp),
        Some(_) => return Err(ApiError::params("`edges` must be SOFT or SHARP")),
    };
    if edges.is_some() && source.lfo().is_none() {
        return Err(ApiError::params("`edges` is an LFO's"));
    }
    let target = arg(args, "target");
    if target.is_none() && edges.is_none() && name.is_none() {
        return Err(ApiError::params(
            "give `target` and `depth`, or an LFO's `edges`, or a macro's `name`",
        ));
    }
    // (Everything checked before anything is changed.)
    let assignment = match target {
        None => None,
        Some(t) => {
            let t = target_of(p, Some(t))?;
            let depth = match arg(args, "depth") {
                Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
                None if p.lfo_patch.depth(source, t).is_none() => {
                    f64::from(lfo::DEFAULT_DEPTH) * 100.0
                }
                None => {
                    return Err(ApiError::params("`depth` is required to change one"));
                }
                Some(_) => return Err(ApiError::params("`depth` must be a number of %")),
            };
            if !(-100.0..=100.0).contains(&depth) {
                return Err(
                    ApiError::params("`depth` is -100 to 100 (% of the knob's travel)")
                        .with_data(json!({"min": -100, "max": 100})),
                );
            }
            Some((t, depth))
        }
    };
    if let Some((t, depth)) = assignment {
        match p.lfo_patch.assign(source, t, (depth / 100.0) as f32) {
            Ok(()) => {}
            Err(Refused::Full) => {
                return Err(ApiError::refused(format!(
                    "{} already modulates {} knobs: remove one first",
                    named(p, source),
                    lfo::MOST
                )));
            }
            Err(Refused::NoSource) => return Err(ApiError::params("no such source")),
        }
    }
    if let (Some(e), Some(n)) = (edges, source.lfo()) {
        p.lfo_patch.set_edges(n, e);
    }
    if let (Some(name), Some(n)) = (name, source.macro_n()) {
        p.lfo_patch.set_name(n, &name);
    }
    Ok(list(p))
}

/// `modulation.remove`: `source` no longer modulating `target`, or anything (no `target`).
pub fn remove(p: &Ca72Params, args: &Value) -> Reply {
    let source = source_of(p, arg(args, "source"))?;
    match arg(args, "target") {
        None => {
            for a in p.lfo_patch.assignments(source).into_iter().flatten() {
                p.lfo_patch.remove(source, a.target);
            }
        }
        Some(Value::String(key)) => {
            let table = values::table(p);
            let id = values::find(&table, key).map_or(key.trim(), |e| e.id);
            let wave = ["osc1_waveform", "osc2_waveform", "osc3_waveform"];
            let lfo_wave = [
                "lfo1_waveform",
                "lfo2_waveform",
                "lfo3_waveform",
                "lfo4_waveform",
            ];
            let t = wave
                .iter()
                .position(|w| *w == id)
                .map(|n| [Target::Osc1Shape, Target::Osc2Shape, Target::Osc3Shape][n])
                .or_else(|| lfo_wave.iter().position(|w| *w == id).map(Target::LfoShape))
                .or_else(|| Target::of_id(id));
            if !t.is_some_and(|t| p.lfo_patch.remove(source, t)) {
                return Err(ApiError::refused(format!(
                    "{} does not modulate {key}",
                    named(p, source)
                )));
            }
        }
        Some(_) => return Err(ApiError::params("`target` must be a knob's id or label")),
    }
    Ok(list(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An agent lists, adds, changes and removes what the LFOs modulate as the page does: a
    /// knob by its id or label, the default depth where none is given, a depth out of range
    /// or a knob that cannot take one refused with nothing changed, EDGES set, at most eight.
    #[test]
    fn agents_assign_and_remove_as_the_page_does() {
        let p = Ca72Params::default();
        let empty = list(&p);
        assert_eq!(
            empty["sources"].as_array().map(Vec::len),
            Some(lfo::SOURCES)
        );
        // (R-LFO's corrections: each LFO's POLARITY, REVERSE and ON, as parameters too.)
        let first = &empty["sources"][0];
        assert_eq!(
            (&first["polarity"], &first["reverse"], &first["on"]),
            (&json!("BI"), &json!(false), &json!(true))
        );
        assert!(
            empty["sources"][lfo::LFOS].get("polarity").is_none(),
            "a macro's: none"
        );
        let table = values::table(&p);
        for id in ["lfo1_polarity", "lfo2_reverse", "lfo4_on"] {
            assert!(values::find(&table, id).is_some(), "{id}");
        }
        assert!(
            empty["targets"]
                .as_array()
                .unwrap()
                .contains(&json!("cutoff"))
        );
        assert!(
            !empty["targets"]
                .as_array()
                .unwrap()
                .contains(&json!("osc1_waveform"))
        );
        let r = set(
            &p,
            &json!({"source": "lfo1", "target": "cutoff", "depth": 40}),
        )
        .unwrap();
        assert_eq!(r["sources"][0]["knobs"][0]["depth"], json!(40.0));
        set(&p, &json!({"source": 2, "target": "MAIN OUTPUT VOLUME"})).unwrap();
        assert_eq!(
            p.lfo_patch.depth(Source::Lfo(1), Target::Volume),
            Some(0.25)
        );
        assert!(
            set(
                &p,
                &json!({"source": "LFO 1", "target": "cutoff", "depth": 140})
            )
            .is_err()
        );
        assert_eq!(p.lfo_patch.depth(Source::Lfo(0), Target::Cutoff), Some(0.4));
        assert!(
            set(
                &p,
                &json!({"source": "lfo1", "target": "osc1_waveform", "depth": 10})
            )
            .is_err()
        );
        assert!(
            set(
                &p,
                &json!({"source": "lfo1", "target": "osc1_range", "depth": 10})
            )
            .is_err()
        );
        assert!(set(&p, &json!({"source": 5, "target": "cutoff"})).is_err());
        set(&p, &json!({"source": "lfo3", "edges": "SHARP"})).unwrap();
        assert_eq!(p.lfo_patch.edges(2), Edges::Sharp);
        for t in [
            "tune",
            "glide",
            "mod_mix",
            "osc1_volume",
            "osc2_volume",
            "emphasis",
            "drive",
        ] {
            set(&p, &json!({"source": "lfo1", "target": t, "depth": -5})).unwrap();
        }
        let full = set(
            &p,
            &json!({"source": "lfo1", "target": "inner", "depth": 5}),
        );
        assert_eq!(full.map_err(|e| e.code), Err(super::super::codes::REFUSED));
        remove(&p, &json!({"source": "lfo1", "target": "CUTOFF FREQUENCY"})).unwrap();
        assert_eq!(p.lfo_patch.count(Source::Lfo(0)), 7);
        assert!(remove(&p, &json!({"source": "lfo1", "target": "cutoff"})).is_err());
        remove(&p, &json!({"source": "lfo1"})).unwrap();
        assert_eq!(p.lfo_patch.count(Source::Lfo(0)), 0);
    }

    /// The macros as the agents name them: by id, label or typed name; a name set (capitals, at
    /// most seven) and listed with the macro's value; a macro's knob, an LFO's own knob or a
    /// macro not a target; `edges` an LFO's and `name` a macro's alone.
    /// The LFOs' own knobs, by an agent as by the page (R-LFO's addendum): an LFO's RATE by its
    /// ID or label (its own too), its LEVEL by "LFO 2 LEVEL" apart from the strip's `level`, its
    /// WAVEFORM only while CONTINUOUS; each listed by its label, the shared copy's marked; the
    /// targets now listed; and removed.
    #[test]
    fn agents_move_the_lfos_own_knobs() {
        let p = Ca72Params::default();
        let at = |s, t| p.lfo_patch.depth(s, t);
        set(
            &p,
            &json!({"source": "lfo2", "target": "lfo1_rate", "depth": 30}),
        )
        .unwrap();
        assert_eq!(at(Source::Lfo(1), Target::LfoRate(0)), Some(0.3));
        set(
            &p,
            &json!({"source": "lfo1", "target": "LFO 1 RATE", "depth": -20}),
        )
        .unwrap();
        assert_eq!(
            at(Source::Lfo(0), Target::LfoRate(0)),
            Some(-0.2),
            "its own"
        );
        set(
            &p,
            &json!({"source": "macro1", "target": "LFO 2 LEVEL", "depth": -50}),
        )
        .unwrap();
        assert_eq!(at(Source::Macro(0), Target::LfoLevel(1)), Some(-0.5));
        set(&p, &json!({"source": "lfo3", "target": "level"})).unwrap();
        assert_eq!(at(Source::Lfo(2), Target::Level), Some(0.25), "the strip's");
        assert_eq!(at(Source::Lfo(2), Target::LfoLevel(2)), None);
        assert!(set(&p, &json!({"source": "lfo1", "target": "lfo3_waveform"})).is_err());
        let r = list(&p);
        let targets = r["targets"].as_array().unwrap();
        for id in ["lfo1_rate", "lfo2_phase", "lfo3_rise", "lfo4_level"] {
            assert!(targets.contains(&json!(id)), "{id}");
        }
        assert!(!targets.contains(&json!("lfo1_waveform")));
        let knob = &r["sources"][1]["knobs"][0];
        assert_eq!(
            (&knob["target"], &knob["label"], &knob["shared"]),
            (&json!("lfo1_rate"), &json!("LFO 1 RATE"), &json!(false))
        );
        assert_eq!(
            r["sources"][lfo::LFOS]["knobs"][0]["label"],
            json!("LFO 2 LEVEL")
        );
        remove(&p, &json!({"source": "macro1", "target": "lfo2_level"})).unwrap();
        assert_eq!(at(Source::Macro(0), Target::LfoLevel(1)), None);
        assert!(remove(&p, &json!({"source": "macro1", "target": "lfo2_level"})).is_err());
    }

    /// The standalone's TEMPO (decisions.md R-LFO): listed among the targets and taken there
    /// alone; in a plug-in refused with its reason, and one a session brought kept, marked inert.
    #[test]
    fn agents_move_the_tempo_in_the_standalone_alone() {
        let p = Ca72Params::default();
        let tempo = json!("tempo");
        assert!(!list(&p)["targets"].as_array().unwrap().contains(&tempo));
        let refused = set(
            &p,
            &json!({"source": "lfo1", "target": "tempo", "depth": 40}),
        );
        assert!(format!("{:?}", refused.unwrap_err()).contains("standalone only"));
        p.set_standalone(true);
        assert!(list(&p)["targets"].as_array().unwrap().contains(&tempo));
        set(
            &p,
            &json!({"source": "macro2", "target": "TEMPO", "depth": -30}),
        )
        .unwrap();
        let r = set(
            &p,
            &json!({"source": "lfo1", "target": "tempo", "depth": 40}),
        )
        .unwrap();
        assert_eq!(p.lfo_patch.depth(Source::Lfo(0), Target::Tempo), Some(0.4));
        assert_eq!(
            p.lfo_patch.depth(Source::Macro(1), Target::Tempo),
            Some(-0.3)
        );
        let knob = &r["sources"][0]["knobs"][0];
        assert_eq!(
            (
                &knob["target"],
                &knob["label"],
                &knob["shared"],
                &knob["inert"]
            ),
            (&tempo, &json!("TEMPO"), &json!(false), &json!(false))
        );
        p.set_standalone(false);
        assert_eq!(list(&p)["sources"][0]["knobs"][0]["inert"], json!(true));
        remove(&p, &json!({"source": "lfo1", "target": "tempo"})).unwrap();
        assert_eq!(p.lfo_patch.depth(Source::Lfo(0), Target::Tempo), None);
    }

    #[test]
    fn agents_name_and_assign_the_macros() {
        let p = Ca72Params::default();
        let r = set(&p, &json!({"source": "MACRO 2", "name": "growl!!!"})).unwrap();
        assert_eq!(p.lfo_patch.name(1), "GROWL!!");
        let m = &r["sources"][lfo::LFOS + 1];
        assert_eq!(m["source"], json!("macro2"));
        assert_eq!(m["name"], json!("GROWL!!"));
        assert_eq!(m["value"], json!(0.0));
        set(
            &p,
            &json!({"source": "growl!!", "target": "drive", "depth": 50}),
        )
        .unwrap();
        assert_eq!(
            p.lfo_patch.depth(Source::Macro(1), Target::Drive),
            Some(0.5)
        );
        for bad in ["macro3", "lfo1_timed"] {
            assert!(
                set(&p, &json!({"source": "macro1", "target": bad})).is_err(),
                "{bad}"
            );
        }
        assert!(set(&p, &json!({"source": "macro1", "edges": "SHARP"})).is_err());
        assert!(set(&p, &json!({"source": "lfo1", "name": "X"})).is_err());
        assert!(set(&p, &json!({"source": "macro7", "name": "X"})).is_err());
        set(&p, &json!({"source": "macro2", "name": ""})).unwrap();
        assert_eq!(p.lfo_patch.name(1), "");
        remove(&p, &json!({"source": "macro2"})).unwrap();
        assert_eq!(p.lfo_patch.count(Source::Macro(1)), 0);
    }
}
