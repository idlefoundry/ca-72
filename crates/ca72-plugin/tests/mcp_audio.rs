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
}
