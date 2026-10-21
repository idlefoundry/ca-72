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
