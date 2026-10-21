//! The effects’ FIND, grouping and stable recent identities (R-FX-SEARCH).

use ca72_panel::strip::fx::{FxOrder, FxRow};
use serde::{Deserialize, Serialize};

use super::Listing;

/// A scan’s index may change; this identity does not.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectId {
    pub format: String,
    pub path: std::path::PathBuf,
    pub id: String,
}

/// The chooser’s computer settings, separate from the sound and the chain.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Preferences {
    pub order: FxOrder,
    pub recent: Vec<EffectId>,
}

impl Preferences {
    pub fn chosen(&mut self, id: EffectId) {
        self.recent.retain(|r| r != &id);
        self.recent.insert(0, id);
        self.recent.truncate(6);
    }
}

/// The cached features, in this precedence: a specific effect before a general utility.
/// Both split VST3 categories and an unsplit `Fx|Reverb` are accepted. No scan is needed.
pub fn effect_type(features: &[String]) -> &'static str {
    let features: Vec<String> = features
        .iter()
        .flat_map(|s| s.split('|'))
        .map(|s| s.trim().to_lowercase())
        .collect();
    for (word, aliases) in [
        ("Reverb", &["reverb"][..]),
        ("Delay", &["delay"]),
        ("Distortion", &["distortion"]),
        (
            "Dynamics",
            &[
                "dynamics",
                "compressor",
                "limiter",
                "gate",
                "expander",
                "transient-shaper",
            ],
        ),
        ("EQ", &["eq", "equalizer", "filter"]),
        (
            "Modulation",
            &[
                "modulation",
                "chorus",
                "flanger",
                "phaser",
                "tremolo",
                "vibrato",
            ],
        ),
        ("Pitch", &["pitch shift", "pitch-shifter"]),
        ("Tools", &["tools", "utility", "analyzer", "analyser"]),
    ] {
        if features.iter().any(|s| aliases.contains(&s.as_str())) {
            return word;
        }
    }
    "Tools"
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Choice {
    Effect(usize),
    Head,
    All,
    Scan,
    Blank,
}

pub(super) fn rows(
    effects: &[Listing],
    query: &str,
    prefs: &Preferences,
    scanning: bool,
) -> Vec<(FxRow, Choice)> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut found: Vec<usize> = effects
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            let fields = [e.name.as_str(), e.vendor.as_str(), effect_type(&e.features)]
                .map(str::to_lowercase);
            let matches = words.iter().all(|w| fields.iter().any(|f| f.contains(w)));
            (matches && (prefs.order != FxOrder::Recent || prefs.recent.contains(&e.key)))
                .then_some(i)
        })
        .collect();
    let group = |i: usize| match prefs.order {
        FxOrder::Maker if effects[i].vendor.trim().is_empty() => "Unknown maker".to_owned(),
        FxOrder::Maker => effects[i].vendor.clone(),
        FxOrder::Type => effect_type(&effects[i].features).to_owned(),
        _ => String::new(),
    };
    found.sort_by_cached_key(|&i| {
        let e = &effects[i];
        (
            if prefs.order == FxOrder::Recent {
                prefs.recent.iter().position(|r| r == &e.key).unwrap_or(6)
            } else {
                0
            },
            group(i).to_lowercase(),
            e.name.to_lowercase(),
            e.vendor.to_lowercase(),
            e.format.clone(),
        )
    });
    let grouped = matches!(prefs.order, FxOrder::Maker | FxOrder::Type);
    let mut out = Vec::new();
    let mut previous = String::new();
    for (at, &i) in found.iter().enumerate() {
        let g = group(i);
        if grouped && (at == 0 || g.to_lowercase() != previous.to_lowercase()) {
            let count = found[at..]
                .iter()
                .take_while(|&&j| group(j).to_lowercase() == g.to_lowercase())
                .count();
            out.push((
                FxRow {
                    name: g.clone(),
                    count: Some(count),
                    ..FxRow::default()
                },
                Choice::Head,
            ));
            previous = g;
        }
        let e = &effects[i];
        out.push((
            FxRow {
                name: e.name.clone(),
                maker: e.vendor.clone(),
                kind: effect_type(&e.features).into(),
                format: e.format.to_uppercase(),
                ..FxRow::default()
            },
            Choice::Effect(i),
        ));
    }
    if prefs.order == FxOrder::Recent {
        out.push((FxRow::default(), Choice::Blank));
        out.push((
            FxRow {
                name: "ALL EFFECTS…".into(),
                action: true,
                ..FxRow::default()
            },
            Choice::All,
        ));
    }
    if !scanning {
        out.push((
            FxRow {
                name: "SCAN AGAIN".into(),
                action: true,
                ..FxRow::default()
            },
            Choice::Scan,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(name: &str, vendor: &str, features: &[&str], format: &str) -> Listing {
        Listing {
            name: name.into(),
            vendor: vendor.into(),
            features: features.iter().map(|s| (*s).into()).collect(),
            format: format.into(),
            key: EffectId {
                format: format.into(),
                path: name.into(),
                id: name.into(),
            },
        }
    }

    fn effects() -> Vec<Listing> {
        vec![
            listing("Tape Echo", "Small Maker", &["Fx|Delay"], "vst3"),
            listing("Hall", "Other Maker", &["audio-effect", "reverb"], "clap"),
            listing("Chorus", "Small Maker", &["chorus"], "clap"),
            listing("Hall", "Other Maker", &["Fx", "Reverb"], "vst3"),
        ]
    }

    fn indices(rows: &[(FxRow, Choice)]) -> Vec<usize> {
        rows.iter()
            .filter_map(|(_, c)| {
                if let Choice::Effect(i) = c {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn find_matches_every_word_across_name_maker_and_type_ignoring_case() {
        let effects = effects();
        for (query, expected) in [
            ("tAp", vec![0]),
            ("SMALL", vec![2, 0]),
            ("VER", vec![1, 3]),
            ("hAL mAk vEr", vec![1, 3]),
            ("ho sm", vec![2, 0]),
            ("hall delay", vec![]),
            (" \t ", vec![2, 1, 3, 0]),
            ("VST3", vec![]),
        ] {
            assert_eq!(
                indices(&rows(&effects, query, &Preferences::default(), false)),
                expected,
                "{query}"
            );
        }
    }

    #[test]
    fn grouped_heads_count_only_the_matches_and_keep_the_scan_indices() {
        let effects = effects();
        for (order, expected) in [
            (FxOrder::Maker, vec![("Other Maker", 2), ("Small Maker", 2)]),
            (
                FxOrder::Type,
                vec![("Delay", 1), ("Modulation", 1), ("Reverb", 2)],
            ),
        ] {
            let prefs = Preferences {
                order,
                ..Preferences::default()
            };
            let grouped = rows(&effects, "", &prefs, false);
            let heads: Vec<_> = grouped
                .iter()
                .filter_map(|(r, _)| r.count.map(|c| (r.name.as_str(), c)))
                .collect();
            assert_eq!(heads, expected);
            let filtered = rows(&effects, "clap", &prefs, false);
            assert!(
                indices(&filtered).is_empty(),
                "format is not a search field"
            );
            let filtered = rows(&effects, "hall", &prefs, false);
            assert_eq!(indices(&filtered), [1, 3]);
            assert_eq!(filtered[0].0.count, Some(2));
        }
    }

    #[test]
    fn recent_is_newest_first_distinguishes_formats_and_ignores_missing_effects() {
        let effects = effects();
        let mut prefs = Preferences {
            order: FxOrder::Recent,
            ..Preferences::default()
        };
        for i in [0, 1, 2, 3, 1] {
            prefs.chosen(effects[i].key.clone());
        }
        assert_eq!(indices(&rows(&effects, "", &prefs, false)), [1, 3, 2, 0]);
        assert_eq!(indices(&rows(&effects, "hall", &prefs, false)), [1, 3]);
        let removed = rows(&effects[..3], "", &prefs, true);
        assert_eq!(indices(&removed), [1, 2, 0]);
        assert_eq!(removed.last().map(|(_, c)| *c), Some(Choice::All));
        for i in 0..8 {
            prefs.chosen(EffectId {
                id: i.to_string(),
                ..EffectId::default()
            });
        }
        assert_eq!(prefs.recent.len(), 6);
        assert_eq!(
            prefs
                .recent
                .iter()
                .map(|id| id.id.as_str())
                .collect::<Vec<_>>(),
            ["7", "6", "5", "4", "3", "2"]
        );
    }

    #[test]
    fn cached_vst3_and_clap_categories_have_short_readable_types() {
        for (category, word) in [
            ("Reverb", "Reverb"),
            ("Delay", "Delay"),
            ("Distortion", "Distortion"),
            ("Dynamics", "Dynamics"),
            ("EQ", "EQ"),
            ("Filter", "EQ"),
            ("Modulation", "Modulation"),
            ("Pitch Shift", "Pitch"),
            ("Tools", "Tools"),
            ("Analyzer", "Tools"),
        ] {
            assert_eq!(effect_type(&[format!("Fx|{category}")]), word);
            assert_eq!(effect_type(&["Fx".into(), category.into()]), word);
        }
        for (feature, word) in [
            ("reverb", "Reverb"),
            ("delay", "Delay"),
            ("distortion", "Distortion"),
            ("compressor", "Dynamics"),
            ("limiter", "Dynamics"),
            ("gate", "Dynamics"),
            ("equalizer", "EQ"),
            ("filter", "EQ"),
            ("chorus", "Modulation"),
            ("flanger", "Modulation"),
            ("phaser", "Modulation"),
            ("pitch-shifter", "Pitch"),
            ("utility", "Tools"),
            ("analyzer", "Tools"),
            ("unknown", "Tools"),
        ] {
            assert_eq!(
                effect_type(&["audio-effect".into(), feature.into(), "stereo".into()]),
                word
            );
        }
        assert_eq!(effect_type(&[]), "Tools");
        assert_eq!(effect_type(&["utility".into(), "reverb".into()]), "Reverb");
    }
}
