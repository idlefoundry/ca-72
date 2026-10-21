//! Bounded offline experiments. Reference and approved measurements are prepared once;
//! candidates remain independent snapshots. No optimiser or live-state mutations.

use super::*;
use ca72_analysis::matching::regions::{self, Limits, Measurement, Region};
use serde::Deserialize;

pub fn regions_schema() -> Value {
    json!({"type":"array","minItems":1,"maxItems":32,"description":"Named windows relative to the selected clip; total duration including overlaps at most120s. At least1024samples each. No hidden alignment. Fundamentals are supplied note/chord estimates, refined separately within±4%; chord bands can overlap.","items":{
    "type":"object","additionalProperties":false,"required":["id","start","seconds"],"properties":{
        "id":{"type":"string","minLength":1,"maxLength":64},"start":{"type":"number","minimum":0},"seconds":{"type":"number","exclusiveMinimum":0},
        "fundamentals_hz":{"type":"array","maxItems":10,"items":{"type":"number","minimum":20}},
        "modulation_hz":{"type":"number","minimum":0.1,"maximum":30,"description":"Known periodic rate for approximate AM/FM depth. Needs one isolated fundamental and3resolvedcycles; unreliable window response returns null."}
    }}})
}

pub fn limits_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"description":"Optional explicit engineering limits against approved audio. No default perceptual pass mark. Unresolved requested measurements remain null. AM/FM limits apply to regions requesting modulation_hz.","properties":{
        "envelope_rmse_db":{"type":"number","minimum":0},"envelope_p95_db":{"type":"number","minimum":0},"am_depth_db":{"type":"number","minimum":0},"fm_depth_cents":{"type":"number","minimum":0},"pitch_cents":{"type":"number","minimum":0}
    }})
}

pub fn schema(p: &crate::params::Ca72Params) -> Value {
    let mut s = render_schema(p);
    s["properties"]["reference"] = json!({"type":"string","description":"Original local WAV, decoded/analyzed once for this batch."});
    s["properties"]["reference_start"] = json!({"type":"number","minimum":0,"default":0});
    s["properties"]["approved"] = json!({"type":"string","description":"Optional approved baseline WAV, same selected duration and rate."});
    s["properties"]["approved_start"] = json!({"type":"number","minimum":0,"default":0});
    s["properties"]["regions"] = regions_schema();
    s["properties"]["preservation_limits"] = limits_schema();
    s["properties"]["candidates"] = json!({"type":"array","minItems":1,"maxItems":8,"description":"Independent variations of the common base after common values/modulations. At most120renderedseconds and12millionframes total. Every candidate validates before any output is created.","items":{
    "type":"object","required":["id"],"additionalProperties":false,"properties":{
        "id":{"type":"string","pattern":"^[a-zA-Z0-9_-]{1,40}$"},
        "values":s["properties"]["values"],"modulations":s["properties"]["modulations"]
    }}});
    s["required"] = json!([
        "effects",
        "notes",
        "reference",
        "regions",
        "candidates",
        "output_dir"
    ]);
    s
}

pub fn optional_regions(args: &Value) -> Result<Option<Vec<Region>>, ApiError> {
    arg(args, "regions")
        .map(|r| {
            serde_json::from_value(r.clone()).map_err(|e| ApiError::params(format!("regions: {e}")))
        })
        .transpose()
}

fn limits(args: &Value) -> Result<Limits, ApiError> {
    if arg(args, "approved").is_none()
        && (arg(args, "preservation_limits").is_some() || arg(args, "approved_start").is_some())
    {
        return Err(ApiError::params(
            "approved audio is required for preservation_limits/approved_start",
        ));
    }
    let limits: Limits = arg(args, "preservation_limits")
        .map(|v| {
            serde_json::from_value(v.clone())
                .map_err(|e| ApiError::params(format!("preservation_limits: {e}")))
        })
        .transpose()?
        .unwrap_or_default();
    limits.validate().map_err(ApiError::params)?;
    Ok(limits)
}

pub fn measurement_summary(m: &[Measurement]) -> Value {
    json!(m.iter().map(|m|json!({"id":m.id,"start":m.start,"seconds":m.seconds,"fundamentals_hz":m.fundamental_estimates_hz,"harmonic_anchors_hz":m.harmonic_anchors_hz,"fundamental_peak_support_db":m.fundamental_peak_support_db,"fft_bin_hz":m.fft_bin_hz,"physical_resolution_hz":m.physical_resolution_hz,"normalization":m.normalization,"harmonic_levels_db":m.harmonics.iter().map(|h|h.relative_db).collect::<Vec<_>>(),"upper_band_db":m.upper_band_db,"modulation":m.modulation,"warnings":m.warnings})).collect::<Vec<_>>())
}

fn difference_summary(d: &[regions::Difference]) -> Value {
    json!(d.iter().map(|d|json!({"id":d.id,"harmonic_rmse_db":d.harmonic_rmse_db,"spectrum_rmse_db":d.spectrum_rmse_db,"upper_band_difference_db":d.upper_band_difference_db,"fundamental_differences_cents":d.fundamental_differences_cents,"am_depth_difference_db":d.am_depth_difference_db,"fm_depth_difference_cents":d.fm_depth_difference_cents,"unexplained_peak_count":d.unexplained_peaks.len(),"strongest_unexplained_peaks":d.unexplained_peaks.iter().take(3).collect::<Vec<_>>()})).collect::<Vec<_>>())
}

fn preservation_summary(p: &regions::Preservation) -> Value {
    let maximum = |f: fn(&regions::PreservationRegion) -> Option<f64>| {
        p.regions.iter().filter_map(f).reduce(f64::max)
    };
    json!({"within_limits":p.within_limits,"candidate_gain_db":p.candidate_gain_db,"limits":p.limits,"maximum_region_envelope_rmse_db":maximum(|r|r.envelope_rmse_db),"maximum_region_envelope_p95_db":maximum(|r|r.envelope_p95_db),"maximum_am_depth_change_db":maximum(|r|r.am_depth_change_db),"maximum_fm_depth_change_cents":maximum(|r|r.fm_depth_change_cents),"outside_limit_regions":p.regions.iter().filter(|r|r.within_limits==Some(false)).map(|r|&r.id).collect::<Vec<_>>(),"unresolved_regions":p.regions.iter().filter(|r|r.requested_checks>0 && r.within_limits.is_none()).map(|r|&r.id).collect::<Vec<_>>()})
}

fn compatible(a: &Clip, b: &Clip) -> Result<(), ApiError> {
    if a.rate != b.rate || a.channels[0].len() != b.channels[0].len() {
        return Err(ApiError::params(
            "reference, candidate and approved selections need identical sample rates and frame counts",
        ));
    }
    Ok(())
}

pub(super) fn compare_regions(args: &Value, a: &Clip, b: &Clip) -> Result<Option<Value>, ApiError> {
    let limits = limits(args)?;
    let Some(specs) = optional_regions(args)? else {
        if arg(args, "approved").is_some() {
            return Err(ApiError::params(
                "approved comparison requires named regions",
            ));
        }
        return Ok(None);
    };
    limits.validate_for(&specs).map_err(ApiError::params)?;
    compatible(a, b)?;
    let reference = regions::analyze(&a.channels, a.rate, &specs).map_err(ApiError::params)?;
    let candidate = regions::analyze(&b.channels, b.rate, &specs).map_err(ApiError::params)?;
    let differences = regions::differences(&reference, &candidate).map_err(ApiError::params)?;
    let (approved, preservation) = if arg(args, "approved").is_some() {
        let c = load(args, "approved", "approved_start")?;
        compatible(a, &c)?;
        let m = regions::analyze(&c.channels, c.rate, &specs).map_err(ApiError::params)?;
        let p = regions::preservation(
            &c.channels,
            &b.channels,
            c.rate,
            &specs,
            &m,
            &candidate,
            &limits,
        )
        .map_err(ApiError::params)?;
        (Some(json!({"source":c.provenance,"regions":m})), Some(p))
    } else {
        (None, None)
    };
    Ok(Some(
        json!({"specifications":specs,"reference":reference,"candidate":candidate,"differences":differences,"approved":approved,"preservation":preservation,"interpretation":"H1–H8 uniform-weight RMS error with−60dB floor; full-spectrum bands use−80dB floor. Each window normalizes to total fundamental-band power, or50–1000Hz when no fundamentals supplied. Chord overlaps are aggregate. Peak flags are diagnostics, not automatic rejection or perceptual scores."}),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    values: Option<Value>,
    modulations: Option<Value>,
}

fn level(ch: &[Vec<f32>], rate: u32) -> Result<(Option<f64>, f64), ApiError> {
    let e = ch
        .iter()
        .flatten()
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        / (ch.len() * ch[0].len()) as f64;
    if e <= 1e-12 {
        return Err(ApiError::params("cannot level-match silent audition audio"));
    }
    let refs: Vec<_> = ch.iter().map(Vec::as_slice).collect();
    Ok((
        ca72_analysis::loudness::loudness(&refs, rate)
            .integrated
            .map(|v| {
                v + if ch.len() == 1 {
                    10.0 * 2.0f64.log10()
                } else {
                    0.0
                }
            }),
        10.0 * e.log10(),
    ))
}

fn match_gain(a: (Option<f64>, f64), b: (Option<f64>, f64)) -> (f64, &'static str) {
    match (a.0, b.0) {
        (Some(a), Some(b)) => (a - b, "LUFS, mono duplicated to stereo"),
        _ => (
            a.1 - b.1,
            "RMS, integrated LUFS unavailable for one or both clips",
        ),
    }
}

fn scaled(ch: &[Vec<f32>], gain: f64) -> Vec<Vec<f32>> {
    ch.iter()
        .map(|c| c.iter().map(|x| (f64::from(*x) * gain) as f32).collect())
        .collect()
}
fn peak(ch: &[Vec<f32>]) -> f64 {
    ch.iter()
        .flatten()
        .map(|x| f64::from(x.abs()))
        .fold(0.0, f64::max)
}
fn html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn run(args: &Value, snapshot: Snapshot, source: Option<Value>) -> Reply {
    let candidates: Vec<Candidate> = serde_json::from_value(
        arg(args, "candidates")
            .cloned()
            .ok_or_else(|| ApiError::params("candidates are required"))?,
    )
    .map_err(|e| ApiError::params(format!("candidates: {e}")))?;
    if candidates.is_empty() || candidates.len() > 8 {
        return Err(ApiError::params("supply1to8candidates"));
    }
    let (base, request) = prepare_render(args, snapshot)?;
    if request.seconds * candidates.len() as f64 > 120.0
        || request.seconds * f64::from(request.sample_rate) * candidates.len() as f64 > 12_000_000.0
    {
        return Err(ApiError::params(
            "batch exceeds120renderedseconds or12millionframes",
        ));
    }
    let specs = optional_regions(args)?.ok_or_else(|| ApiError::params("regions are required"))?;
    let limits = limits(args)?;
    limits.validate_for(&specs).map_err(ApiError::params)?;
    let reference = load(args, "reference", "reference_start")?;
    if reference.rate != request.sample_rate
        || reference.channels[0].len()
            != (request.seconds * f64::from(request.sample_rate)).round() as usize
    {
        return Err(ApiError::params(
            "render rate/duration must match the selected reference exactly",
        ));
    }
    let target =
        regions::analyze(&reference.channels, reference.rate, &specs).map_err(ApiError::params)?;
    let reference_level = level(&reference.channels, reference.rate)?;
    let approved = arg(args, "approved")
        .map(|_| load(args, "approved", "approved_start"))
        .transpose()?;
    let prior = approved
        .as_ref()
        .map(|a| {
            compatible(&reference, a)?;
            regions::analyze(&a.channels, a.rate, &specs).map_err(ApiError::params)
        })
        .transpose()?;
    let mut ids = std::collections::BTreeSet::new();
    let mut prepared = Vec::new();
    for c in &candidates {
        if c.id.is_empty()
            || c.id.len() > 40
            || !c
                .id
                .bytes()
                .all(|x| x.is_ascii_alphanumeric() || x == b'_' || x == b'-')
            || !ids.insert(&c.id)
        {
            return Err(ApiError::params(
                "candidate ids must be unique1–40ASCIIletters/digits/underscore/hyphen",
            ));
        }
        let mut s = base.clone();
        if let Some(v) = &c.values {
            s.override_values(v)?;
        }
        if let Some(v) = &c.modulations {
            s.override_modulations(v)?;
        }
        s.validate()?;
        prepared.push(s);
    }
    // Input, routing and region validation above is atomic. Runtime failures can leave an
    // incomplete folder; only experiment.json marks a completed batch, never a partial pass.
    let dir = directory(args)?;
    let mut rows = Vec::new();
    let mut audio = Vec::new();
    let approved_gain = approved
        .as_ref()
        .map(|a| {
            level(&a.channels, a.rate)
                .map(|v| 10.0f64.powf(match_gain(reference_level, v).0 / 20.0))
        })
        .transpose()?;
    let mut maximum = peak(&reference.channels).max(
        approved
            .as_ref()
            .map_or(0.0, |a| peak(&a.channels) * approved_gain.unwrap_or(1.0)),
    );
    for (c, s) in candidates.iter().zip(prepared) {
        let render_args = json!({"effects":"dry","notes":request.notes,"seconds":request.seconds,"sample_rate":request.sample_rate,"output_dir":dir.join(&c.id)});
        let r = render(&render_args, s, source.clone())?;
        let clip = load(
            &json!({"path":r["path"],"seconds":request.seconds}),
            "path",
            "start",
        )?;
        let measured =
            regions::analyze(&clip.channels, clip.rate, &specs).map_err(ApiError::params)?;
        let differences = regions::differences(&target, &measured).map_err(ApiError::params)?;
        let preservation = approved
            .as_ref()
            .zip(prior.as_ref())
            .map(|(a, m)| {
                regions::preservation(
                    &a.channels,
                    &clip.channels,
                    clip.rate,
                    &specs,
                    m,
                    &measured,
                    &limits,
                )
                .map_err(ApiError::params)
            })
            .transpose()?;
        let (match_db, matching_method) =
            match_gain(reference_level, level(&clip.channels, clip.rate)?);
        let gain = 10.0f64.powf(match_db / 20.0);
        maximum = maximum.max(peak(&clip.channels) * gain);
        json_file(&dir.join(&c.id).join("snapshot.json"), &r["snapshot"])?;
        let metrics = json!({"id":c.id,"reference":reference.provenance,"candidate":r,"region_measurements":measured,"differences":differences,"preservation":preservation,"candidate_gain_db":match_db,"level_matching":matching_method});
        json_file(&dir.join(&c.id).join("metrics.json"), &metrics)?;
        rows.push(json!({"id":c.id,"render_path":r["path"],"snapshot_path":dir.join(&c.id).join("snapshot.json"),"manifest_path":dir.join(&c.id).join("manifest.json"),"metrics_path":dir.join(&c.id).join("metrics.json"),"wav_sha256":r["wav_sha256"],"candidate_gain_db":match_db,"level_matching":matching_method,"regions":difference_summary(&differences),"preservation":preservation.as_ref().map(preservation_summary)}));
        audio.push((clip.channels, gain));
    }
    let common = 10.0f64
        .powf((-18.0 - reference_level.0.unwrap_or(reference_level.1)) / 20.0)
        .min(0.9 / maximum);
    write(
        &dir.join("reference.wav"),
        &wav(reference.rate, &scaled(&reference.channels, common))?,
    )?;
    if let Some(a) = &approved {
        write(
            &dir.join("approved.wav"),
            &wav(
                a.rate,
                &scaled(&a.channels, common * approved_gain.unwrap_or(1.0)),
            )?,
        )?;
    }
    let mut page = String::from(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>CA-72 experiment</title><style>body{font:16px/1.6 system-ui;background:#10171e;color:#edf3f6;max-width:1000px;margin:40px auto;padding:20px}article{background:#1a252e;padding:20px;margin:15px 0;border-radius:12px}audio{width:100%}a{color:#a8e1db}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:8px;border-bottom:1px solid #354650}.scroll{overflow:auto}</style><h1>CA-72 experiment</h1><p>Dry synthesis compared with the actual recording. One common listening gain after level matching; measurements are diagnostics, not perceptual scores. Choose a candidate by listening and inspecting its tradeoffs, then export and replay its snapshot.</p><article><h2>Original reference</h2><audio controls preload=metadata src=reference.wav></audio></article>",
    );
    if approved.is_some() {
        page.push_str("<article><h2>Approved baseline</h2><audio controls preload=metadata src=approved.wav></audio></article>");
    }
    for ((c, (ch, gain)), row) in candidates.iter().zip(audio).zip(&rows) {
        write(
            &dir.join(&c.id).join("audition.wav"),
            &wav(reference.rate, &scaled(&ch, common * gain))?,
        )?;
        page.push_str(&format!("<article><h2>{}</h2><audio controls preload=metadata src='{}/audition.wav'></audio><p><a href='{}/metrics.json'>Measurements</a> · <a href='{}/manifest.json'>Snapshot and render conditions</a></p><div class=scroll><table>",html(&c.id),c.id,c.id,c.id));
        if !row["preservation"].is_null() {
            let verdict = match row["preservation"]["within_limits"].as_bool() {
                Some(true) => "Within specified engineering limits",
                Some(false) => "Outside specified engineering limits",
                None => "Unresolved or no limits supplied",
            };
            page.push_str(&format!("<caption>{}</caption>", verdict));
        }
        page.push_str("<tr><th>Region</th><th>Harmonic RMS dB</th><th>Spectrum RMS dB</th><th>Upper band Δ dB</th><th>Peak flags</th><th>Strongest flag Hz</th></tr>");
        for d in row["regions"].as_array().expect("region rows") {
            page.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{:.3}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html(d["id"].as_str().unwrap_or("")),
                d["harmonic_rmse_db"]
                    .as_f64()
                    .map_or("unresolved".into(), |v| format!("{v:.3}")),
                d["spectrum_rmse_db"].as_f64().unwrap_or(0.0),
                d["upper_band_difference_db"]
                    .as_f64()
                    .map_or("unresolved".into(), |v| format!("{v:+.3}")),
                d["unexplained_peak_count"],
                d["strongest_unexplained_peaks"][0]["hz"]
                    .as_f64()
                    .map_or("—".into(), |v| format!("{v:.1}"))
            ));
        }
        page.push_str("</table></div></article>");
    }
    page.push_str("<p><a href=experiment.json>Complete experiment and source provenance</a></p><script>document.querySelectorAll('audio').forEach(a=>a.addEventListener('play',()=>document.querySelectorAll('audio').forEach(b=>{if(a!==b)b.pause()})))</script></html>");
    write(&dir.join("report.html"), page.as_bytes())?;
    let result = json!({"format":1,"analysis_format":1,"engine_source_sha256":env!("CA72_ENGINE_SOURCE_SHA256"),"request":args,"reference":reference.provenance,"reference_regions":target,"approved":approved.as_ref().map(|a|&a.provenance),"approved_regions":prior,"candidates":rows,"report_path":dir.join("report.html"),"manifest_path":dir.join("experiment.json"),"reference_decodes":1,"reference_region_analysis_passes":1,"common_listening_gain_db":20.0*common.log10(),"approved_match_gain_db":approved_gain.map(|g|20.0*g.log10()),"interpretation":"Independent dry offline candidates. Region diagnostics normalize independently of listening LUFS. No automatic winner. experiment.json is the completion marker. snapshot_path can be passed directly as snapshot_file to ca72_preset_export or ca72_render; export/reload before delivery."});
    json_file(&dir.join("experiment.json"), &result)?;
    Ok(
        json!({"manifest_path":dir.join("experiment.json"),"report_path":dir.join("report.html"),"candidates":rows,"reference_decodes":1,"reference_region_analysis_passes":1,"interpretation":result["interpretation"]}),
    )
}
