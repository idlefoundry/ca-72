//! What the modulation sources (the four LFOs and the six macros) modulate, for the agents
//! (decisions.md R-LFO, R-MACROS, docs/mcp.md): `modulation.list`, `modulation.set` and
//! `modulation.remove`, carried out on the patch as the LFO page's and the macros' drag and
//! menus are (`lfo::Patch`, patch state: no host gesture), and the macros' typed names. The LFOs'
//! own controls and the macros' knobs are parameters (`lfoN_rate`, `macroN`), set with
//! `parameters.set`.

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
            let n = n.as_u64().ok_or_else(bad)? as usize;
            (1..=lfo::LFOS)
                .contains(&n)
                .then_some(Source::Lfo(n - 1))
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

/// The knob an agent names (an id, a panel label or a host name; an oscillator's WAVEFORM, its
/// CONTINUOUS shape), if it can take a source now, as the panel's drag allows.
fn target_of(p: &Ca72Params, v: Option<&Value>) -> Result<Target, ApiError> {
    let Some(Value::String(key)) = v else {
        return Err(ApiError::params("`target` must be a knob's id or label"));
    };
    let table = values::table(p);
    let id = values::find(&table, key)
        .map(|e| e.id)
        .unwrap_or(key.trim());
    lfo::target_of(p, id).ok_or_else(|| {
        let why = match Target::of_id(id) {
            Some(Target::Osc1Shape | Target::Osc2Shape | Target::Osc3Shape) => {
                format!("{key} takes a source only through its WAVEFORM while CONTINUOUS")
            }
            Some(_) => format!("{key} cannot take a source now (its oscillator is timed)"),
            None if id.ends_with("_waveform") && id.starts_with("osc") => {
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
    ids
}

/// Whether an assignment on `t` moves anything now (a shape's oscillator CONTINUOUS, a
/// FREQUENCY's not timed).
fn live(p: &Ca72Params, t: Target) -> bool {
    match t {
        Target::Osc1Shape => p.continuous(0),
        Target::Osc2Shape => p.continuous(1),
        Target::Osc3Shape => p.continuous(2),
        Target::Osc2Frequency => !p.timed_now(1),
        Target::Osc3Frequency => !p.timed_now(2),
        _ => true,
    }
}

/// `modulation.list`: every source with what it modulates (each knob's id, label, depth in %,
/// and whether it takes the LFO's shared copy with MONO off), its EDGES, MONO, POLARITY, REVERSE
/// and ON; the most a
/// source takes; and the knobs that can take one now.
pub fn list(p: &Ca72Params) -> Value {
    let table = values::table(p);
    let sources: Vec<Value> = Source::all()
        .map(|s| {
            let knobs: Vec<Value> = p
                .lfo_patch
                .assignments(s)
                .into_iter()
                .flatten()
                .map(|a| {
                    let mono = s.lfo().is_none_or(|n| p.lfo(n).mono.value());
                    json!({
                        "target": a.target.id(),
                        "label": values::find(&table, a.target.id()).map_or("", |e| e.label),
                        "depth": (f64::from(a.depth) * 1000.0).round() / 10.0,
                        "shared": !mono && !a.target.per_voice(),
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
    json!({"sources": sources, "most": lfo::MOST, "targets": targets_now(p)})
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
    if let Some(e) = edges {
        let Some(n) = source.lfo() else {
            return Err(ApiError::params("`edges` is an LFO's"));
        };
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
            let t = wave
                .iter()
                .position(|w| *w == id)
                .map(|n| [Target::Osc1Shape, Target::Osc2Shape, Target::Osc3Shape][n])
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
        for bad in ["macro3", "lfo1_rate"] {
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
