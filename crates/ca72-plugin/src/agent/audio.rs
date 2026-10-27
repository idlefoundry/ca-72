//! Local files and the offline measurement loop (#77). Only the state capture is sent to
//! the application's window. Rendering, FFTs and file I/O run in the bridge process.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{ApiError, Reply, arg, flag_arg, text_arg};
use crate::offline::{Request, Snapshot};
use ca72_analysis::matching::{self, Features};

pub mod experiment;

const MAX_BYTES: u64 = 128 * 1024 * 1024;

fn schema(props: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":props,"required":required,"additionalProperties":false})
}

fn measurement_properties() -> Value {
    json!({
        "output_dir":{"type":"string","description":"New absolute local directory; parent must exist. Existing output is refused."},
        "seconds":{"type":"number","exclusiveMinimum":0,"maximum":60,"description":"Duration to select from each crop start; omitted selects the complete remainder, which must be at most 60 seconds."},
        "fundamental_hz":{"type":"number","exclusiveMinimum":0,"description":"Known fundamental of a held note, to measure its harmonics; otherwise estimated from the predominant channel."},
        "modulation_hz":{"type":"number","exclusiveMinimum":0,"description":"Expected spacing of modulation sidebands around each harmonic. Use a held-note crop long enough to resolve them."},
        "regions":experiment::regions_schema(),
        "include_image":{"type":"boolean","default":true}
    })
}

pub fn analyze_schema(_: &crate::params::Ca72Params) -> Value {
    let mut p = measurement_properties();
    p["path"] = json!({"type":"string","description":"Absolute path to a mono/stereo PCM or float WAV, at most 128 MiB."});
    p["start"] = json!({"type":"number","minimum":0,"default":0});
    schema(p, &["path", "output_dir"])
}

pub fn compare_schema(_: &crate::params::Ca72Params) -> Value {
    let mut p = measurement_properties();
    for key in ["reference", "candidate"] {
        p[key] = json!({"type":"string","description":"Absolute local WAV path."});
    }
    for key in ["reference_start", "candidate_start"] {
        p[key] = json!({"type":"number","minimum":0,"default":0});
    }
    p["approved"] = json!({"type":"string","description":"Optional approved baseline WAV. Requires regions; compares expression against this baseline separately from matching the reference."});
    p["approved_start"] = json!({"type":"number","minimum":0,"default":0});
    p["preservation_limits"] = experiment::limits_schema();
    schema(p, &["reference", "candidate", "output_dir"])
}

pub fn render_schema(p: &crate::params::Ca72Params) -> Value {
    let values: serde_json::Map<String, Value> = super::values::table(p)
        .iter()
        .map(|e| (e.id.into(), e.schema()))
        .collect();
    let mut modulation = super::tools::modulation_set_schema(p);
    modulation["properties"]["remove"] = json!({"type":"boolean","default":false,"description":"Remove this source's target, or all of its assignments when target is omitted. Takes source and optional target only."});
    schema(
        json!({
            "output_dir":{"type":"string","description":"New absolute local directory; parent must exist. Never overwrites a previous render."},
            "effects":{"type":"string","enum":["dry"],"description":"Explicitly render the synth alone; hosted effects are excluded, external input silent."},
            "init":{"type":"boolean","const":true,"description":"Start from factory parameter defaults without a library or live app. Excludes every other sound source."},
            "snapshot":{"type":"object","description":"Exact snapshot returned by ca72_snapshot or ca72_render; choose only one sound source."},
            "snapshot_file":{"type":"string","description":"Absolute path to an exact snapshot.json returned by ca72_experiment, at most1MiB. Same engine fingerprint required. Excludes snapshot/preset/preset_file."},
            "preset":{"type":"string","description":"Library preset to render without loading it into the live synth; excludes snapshot. With neither, captures the selected live instance."},
            "preset_file":{"type":"string","description":"Absolute path to the exact user-preset TOML to reload, up to 1 MiB. Excludes snapshot and preset. Does not import or change the player's library. Recorded render conditions from ca72_preset_export can be supplied again."},
            "notes":{"type":"array","minItems":1,"maxItems":4096,"items":{
                "type":"object","properties":{"note":{"type":"integer","minimum":0,"maximum":127},"start":{"type":"number","minimum":0},"duration":{"type":"number","exclusiveMinimum":0}},"required":["note","start","duration"],"additionalProperties":false}},
            "seconds":{"type":"number","minimum":0.05,"maximum":60,"default":3,"description":"Total rendered time, including release tail. Every note must end within it."},
            "sample_rate":{"type":"integer","minimum":8000,"maximum":192000,"default":48000},
            "seed":{"type":"integer","minimum":0,"description":"Random seed for repeatable noise, component variations and arpeggiator randomness; default is captured seed, or 72 for a library preset."},
            "tempo":{"type":"number","minimum":30,"maximum":300,"description":"BPM before modulation; defaults to snapshot tempo (120 for a library preset)."},
            "frequency_dials":{"type":"string","enum":["SEMITONES","HARDWARE"],"description":"Frequency modulation travel preference; defaults to snapshot law. A preset file does not store this player preference."},
            "values":{"type":"object","properties":values,"additionalProperties":false,"description":"Candidate overrides in ca72_set's panel units, including all four LFOs' mode (Env for one cycle), rate, phase, rise, polarity, reverse, mono and level. Applied before modulations; only this offline copy is changed. Each LFO Retrig resets phase/RISE on every note independently of main retrigger; shared Env follows phrase/main-envelope triggers. MONO-off mixer/filter routes can be per voice; main volume/output remain shared."},
            "modulations":{"type":"array","maxItems":128,"items":modulation,"description":"Routing edits in order, using ca72_modulation_set's source/target/depth/edges/name, or remove:true. Configure LFO controls in values. Allows secondary envelopes (Env), vibrato and macro routing with no app. Existing routing is retained unless edited; a rejected batch changes nothing. Effects pins are unavailable in a dry render."}
        }),
        &["output_dir", "effects", "notes"],
    )
}

pub fn export_schema(_: &crate::params::Ca72Params) -> Value {
    schema(
        json!({
            "snapshot":{"type":"object","description":"Exact candidate snapshot; choose snapshot or snapshot_file."},
            "snapshot_file":{"type":"string","description":"Absolute path to an exact experiment snapshot.json, at most1MiB. Avoids copying large state through the agent context."},
            "name":{"type":"string","description":"Preset name, following the normal library naming rules."},
            "tags":{"type":"array","items":{"type":"string"}},
            "output_dir":{"type":"string","description":"New absolute local directory; parent must exist. Never overwrites or installs into the player's library."}
        }),
        &["name", "output_dir"],
    )
}

pub fn preset_file(args: &Value) -> Result<(Snapshot, Value), ApiError> {
    let path = path(args, "preset_file")?;
    let mut bytes = Vec::new();
    std::fs::File::open(&path)
        .map_err(io_error)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > 1024 * 1024 {
        return Err(ApiError::params("preset file exceeds 1 MiB"));
    }
    let text = std::str::from_utf8(&bytes).map_err(io_error)?;
    let sound = crate::library::parse_user_sound(text).map_err(ApiError::params)?;
    Ok((
        Snapshot::sound_at_quality(&sound, arg(args, "values").and_then(|v| v.get("quality")))?,
        json!({"path":path,"sha256":sha(&bytes),"name":sound.name}),
    ))
}

pub fn snapshot_file(args: &Value) -> Result<(Snapshot, Value), ApiError> {
    let path = path(args, "snapshot_file")?;
    let mut bytes = Vec::new();
    std::fs::File::open(&path)
        .map_err(io_error)?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() > 1024 * 1024 {
        return Err(ApiError::params("snapshot file exceeds1MiB"));
    }
    let snapshot: Snapshot = serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::params(format!("snapshot_file: {e}")))?;
    snapshot.validate()?;
    Ok((
        snapshot,
        json!({"kind":"snapshot_file","path":path,"sha256":sha(&bytes)}),
    ))
}

pub fn export_preset(args: &Value) -> Reply {
    let snapshot: Snapshot = match (arg(args, "snapshot"), arg(args, "snapshot_file")) {
        (Some(v), None) => serde_json::from_value(v.clone())
            .map_err(|e| ApiError::params(format!("snapshot: {e}")))?,
        (None, Some(_)) => snapshot_file(args)?.0,
        _ => {
            return Err(ApiError::params(
                "choose exactly one of snapshot or snapshot_file",
            ));
        }
    };
    let p = snapshot.restore_params()?;
    let tags = arg(args, "tags")
        .map(|v| {
            serde_json::from_value::<Vec<String>>(v.clone())
                .map_err(|e| ApiError::params(format!("tags: {e}")))
        })
        .transpose()?;
    let dir = directory(args)?;
    let lo = p.controls().potato;
    let entry = crate::library::Library::at(&dir)
        .save_with(
            text_arg(args, "name")?,
            crate::presets::values_now(&p),
            tags,
            Some((lo, p.drive_curve.saved().db)),
            crate::presets::names_now(&p),
            None,
        )
        .map_err(io_error)?;
    let file = entry
        .file()
        .ok_or_else(|| ApiError::refused("preset export has no file"))?;
    let table = super::values::table(&p);
    let mut kept = serde_json::Map::new();
    for id in crate::library::KEPT.iter().chain(crate::presets::PLAYERS) {
        let e = super::values::find(&table, id)
            .ok_or_else(|| ApiError::refused(format!("missing {id}")))?;
        kept.insert((*id).into(), e.read()["value"].clone());
    }
    let conditions = json!({"values":kept,"seed":snapshot.seed,"tempo":snapshot.tempo,
        "frequency_dials":if snapshot.hardware_frequency_dials {"HARDWARE"} else {"SEMITONES"}});
    let result = json!({"preset_file":file,"preset_sha256":sha(&std::fs::read(file).map_err(io_error)?),
        "name":entry.sound.name,"render_conditions":conditions,"manifest_path":dir.join("manifest.json"),
        "processing":"export only; player's library and live instrument unchanged; hosted effects excluded",
        "serialization":"normal user preset format; values rounded to four decimal places"});
    json_file(
        &dir.join("manifest.json"),
        &json!({"format":1,"snapshot":snapshot,"export":result}),
    )?;
    Ok(result)
}

pub fn number(args: &Value, key: &str, default: f64) -> Result<f64, ApiError> {
    match arg(args, key) {
        None => Ok(default),
        Some(v) => v
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| ApiError::params(format!("{key} must be a finite number"))),
    }
}

fn optional_number(args: &Value, key: &str) -> Result<Option<f64>, ApiError> {
    arg(args, key).map(|_| number(args, key, 0.0)).transpose()
}

fn path(args: &Value, key: &str) -> Result<PathBuf, ApiError> {
    let path = PathBuf::from(text_arg(args, key)?);
    if !path.is_absolute() {
        return Err(ApiError::params(format!(
            "{key} must be an absolute local path"
        )));
    }
    Ok(path)
}

fn io_error(e: impl std::fmt::Display) -> ApiError {
    ApiError::refused(format!("local audio file: {e}"))
}

fn directory(args: &Value) -> Result<PathBuf, ApiError> {
    let dir = path(args, "output_dir")?;
    // A new directory every time. Existing references and previous candidates are never
    // overwritten; manifest.json/metrics.json is written last as the completion marker.
    std::fs::create_dir(&dir).map_err(|e| io_error(format!("{}: {e}", dir.display())))?;
    Ok(dir)
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), ApiError> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    f.write_all(bytes).map_err(io_error)
}

fn json_file(path: &Path, value: &Value) -> Result<(), ApiError> {
    write(path, &serde_json::to_vec_pretty(value).map_err(io_error)?)
}

pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Strictly check a user-supplied RIFF container before the general-purpose WAV decoder:
/// it must not silently pad a truncated chunk, discard a partial frame, or allocate from
/// a bogus channel count. Files remain on this computer and are never uploaded.
fn strict_wav(bytes: &[u8]) -> Result<(u32, Vec<Vec<f32>>), ApiError> {
    let bad = || {
        ApiError::params(
            "invalid or truncated WAV; use a complete mono/stereo PCM or IEEE-float WAV",
        )
    };
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(bad());
    }
    let u16_at = |at: usize| {
        bytes
            .get(at..at + 2)
            .and_then(|b| <[u8; 2]>::try_from(b).ok())
            .map(u16::from_le_bytes)
    };
    let u32_at = |at: usize| {
        bytes
            .get(at..at + 4)
            .and_then(|b| <[u8; 4]>::try_from(b).ok())
            .map(u32::from_le_bytes)
    };
    let end = (u32_at(4).ok_or_else(bad)? as usize)
        .checked_add(8)
        .ok_or_else(bad)?;
    if end != bytes.len() {
        return Err(bad());
    }
    let (mut at, mut format, mut data) = (12usize, None, None);
    while at < end {
        let size = u32_at(at + 4).ok_or_else(bad)? as usize;
        let body = at.checked_add(8).ok_or_else(bad)?;
        let next = body.checked_add(size).ok_or_else(bad)?;
        if next > end {
            return Err(bad());
        }
        match bytes.get(at..at + 4) {
            Some(b"fmt ") => {
                if format.is_some() || size < 16 {
                    return Err(bad());
                }
                let ch = u16_at(body + 2).ok_or_else(bad)?;
                let rate = u32_at(body + 4).ok_or_else(bad)?;
                let bits = u16_at(body + 14).ok_or_else(bad)?;
                let tag = u16_at(body).ok_or_else(bad)?;
                if !(1..=2).contains(&ch)
                    || !(8000..=192000).contains(&rate)
                    || ![8, 16, 24, 32, 64].contains(&bits)
                    || (tag == 0xfffe && size < 40)
                {
                    return Err(bad());
                }
                let align = usize::from(ch) * usize::from(bits / 8);
                if usize::from(u16_at(body + 12).ok_or_else(bad)?) != align {
                    return Err(bad());
                }
                format = Some(align);
            }
            Some(b"data") if data.replace(size).is_some() => return Err(bad()),
            _ => {}
        }
        at = next.checked_add(size & 1).ok_or_else(bad)?;
        if at > end {
            return Err(bad());
        }
    }
    let align = format.ok_or_else(bad)?;
    if !data.is_some_and(|n| n > 0 && n % align == 0) {
        return Err(bad());
    }
    let (rate, channels) = ca72_analysis::wav::read(bytes).map_err(ApiError::params)?;
    if channels.iter().flatten().any(|x| !x.is_finite()) {
        return Err(ApiError::params("WAV contains non-finite audio"));
    }
    Ok((rate, channels))
}

#[derive(Debug)]
struct Clip {
    rate: u32,
    channels: Vec<Vec<f32>>,
    provenance: Value,
}

fn load(args: &Value, key: &str, start_key: &str) -> Result<Clip, ApiError> {
    let path = path(args, key)?;
    let f = std::fs::File::open(&path).map_err(io_error)?;
    if f.metadata().map_err(io_error)?.len() > MAX_BYTES {
        return Err(ApiError::params(
            "WAV exceeds 128 MiB; supply a shorter local excerpt",
        ));
    }
    let mut bytes = Vec::new();
    f.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ApiError::params("WAV exceeds 128 MiB"));
    }
    let hash = sha(&bytes);
    let (rate, channels) = strict_wav(&bytes)?;
    let start = number(args, start_key, 0.0)?;
    if start < 0.0 {
        return Err(ApiError::params("crop start cannot be negative"));
    }
    let first = (start * rate as f64).round() as usize;
    let seconds = number(
        args,
        "seconds",
        channels[0].len().saturating_sub(first) as f64 / rate as f64,
    )?;
    if !(0.0..=matching::MAX_SECONDS).contains(&seconds) || seconds == 0.0 {
        return Err(ApiError::params(
            "select a positive excerpt of at most 60 seconds with start and seconds",
        ));
    }
    let frames = (seconds * rate as f64).round() as usize;
    let end = first
        .checked_add(frames)
        .filter(|n| *n <= channels[0].len())
        .ok_or_else(|| {
            ApiError::params(
                "the requested crop extends past the WAV; no padding or truncation is applied",
            )
        })?;
    let channels: Vec<Vec<f32>> = channels
        .into_iter()
        .map(|c| c[first..end].to_vec())
        .collect();
    matching::validate(&channels, rate).map_err(ApiError::params)?;
    Ok(Clip {
        rate,
        channels,
        provenance: json!({"path": path, "sha256": hash, "start_frame":first, "frames":frames, "sample_rate":rate}),
    })
}

pub fn wav(rate: u32, channels: &[Vec<f32>]) -> Result<Vec<u8>, ApiError> {
    matching::validate(channels, rate).map_err(ApiError::params)?;
    let ch = channels.len() as u16;
    let data = (channels[0].len() * channels.len() * 4) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&ch.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * u32::from(ch) * 4).to_le_bytes());
    b.extend_from_slice(&(ch * 4).to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for i in 0..channels[0].len() {
        for c in channels {
            b.extend_from_slice(&c[i].to_le_bytes());
        }
    }
    Ok(b)
}

fn summary(f: &Features) -> Value {
    json!({"sample_rate":f.sample_rate,"frames":f.frames,"seconds":f.seconds,"rms_dbfs":f.rms_dbfs,"peak_dbfs":f.peak_dbfs,
        "loudness":f.loudness,"pitch":f.pitch,"onsets_seconds":f.onsets_seconds,
        "harmonic_fundamental_hz":f.harmonic_fundamental_hz,"harmonic_bin_hz":f.harmonic_bin_hz,"harmonics":f.harmonics,
        "modulation_hz":f.modulation_hz,"warning":f.warning})
}

fn image(bytes: &[u8]) -> Value {
    json!({"type":"image","mimeType":"image/png","data":base64::engine::general_purpose::STANDARD.encode(bytes)})
}

fn axes(rate: u32, frames: usize, top: f64) -> Value {
    json!({"frequency_axis":"logarithmic, top to bottom", "min_hz":20,"max_hz":rate/2,
        "time_axis":"seconds, left to right; frame starts", "first_frame_start_seconds":0,
        "last_frame_start_seconds":frames.saturating_sub(4096) as f64 / rate as f64,
        "window_samples":4096,"colour_min_db":top-80.0,"colour_max_db":top})
}

pub fn analyze(args: &Value) -> Reply {
    let include_image = flag_arg(args, "include_image", true)?;
    let clip = load(args, "path", "start")?;
    let f = matching::features(
        &clip.channels,
        clip.rate,
        optional_number(args, "fundamental_hz")?,
        optional_number(args, "modulation_hz")?,
    )
    .map_err(ApiError::params)?;
    let regions = experiment::optional_regions(args)?;
    let region_measurements = regions
        .as_ref()
        .map(|r| matching::regions::analyze(&clip.channels, clip.rate, r).map_err(ApiError::params))
        .transpose()?;
    let (png, top) =
        matching::spectrogram(&clip.channels, clip.rate, 0.0, None).map_err(io_error)?;
    let dir = directory(args)?;
    write(&dir.join("spectrogram.png"), &png)?;
    let metrics = json!({"source":clip.provenance,"features":f,"regions":region_measurements,"spectrogram_axes":axes(clip.rate,clip.channels[0].len(),top),"processing":"local; no upload or source separation"});
    json_file(&dir.join("metrics.json"), &metrics)?;
    let mut result = json!({"metrics_path":dir.join("metrics.json"),"spectrogram_path":dir.join("spectrogram.png"),"source":clip.provenance,"features":summary(&f),"spectrogram_axes":metrics["spectrogram_axes"]});
    if let Some(regions) = &region_measurements {
        result["regions"] = experiment::measurement_summary(regions);
    }
    if include_image {
        result["__mcp_images"] = json!([image(&png)]);
    }
    Ok(result)
}

pub fn compare(args: &Value) -> Reply {
    let include_image = flag_arg(args, "include_image", true)?;
    let a = load(args, "reference", "reference_start")?;
    let b = load(args, "candidate", "candidate_start")?;
    if a.rate != b.rate {
        return Err(ApiError::params(
            "sample rates differ; render the candidate at the reference's sample_rate",
        ));
    }
    let region_comparison = experiment::compare_regions(args, &a, &b)?;
    let m = matching::compare(
        &a.channels,
        &b.channels,
        a.rate,
        optional_number(args, "fundamental_hz")?,
        optional_number(args, "modulation_hz")?,
    )
    .map_err(ApiError::params)?;
    let (pa, top) = matching::spectrogram(&a.channels, a.rate, 0.0, None).map_err(io_error)?;
    let (pb, _) = matching::spectrogram(&b.channels, b.rate, m.candidate_gain_db, Some(top))
        .map_err(io_error)?;
    let gain = 10.0f64.powf(m.candidate_gain_db / 20.0);
    let peak_a = a
        .channels
        .iter()
        .flatten()
        .map(|v| f64::from(v.abs()))
        .fold(0.0, f64::max);
    let peak_b = b
        .channels
        .iter()
        .flatten()
        .map(|v| f64::from(v.abs()) * gain)
        .fold(0.0, f64::max);
    let common = (0.9 / peak_a.max(peak_b)).min(1.0);
    let scaled = |ch: &[Vec<f32>], g: f64| {
        ch.iter()
            .map(|c| c.iter().map(|v| (f64::from(*v) * g) as f32).collect())
            .collect::<Vec<Vec<f32>>>()
    };
    let wa = wav(a.rate, &scaled(&a.channels, common))?;
    let wb = wav(b.rate, &scaled(&b.channels, common * gain))?;
    let dir = directory(args)?;
    for (name, bytes) in [
        ("reference.png", &pa),
        ("candidate.png", &pb),
        ("reference_ab.wav", &wa),
        ("candidate_ab.wav", &wb),
    ] {
        write(&dir.join(name), bytes)?;
    }
    let metrics = json!({"reference_source":a.provenance,"candidate_source":b.provenance,"comparison":m,"regions":region_comparison,
        "ab_common_gain_db":20.0*common.log10(),"spectrogram_axes":axes(a.rate,a.channels[0].len(),top),"processing":"local; no upload, time alignment, pitch shift or source separation"});
    json_file(&dir.join("metrics.json"), &metrics)?;
    let mut result = json!({"metrics_path":dir.join("metrics.json"),"reference_spectrogram":dir.join("reference.png"),"candidate_spectrogram":dir.join("candidate.png"),
        "reference_ab":dir.join("reference_ab.wav"),"candidate_ab":dir.join("candidate_ab.wav"),"candidate_gain_db":m.candidate_gain_db,"level_matching":m.level_matching,
        "resolutions":m.resolutions,"envelope_mae_db":m.envelope_mae_db,"median_pitch_difference_cents":m.median_pitch_difference_cents,"harmonic_differences_db":m.harmonic_differences_db,
        "reference":summary(&m.reference),"candidate":summary(&m.candidate),"interpretation":m.interpretation,"spectrogram_axes":metrics["spectrogram_axes"],"ab_common_gain_db":metrics["ab_common_gain_db"]});
    if let Some(regions) = &region_comparison {
        result["regions"] = regions["differences"].clone();
        result["preservation"] = regions["preservation"].clone();
    }
    if include_image {
        result["__mcp_images"] = json!([image(&pa), image(&pb)]);
    }
    Ok(result)
}

fn prepare_render(args: &Value, mut snapshot: Snapshot) -> Result<(Snapshot, Request), ApiError> {
    if text_arg(args, "effects")? != "dry" {
        return Err(ApiError::params(
            "effects must be explicitly dry; hosted third-party effects and external input are not rendered",
        ));
    }
    if let Some(v) = arg(args, "values") {
        snapshot.override_values(v)?;
    }
    if let Some(v) = arg(args, "seed") {
        snapshot.seed = v
            .as_u64()
            .ok_or_else(|| ApiError::params("seed must be an unsigned integer"))?;
    }
    if let Some(v) = optional_number(args, "tempo")? {
        if !(30.0..=300.0).contains(&v) {
            return Err(ApiError::params("tempo must be 30 to 300 BPM"));
        }
        snapshot.tempo = v.round();
    }
    if let Some(v) = arg(args, "frequency_dials") {
        snapshot.hardware_frequency_dials = match v.as_str() {
            Some("HARDWARE") => true,
            Some("SEMITONES") => false,
            _ => {
                return Err(ApiError::params(
                    "frequency_dials must be SEMITONES or HARDWARE",
                ));
            }
        };
    }
    if let Some(v) = arg(args, "modulations") {
        snapshot.override_modulations(v)?;
    }
    let request = Request {
        notes: serde_json::from_value(
            arg(args, "notes")
                .cloned()
                .ok_or_else(|| ApiError::params("notes are required"))?,
        )
        .map_err(|e| ApiError::params(format!("notes: {e}")))?,
        seconds: number(args, "seconds", 3.0)?,
        sample_rate: match arg(args, "sample_rate") {
            None => 48000,
            Some(v) => v
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| ApiError::params("sample_rate must be a whole number"))?,
        },
    };
    snapshot.validate()?;
    request.validate()?;
    Ok((snapshot, request))
}

pub fn render(args: &Value, snapshot: Snapshot, preset_source: Option<Value>) -> Reply {
    let (snapshot, request) = prepare_render(args, snapshot)?;
    // Check the output before expensive work; all writes are confined to the new folder.
    let dir = directory(args)?;
    let result = crate::offline::render(&snapshot, &request)?;
    let bytes = wav(request.sample_rate, &result.channels)?;
    let wave_hash = sha(&bytes);
    let snapshot_hash = sha(&serde_json::to_vec(&snapshot).map_err(io_error)?);
    write(&dir.join("render.wav"), &bytes)?;
    let manifest = json!({"format":1,"engine_version":env!("CARGO_PKG_VERSION"),"engine_source_sha256":env!("CA72_ENGINE_SOURCE_SHA256"),
        "snapshot":snapshot,"snapshot_sha256":snapshot_hash,"request":request,"wav_sha256":wave_hash,
        "preset_file_source":if arg(args,"preset_file").is_some() {preset_source.clone()} else {None},"input_source":preset_source,"modulation_edits":arg(args,"modulations"),
        "latency_compensated_samples":result.latency_compensated_samples,"block_frames":crate::offline::BLOCK,
        "effects":"dry","external_input":"silence","initial_state":"fresh voices; seed resets noise and component variations; no live held notes or oscillator phase copied",
        "processing":"same Ca72::process as playback; offline, no device, real-time workers or live-state changes"});
    json_file(&dir.join("manifest.json"), &manifest)?;
    Ok(
        json!({"path":dir.join("render.wav"),"manifest_path":dir.join("manifest.json"),"wav_sha256":wave_hash,"snapshot_sha256":snapshot_hash,
        "snapshot":snapshot,"sample_rate":request.sample_rate,"frames":result.channels[0].len(),"effects":"dry","latency_compensated_samples":result.latency_compensated_samples}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exported_files_keep_modulation_curves_pot_positions_and_replay_conditions() {
        let dir = tempfile::tempdir().expect("dir");
        let p = crate::params::Ca72Params::default();
        crate::agent::modulation::set(
            &p,
            &json!({"source":"lfo1","target":"osc2_frequency","depth":20}),
        )
        .expect("assignment");
        p.drive_curve.load_preset([-2.0, -4.0, -5.0, -6.0]);
        let mut snapshot = Snapshot::capture(&p);
        snapshot.hardware_frequency_dials = true;
        snapshot.seed = 981;
        snapshot.tempo = 131.0;
        snapshot.override_values(&json!({"quality":"HI","osc3_frequency":8.15,"pitch_wheel":0.12,"midi_bend_range":7,"lfo1_rate":3})).expect("values");
        let result=export_preset(&json!({"snapshot":snapshot,"name":"Reference Fit","output_dir":dir.path().join("export")})).expect("export");
        let source =
            std::fs::read_to_string(result["preset_file"].as_str().expect("path")).expect("file");
        let sound = crate::library::parse_user_sound(&source).expect("parse");
        let (_, pot) = sound
            .values
            .iter()
            .find(|(id, _)| id == "osc3_frequency")
            .expect("pot");
        assert!(
            (7.4..7.5).contains(pot),
            "stored physical pot, not the agent's 8.15 semitones: {pot}"
        );
        assert!(
            crate::library::KEPT
                .iter()
                .chain(crate::presets::PLAYERS)
                .all(|id| !sound.values.iter().any(|(k, _)| k == id))
        );
        assert_eq!(sound.drive_curve, Some([-2.0, -4.0, -5.0, -6.0]));
        let mut restored = Snapshot::sound(&sound).expect("sound");
        restored
            .override_values(&result["render_conditions"]["values"])
            .expect("player conditions are valid tool inputs");
        let params = restored.restore_params().expect("restore");
        assert!(params.lfo_patch.same_as_values(&sound.values));
        assert_eq!(result["render_conditions"]["frequency_dials"], "HARDWARE");
        assert_eq!(result["render_conditions"]["seed"], 981);
        assert_eq!(result["render_conditions"]["tempo"], 131.0);
        assert!(source.contains("lfo1_to_osc2_frequency"));
        let invalid = dir.path().join("invalid.toml");
        std::fs::write(&invalid, "format = 88\nname = 'Reference Fit'\n[values]\n").expect("write");
        assert!(preset_file(&json!({"preset_file":invalid})).is_err());
        let dual = dir.path().join("dual.toml");
        std::fs::write(&dual,"format = 1\nname = 'Two curves'\ndrive_curve = [-1,-2,-3,-4]\ndrive_curve_lo = [-5,-6,-7,-8]\n[values]\ndrive = 12\n").expect("write");
        for (quality, expected) in [
            ("HI", [-1.0, -2.0, -3.0, -4.0]),
            ("LO", [-5.0, -6.0, -7.0, -8.0]),
            ("ULTRA", [-1.0, -2.0, -3.0, -4.0]),
        ] {
            let (s, _) = preset_file(&json!({"preset_file":dual,"values":{"quality":quality}}))
                .expect("quality-specific preset");
            assert_eq!(
                s.restore_params().expect("restore").drive_curve.saved().db,
                expected
            );
        }
    }
    #[test]
    fn files_round_trip_but_truncation_non_finite_and_partial_frames_are_refused() {
        let c = vec![vec![0.25; 4096], vec![-0.5; 4096]];
        let bytes = wav(48000, &c).expect("wav");
        assert_eq!(strict_wav(&bytes).expect("read").1, c);
        assert!(strict_wav(&bytes[..bytes.len() - 1]).is_err());
        let mut bad = bytes.clone();
        bad[44..48].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(strict_wav(&bad).is_err());
        let mut bad = bytes.clone();
        bad[40..44].copy_from_slice(&32767u32.to_le_bytes());
        assert!(strict_wav(&bad).is_err());
    }
    #[test]
    fn analysis_and_comparison_preserve_sources_and_reject_existing_output_and_bad_crops() {
        let d = tempfile::tempdir().expect("dir");
        let c = vec![(0..12000).map(|i| (i as f32 * 0.04).sin() * 0.1).collect()];
        let p = d.path().join("source.wav");
        let bytes = wav(48000, &c).expect("wav");
        write(&p, &bytes).expect("write");
        let args = json!({"path":p,"output_dir":d.path().join("analysis"),"include_image":false});
        assert!(analyze(&args).is_ok());
        assert!(analyze(&args).is_err());
        assert!(
            analyze(&json!({"path":p,"start":0.2,"seconds":0.2,"output_dir":d.path().join("bad")}))
                .is_err()
        );
        let r = compare(&json!({"reference":p,"candidate":p,"output_dir":d.path().join("compare"),"include_image":false})).expect("compare");
        assert_eq!(r["candidate_gain_db"], json!(0.0));
        assert_eq!(std::fs::read(&p).expect("source"), bytes);
        assert!(d.path().join("compare/reference_ab.wav").exists());
    }
}
