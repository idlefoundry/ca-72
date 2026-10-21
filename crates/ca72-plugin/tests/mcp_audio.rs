//! A synthetic held-bass reference fitted through the shipped bridge's stdio interface.
//! This tests the measurement loop, not reproduction of the original White Lotus recording.
#![allow(clippy::unwrap_used)]

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next: u64,
}
impl Client {
    fn new(home: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ca72-mcp"))
            .env("IDLE_FOUNDRY_SHARED", home.join("isolated-presets"))
            .env("XDG_DATA_HOME", home.join("data"))
            .env("CA72_AGENT_DIR", home.join("no-live-instance"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            input,
            output,
            next: 0,
        }
    }
    fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        writeln!(
            self.input,
            "{}",
            json!({"jsonrpc":"2.0","id":self.next,"method":method,"params":params})
        )
        .unwrap();
        self.input.flush().unwrap();
        let mut line = String::new();
        assert!(
            self.output.read_line(&mut line).unwrap() > 0,
            "bridge ended"
        );
        let result: Value = serde_json::from_str(&line).unwrap();
        assert!(result.get("error").is_none(), "{result}");
        result["result"].clone()
    }
    fn tool(&mut self, name: &str, args: Value) -> Value {
        self.rpc("tools/call", json!({"name":name,"arguments":args}))
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn ok(result: Value) -> Value {
    assert_eq!(result["isError"], false, "{result}");
    result["structuredContent"].clone()
}

#[test]
fn a_synthetic_white_lotus_reference_is_measured_fitted_and_replayed_without_a_device() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    let initialized = c.rpc(
        "initialize",
        json!({"protocolVersion":"2025-11-25","clientInfo":{"name":"audio-loop-test"}}),
    );
    assert!(initialized.pointer("/capabilities/prompts").is_some());
    let prompt = c.rpc(
        "prompts/get",
        json!({"name":"ca72_match_reference","arguments":{"reference":"/local/reference.wav"}}),
    );
    assert!(
        prompt["messages"][0]["content"]["text"]
            .as_str()
            .unwrap()
            .contains("ca72_compare")
    );
    let notes =
        json!([{"note":38,"start":0.0,"duration":0.25},{"note":36,"start":0.3,"duration":0.25}]);
    let reference=ok(c.tool("ca72_render",json!({"preset":"White Lotus","effects":"dry","values":{"quality":"LO","cutoff":-2.0},"notes":notes,"seconds":0.65,"sample_rate":48000,"seed":72,"output_dir":dir.path().join("reference")})));
    assert_eq!(reference["frames"], 31200);
    let analyzed=c.tool("ca72_analyze",json!({"path":reference["path"],"fundamental_hz":73.4162,"modulation_hz":11.3,"output_dir":dir.path().join("analysis")}));
    assert_eq!(analyzed["isError"], false, "{analyzed}");
    assert_eq!(analyzed["content"][1]["type"], "image");
    assert!(analyzed["structuredContent"].get("__mcp_images").is_none());
    let mut errors = Vec::new();
    for (i, cutoff) in [-1.4, -2.0, -2.6].iter().enumerate() {
        let candidate=ok(c.tool("ca72_render",json!({"snapshot":reference["snapshot"],"effects":"dry","values":{"cutoff":cutoff},"notes":notes,"seconds":0.65,"sample_rate":48000,"output_dir":dir.path().join(format!("candidate-{i}"))})));
        let compared=ok(c.tool("ca72_compare",json!({"reference":reference["path"],"candidate":candidate["path"],"include_image":false,"output_dir":dir.path().join(format!("comparison-{i}"))})));
        let error = compared["resolutions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["spectral_mae_db"].as_f64().unwrap())
            .sum::<f64>()
            / 3.0;
        errors.push(error);
        if i == 1 {
            assert_eq!(
                reference["wav_sha256"], candidate["wav_sha256"],
                "same snapshot repeats exactly"
            );
        }
    }
    assert!(errors[0] > 1.0 && errors[2] > 1.0, "{errors:?}");
    assert!(errors[1] < 0.0001, "{errors:?}");
    let existing=c.tool("ca72_render",json!({"snapshot":reference["snapshot"],"effects":"dry","notes":notes,"seconds":0.65,"output_dir":dir.path().join("reference")}));
    assert_eq!(existing["isError"], true);
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("reference/manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["wav_sha256"], reference["wav_sha256"]);
    assert!(manifest["engine_source_sha256"].as_str().unwrap().len() == 64);
    assert_eq!(manifest["effects"], "dry");
    // A real preset file delivered without opening or changing an application or the
    // player's library; reloading this exact file does not resolve a factory name.
    let exported = ok(c.tool("ca72_preset_export", json!({"snapshot":reference["snapshot"],"name":"White Lotus","tags":["bass"],"output_dir":dir.path().join("export")})));
    let mut replay = exported["render_conditions"].clone();
    replay["preset_file"] = exported["preset_file"].clone();
    replay["effects"] = json!("dry");
    replay["notes"] = notes.clone();
    replay["seconds"] = json!(0.65);
    replay["output_dir"] = json!(dir.path().join("file-replay"));
    let reloaded = ok(c.tool("ca72_render", replay));
    let same = ok(c.tool("ca72_compare", json!({"reference":reference["path"],"candidate":reloaded["path"],"include_image":false,"output_dir":dir.path().join("file-replay-check")})));
    assert!(same["candidate_gain_db"].as_f64().unwrap().abs() < 0.1);
    assert!(
        same["resolutions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["spectral_mae_db"].as_f64().unwrap() < 0.5)
    );
    let source: Value = serde_json::from_slice(
        &std::fs::read(dir.path().join("file-replay/manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        source["preset_file_source"]["sha256"],
        exported["preset_sha256"]
    );
    assert!(
        !dir.path().join("isolated-presets").exists(),
        "no library installation"
    );
    let refused=c.tool("ca72_render",json!({"preset":"White Lotus","preset_file":exported["preset_file"],"effects":"dry","notes":notes,"output_dir":dir.path().join("ambiguous-file")}));
    assert_eq!(refused["isError"], true);
    let existing=c.tool("ca72_preset_export",json!({"snapshot":reference["snapshot"],"name":"Overwrite","output_dir":dir.path().join("export")}));
    assert_eq!(existing["isError"], true);
}

#[test]
fn computer_preset_shortcuts_cannot_consume_offline_performance_notes() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    let notes = json!([{"note":38,"start":0,"duration":0.2}]);
    let before=ok(c.tool("ca72_render",json!({"preset":"White Lotus","effects":"dry","notes":notes,"seconds":0.3,"output_dir":dir.path().join("before")})));
    let settings = dir.path().join("isolated-presets/CA-72");
    std::fs::create_dir_all(&settings).unwrap();
    let text = "[preset_actions.next_preset]\nchannel = 1\nnote = 38\n";
    std::fs::write(settings.join("settings.toml"), text).unwrap();
    let mut other = Client::new(dir.path());
    let after=ok(other.tool("ca72_render",json!({"snapshot":before["snapshot"],"effects":"dry","notes":notes,"seconds":0.3,"output_dir":dir.path().join("after")})));
    assert_eq!(before["wav_sha256"], after["wav_sha256"]);
    let analyzed = ok(other.tool(
        "ca72_analyze",
        json!({"path":after["path"],"include_image":false,"output_dir":dir.path().join("audible")}),
    ));
    assert!(analyzed["features"]["peak_dbfs"].as_f64().unwrap() > -60.0);
    assert_eq!(
        std::fs::read_to_string(settings.join("settings.toml")).unwrap(),
        text
    );
}

#[test]
fn named_preset_rendering_selects_the_requested_qualitys_curve() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    let factory: toml::Value = toml::from_str(include_str!("../sounds/presets.toml")).unwrap();
    let preset = factory["preset"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"].as_str() == Some("White Lotus"))
        .unwrap();
    for quality in ["LO", "HI", "ULTRA"] {
        let render=ok(c.tool("ca72_render",json!({"preset":"White Lotus","effects":"dry","values":{"quality":quality,"drive":12},"notes":[{"note":38,"start":0,"duration":0.08}],"seconds":0.12,"output_dir":dir.path().join(quality)})));
        let actual: Value = serde_json::from_str(
            render["snapshot"]["fields"]["drive_curve"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let key = if quality == "LO" {
            "drive_curve_lo"
        } else {
            "drive_curve"
        };
        for (a, b) in actual["db"]
            .as_array()
            .unwrap()
            .iter()
            .zip(preset[key].as_array().unwrap())
        {
            assert!(
                (a.as_f64().unwrap() - b.as_float().unwrap()).abs() < 1e-5,
                "{quality}: wrong AUTO GAIN curve"
            );
        }
    }
}

#[test]
fn offline_env_routing_retriggers_per_voice_and_survives_preset_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    let notes = json!([
        {"note":60,"start":0,"duration":0.75},
        {"note":72,"start":0.35,"duration":0.35}
    ]);
    let values = json!({"quality":"LO","poly":true,"osc1_on":true,"osc2_on":false,
        "osc3_on":false,"noise_on":false,"emphasis":0,"contour_amount":0,
        "osc1_volume":0,"loudness_attack":0,"loudness_sustain":10,"volume":10,
        "lfo1_mode":"Env","lfo1_waveform":"Sawtooth","lfo1_reverse":true,
        "lfo1_polarity":"Unipolar","lfo1_rate":4,"lfo1_mono":false,
        "lfo1_phase":0,"lfo1_rise":0,"retrigger":false});
    let mut args = json!({"preset":"White Lotus","effects":"dry","values":values,
        "modulations":[{"source":"lfo1","target":"osc1_volume","depth":100}],
        "notes":notes,"seconds":0.85,"sample_rate":48000,
        "output_dir":dir.path().join("env")});
    let rendered = ok(c.tool("ca72_render", args.clone()));
    let bytes = std::fs::read(rendered["path"].as_str().unwrap()).unwrap();
    let (rate, audio) = ca72_analysis::wav::read(&bytes).unwrap();
    let ac_rms = |start: f64, end: f64| {
        let a = (start * f64::from(rate)) as usize;
        let b = (end * f64::from(rate)) as usize;
        let mean = audio[0][a..b].iter().map(|x| f64::from(*x)).sum::<f64>() / (b - a) as f64;
        (audio[0][a..b]
            .iter()
            .map(|x| (f64::from(*x) - mean).powi(2))
            .sum::<f64>()
            / (b - a) as f64)
            .sqrt()
    };
    // Mixer changes also move the circuit's DC operating point. Judge the audible
    // oscillator energy, allowing that low-frequency settling tail to remain.
    let first = ac_rms(0.04, 0.10);
    let second = ac_rms(0.39, 0.45);
    assert!(first > 0.0001, "first envelope sounds");
    assert!(
        ac_rms(0.28, 0.32) < first * 0.01,
        "ENV attenuates the held note by at least 40 dB"
    );
    assert!(
        second > first * 0.2,
        "new poly voice has its own envelope while first key remains held"
    );
    assert!(
        ac_rms(0.64, 0.68) < second * 0.01,
        "second envelope also finishes"
    );
    let exported = ok(c.tool(
        "ca72_preset_export",
        json!({"snapshot":rendered["snapshot"],
        "name":"Envelope Study","output_dir":dir.path().join("export")}),
    ));
    let mut replay = exported["render_conditions"].clone();
    replay["preset_file"] = exported["preset_file"].clone();
    replay["effects"] = json!("dry");
    replay["notes"] = notes;
    replay["seconds"] = json!(0.85);
    replay["output_dir"] = json!(dir.path().join("replay"));
    let reloaded = ok(c.tool("ca72_render", replay));
    assert_eq!(
        rendered["snapshot"]["fields"]["lfo"],
        reloaded["snapshot"]["fields"]["lfo"]
    );
    let compared = ok(c.tool("ca72_compare", json!({"reference":rendered["path"],
        "candidate":reloaded["path"],"include_image":false,"output_dir":dir.path().join("reload-check")})));
    assert!(
        compared["resolutions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["spectral_mae_db"].as_f64().unwrap() < 0.5)
    );
    // A malformed batch neither creates output nor terminates the bridge, including
    // source zero (formerly an eager unsigned subtraction in the shared live parser).
    for (i, edits) in [
        json!([{"source":"lfo1","target":"cutoff","depth":20},{"source":0,"target":"cutoff"}]),
        json!([{"source":"lfo1","target":"cutoff","deph":20}]),
        json!([{"source":"lfo1","target":"cutoff","depth":101}]),
        json!([{"source":"lfo1","remove":true,"depth":20}]),
        json!([{"source":"lfo1","target":"fx1","depth":20}]),
    ]
    .into_iter()
    .enumerate()
    {
        let output = dir.path().join(format!("bad-{i}"));
        args["output_dir"] = json!(output);
        args["modulations"] = edits;
        assert_eq!(c.tool("ca72_render", args.clone())["isError"], true);
        assert!(!output.exists());
    }
    args["snapshot"] = rendered["snapshot"].clone();
    args.as_object_mut().unwrap().remove("preset");
    args["values"] = json!({"osc1_volume":10});
    args["modulations"] = json!([{"source":"lfo1","remove":true}]);
    args["output_dir"] = json!(dir.path().join("removed"));
    let removed = ok(c.tool("ca72_render", args));
    assert_ne!(removed["wav_sha256"], rendered["wav_sha256"]);
}

/// A held first key makes the second key legato. Individual RETRIG must restart RISE in
/// the actual shipped bridge with the main envelope RETRIGGER off, including file replay.
#[test]
fn offline_individual_retrig_restarts_rise_on_legato_notes() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    let notes = json!([
        {"note":60,"start":0,"duration":0.9},
        {"note":48,"start":0.5,"duration":0.4}
    ]);
    let mut args = json!({"preset":"White Lotus","effects":"dry",
        "values":{"quality":"LO","poly":false,"osc1_on":true,"osc2_on":false,
        "osc3_on":false,"noise_on":false,"emphasis":0,"contour_amount":0,
        "osc1_volume":10,"loudness_attack":0,"loudness_sustain":10,"volume":0,
        "lfo1_on":true,"lfo1_mode":"Retrig","lfo1_waveform":"Square",
        "lfo1_polarity":"Unipolar","lfo1_rate":0.01,"lfo1_mono":true,
        "lfo1_phase":0,"lfo1_rise":0.4,"retrigger":false},
        "modulations":[{"source":"lfo1","target":"volume","depth":100}],
        "notes":notes,"seconds":0.95,"sample_rate":48000,
        "output_dir":dir.path().join("individual")});
    let rendered = ok(c.tool("ca72_render", args.clone()));
    assert_eq!(rendered["snapshot"]["parameters"]["retrigger"], 0.0);
    let rms = |path: &Value, start: f64, end: f64| {
        let bytes = std::fs::read(path.as_str().unwrap()).unwrap();
        let (rate, audio) = ca72_analysis::wav::read(&bytes).unwrap();
        let a = (start * f64::from(rate)) as usize;
        let b = (end * f64::from(rate)) as usize;
        let mean = audio[0][a..b].iter().map(|x| f64::from(*x)).sum::<f64>() / (b - a) as f64;
        (audio[0][a..b]
            .iter()
            .map(|x| (f64::from(*x) - mean).powi(2))
            .sum::<f64>()
            / (b - a) as f64)
            .sqrt()
    };
    let late_first = rms(&rendered["path"], 0.32, 0.40);
    let reset = rms(&rendered["path"], 0.52, 0.56);
    assert!(late_first > 1e-4);
    assert!(
        reset < late_first * 0.2,
        "individual RISE restarts on the overlapping key"
    );
    assert!(
        rms(&rendered["path"], 0.82, 0.88) > reset * 4.0,
        "RISE builds again"
    );
    args["values"]["lfo1_mode"] = json!("Free");
    args["output_dir"] = json!(dir.path().join("free"));
    let free = ok(c.tool("ca72_render", args));
    assert!(
        rms(&free["path"], 0.52, 0.56) > reset * 4.0,
        "FREE keeps running through legato"
    );
    let e = ok(c.tool(
        "ca72_preset_export",
        json!({"snapshot":rendered["snapshot"],
        "name":"Individual Retrig","output_dir":dir.path().join("export")}),
    ));
    let mut replay = e["render_conditions"].clone();
    replay["preset_file"] = e["preset_file"].clone();
    replay["effects"] = json!("dry");
    replay["notes"] = notes;
    replay["seconds"] = json!(0.95);
    replay["output_dir"] = json!(dir.path().join("replay"));
    let reloaded = ok(c.tool("ca72_render", replay));
    assert_eq!(reloaded["snapshot"]["parameters"]["retrigger"], 0.0);
    let reload_reset = rms(&reloaded["path"], 0.52, 0.56);
    let reload_late = rms(&reloaded["path"], 0.82, 0.88);
    assert!(reload_reset < late_first * 0.2);
    assert!(reload_late > 1e-4 && reload_late > reload_reset * 4.0);
}

#[test]
fn named_regions_preservation_and_experiments_work_through_production_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::new(dir.path());
    c.rpc(
        "initialize",
        json!({"protocolVersion":"2025-11-25","clientInfo":{"name":"experiment-test"}}),
    );
    let init=ok(c.tool("ca72_render",json!({"init":true,"effects":"dry","notes":[{"note":60,"start":0.0,"duration":0.1}],"seconds":0.15,"values":{"quality":"LO"},"output_dir":dir.path().join("init")})));
    assert_eq!(init["frames"], 7200);
    let invalid_init=c.tool("ca72_render",json!({"init":false,"effects":"dry","notes":[{"note":60,"start":0.0,"duration":0.1}],"seconds":0.15,"output_dir":dir.path().join("invalid-init")}));
    assert_eq!(invalid_init["isError"], true);
    assert!(!dir.path().join("invalid-init").exists());
    let notes = json!([{"note":69,"start":0.05,"duration":0.7}]);
    let reference=ok(c.tool("ca72_render",json!({"preset":"White Lotus","effects":"dry","values":{"quality":"LO","cutoff":-1.0},"notes":notes,"seconds":0.9,"seed":72,"output_dir":dir.path().join("reference")})));
    let regions = json!([{"id":"hold","start":0.15,"seconds":0.5,"fundamentals_hz":[440.0]}]);
    let analyzed=ok(c.tool("ca72_analyze",json!({"path":reference["path"],"regions":regions,"include_image":false,"output_dir":dir.path().join("regions")})));
    assert_eq!(analyzed["regions"][0]["id"], "hold");
    assert_eq!(
        analyzed["regions"][0]["harmonic_levels_db"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    let args = json!({"snapshot":reference["snapshot"],"reference":reference["path"],"approved":reference["path"],"effects":"dry","notes":notes,"seconds":0.9,"regions":regions,"preservation_limits":{"envelope_rmse_db":0.1,"envelope_p95_db":0.2},"candidates":[{"id":"control"},{"id":"darker","values":{"cutoff":-2.0}}],"output_dir":dir.path().join("experiment")});
    let mut invalid = args.clone();
    invalid["output_dir"] = json!(dir.path().join("invalid"));
    invalid["candidates"][1]["values"] = json!({"cutoff":900});
    assert_eq!(c.tool("ca72_experiment", invalid)["isError"], true);
    assert!(
        !dir.path().join("invalid").exists(),
        "validate the entire batch before output"
    );
    let mut oversized = args.clone();
    oversized["output_dir"] = json!(dir.path().join("oversized"));
    oversized["candidates"] = json!(
        (0..9)
            .map(|i| json!({"id":format!("c{i}")}))
            .collect::<Vec<_>>()
    );
    assert_eq!(c.tool("ca72_experiment", oversized)["isError"], true);
    assert!(!dir.path().join("oversized").exists());
    let run = ok(c.tool("ca72_experiment", args));
    assert_eq!(run["reference_decodes"], 1);
    assert_eq!(run["reference_region_analysis_passes"], 1);
    let rows = run["candidates"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["wav_sha256"], reference["wav_sha256"]);
    assert_eq!(rows[0]["preservation"]["within_limits"], true);
    assert!(rows[0]["regions"][0]["harmonic_rmse_db"].as_f64().unwrap() < 1e-8);
    assert!(std::path::Path::new(run["report_path"].as_str().unwrap()).is_file());
    let replay=ok(c.tool("ca72_render",json!({"snapshot_file":rows[0]["snapshot_path"],"effects":"dry","notes":notes,"seconds":0.9,"output_dir":dir.path().join("file-replay")})));
    assert_eq!(replay["wav_sha256"], reference["wav_sha256"]);
    let exported=ok(c.tool("ca72_preset_export",json!({"snapshot_file":rows[0]["snapshot_path"],"name":"Experiment Winner","output_dir":dir.path().join("export")})));
    assert!(std::path::Path::new(exported["preset_file"].as_str().unwrap()).is_file());
    let compared=ok(c.tool("ca72_compare",json!({"reference":reference["path"],"candidate":rows[0]["render_path"],"approved":reference["path"],"regions":regions,"preservation_limits":{"envelope_rmse_db":0.1},"include_image":false,"output_dir":dir.path().join("compare-regions")})));
    assert_eq!(compared["preservation"]["within_limits"], true);
    assert_eq!(compared["regions"][0]["id"], "hold");
    let mut bad = json!({"reference":reference["path"],"candidate":reference["path"],"approved":reference["path"],"regions":regions,"preservation_limits":{"am_depth_db":0.1},"include_image":false,"output_dir":dir.path().join("missing-motion")});
    assert_eq!(c.tool("ca72_compare", bad.clone())["isError"], true);
    assert!(!dir.path().join("missing-motion").exists());
    bad["preservation_limits"] = json!({"envelope_rmse_db":0.1});
    bad["regions"][0]["modulation_hz"] = json!(4.2);
    bad["output_dir"] = json!(dir.path().join("short-motion"));
    bad["preservation_limits"]["am_depth_db"] = json!(0.1);
    let unresolved = ok(c.tool("ca72_compare", bad));
    assert!(
        unresolved["preservation"]["within_limits"].is_null(),
        "unresolved requested motion cannot pass"
    );
    assert!(
        c.rpc("tools/list", json!({}))["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "ca72_experiment")
    );
}
