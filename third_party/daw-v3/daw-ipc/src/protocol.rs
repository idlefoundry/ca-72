//! The control channel between the host and a helper: one JSON object per line on the
//! helper's stdin and stdout. Every protocol line starts with [`PREFIX`], so that anything
//! else a plugin prints to stdout is recognised as log output rather than corrupting the
//! channel. Audio never goes through this channel; see [`crate::layout`].

use crate::layout::TransportName;
use serde::{Deserialize, Serialize};

/// Protocol version; the helper refuses a host with another version (3: `LoadState`, a
/// running plugin's state in `param_values`, D234; 4: `Params`, `ParamText`, the
/// `params_changed` notice and `ParamInfo`'s new fields, the CA-72's PATCHES.md change 6).
pub const VERSION: u32 = 4;

/// Starts every protocol line in both directions.
pub const PREFIX: &str = "\u{1e}DAW1 ";

/// A request from the host. `id` pairs it with its reply.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: u64,
    #[serde(flatten)]
    pub op: Op,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Checks the protocol version.
    Hello { version: u32 },
    /// Lists the plugins in a plugin file.
    Scan { path: String },
    /// Loads a plugin into a new instance and restores its state, without activating it.
    Load {
        instance: u32,
        path: String,
        plugin: String,
        /// Hex-encoded state chunk to restore.
        state: Option<String>,
    },
    /// Sets parameters (plain values), then activates the instance and starts its
    /// processing thread on the given transport.
    Activate {
        instance: u32,
        sample_rate: f64,
        max_block: u32,
        params: Vec<(u32, f64)>,
        transport: TransportName,
        /// The transport's current request word: blocks up to it are not the new
        /// processing thread's to process.
        start: u32,
        /// Ask for real-time scheduling for the processing thread.
        #[serde(default)]
        realtime: bool,
    },
    /// Saves the instance's state.
    SaveState { instance: u32 },
    /// Loads a state chunk (hex) into the instance, running or not (D234: the project's
    /// state for a device changed, an undo of a preset its plugin loaded): the host's
    /// doing, so what the plugin says of it after is not a preset of its own.
    LoadState { instance: u32, state: String },
    /// Stops, deactivates and destroys an instance.
    Unload { instance: u32 },
    /// Opens the instance's GUI (DESIGN 10.5) in a window titled `title`, at (`x`, `y`)
    /// on the screen if given; shows it again if open. Replies as [`Op::GuiInfo`].
    GuiOpen {
        instance: u32,
        title: String,
        #[serde(default)]
        x: Option<i32>,
        #[serde(default)]
        y: Option<i32>,
    },
    /// Lists the instance's parameters again, with their values now: a list of
    /// [`ParamInfo`] (PATCHES.md change 6).
    Params { instance: u32 },
    /// The plugin's own text for each (parameter ID, value) pair, the value in its units
    /// (`ParamInfo`'s): VST3's `getParamStringByValue` (with its units after a bare number),
    /// CLAP's `value_to_text`. A list as long as `values`, `null` where it gave none.
    ParamText {
        instance: u32,
        values: Vec<(u32, f64)>,
    },
    /// Moves the instance's GUI window.
    GuiMove { instance: u32, x: i32, y: i32 },
    /// Closes the instance's GUI.
    GuiClose { instance: u32 },
    /// The instance's GUI: `{open, floating, api, title, window, width, height, position,
    /// visible, children}`. When the user closes a GUI's window, the helper sends a
    /// notice `{notice: "gui_closed", instance}`.
    GuiInfo { instance: u32 },
    /// Exits the helper.
    Quit,
}

/// A reply, or a notice the helper sends on its own (`id` 0).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reply {
    pub id: u64,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub result: serde_json::Value,
}

/// A plugin as its descriptor presents it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginDesc {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub description: String,
    pub features: Vec<String>,
    /// `clap` or `vst3`.
    #[serde(default)]
    pub format: String,
}

/// A plugin parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ParamInfo {
    pub id: u32,
    pub name: String,
    pub module: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub stepped: bool,
    pub hidden: bool,
    pub automatable: bool,
    pub modulatable: bool,
    /// The parameter's value when the plugin was listed, after its state was restored
    /// (in the units of `min` and `max`): what it has while the project sets none, which
    /// is the state's value, not `default` (D159).
    #[serde(default)]
    pub value: Option<f64>,
    /// The plugin's own (plain) values along the range, at `PLAIN_STEPS + 1` even steps
    /// from `min` to `max`, where they differ from the range itself: a VST3 parameter is
    /// normalized here, and other programs write its values in the plugin's units, which
    /// need not lie evenly (D159). Empty: none.
    #[serde(default)]
    pub plain: Vec<f64>,
    /// Its value chooses the plugin's names for its keys (D216): a VST3 program change
    /// (`kIsProgramChange`), a CLAP enumeration (`CLAP_PARAM_IS_ENUM`).
    #[serde(default)]
    pub program: bool,
    /// Its short name (VST3's `shortTitle`; CLAP has none: empty).
    #[serde(default)]
    pub short: String,
    /// Its units (VST3's `units`, "dB"; CLAP has none: empty).
    #[serde(default)]
    pub units: String,
    /// How many steps it has past its first value (VST3's `stepCount`, a CLAP stepped
    /// parameter's range); 0 continuous. A VST3 parameter stays normalized (`stepped`
    /// false): the plugin quantizes its own value.
    #[serde(default)]
    pub steps: u32,
    /// Shown, not taken (VST3's `kIsReadOnly`, CLAP's `CLAP_PARAM_IS_READONLY`: a meter).
    #[serde(default)]
    pub read_only: bool,
    /// The plugin's own bypass (VST3's `kIsBypass`, CLAP's `CLAP_PARAM_IS_BYPASS`).
    #[serde(default)]
    pub bypass: bool,
}

/// The steps of [`ParamInfo::plain`].
pub const PLAIN_STEPS: usize = 16;

impl ParamInfo {
    /// The plain value for a normalized one (DESIGN 6.3 stores normalized values).
    pub fn plain(&self, normalized: f64) -> f64 {
        let v = self.min + (self.max - self.min) * normalized.clamp(0.0, 1.0);
        if self.stepped { v.round() } else { v }
    }

    /// The normalized value for a plain one.
    pub fn normalized(&self, plain: f64) -> f64 {
        if self.max > self.min {
            ((plain - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// The result of `Load`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Loaded {
    pub desc: PluginDesc,
    pub params: Vec<ParamInfo>,
    /// Channels of the main audio input and output ports (0 if absent).
    pub inputs: u32,
    pub outputs: u32,
    /// Whether the plugin takes note events.
    pub notes: bool,
    /// Whether restoring the given state succeeded.
    pub state_restored: bool,
    /// The plugin's names for its keys (CLAP's note-name extension, M18 W9; VST3's program
    /// pitch names, D211), as it gave them once loaded with its state.
    #[serde(default)]
    pub note_names: Vec<NoteName>,
}

/// A plugin's name for a key (CLAP `clap_note_name`, VST3's program pitch name): `channel`
/// -1 is every channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NoteName {
    pub key: i16,
    pub channel: i16,
    pub name: String,
}

/// A notice the helper sends on its own (`Reply` with ID 0) when a running instance's
/// names for its keys changed (D216): `{"notice": "note_names", "instance", "names"}`.
pub const NOTICE_NOTE_NAMES: &str = "note_names";

/// A notice the helper sends on its own when a running plugin edited its parameters itself
/// (D222: in its editor): `{"notice": "param_edits", "instance", "edits": [ParamEdit]}`.
pub const NOTICE_PARAM_EDITS: &str = "param_edits";

/// A notice the helper sends on its own when a running CLAP plugin asks the host to clear
/// what refers to a parameter (D222, `clap_host_params.clear`): `{"notice":
/// "param_clear", "instance", "param", "flags"}`, the flags CLAP's (`CLAP_PARAM_CLEAR_*`).
pub const NOTICE_PARAM_CLEAR: &str = "param_clear";

/// A notice the helper sends on its own when a running plugin said its parameters' values
/// changed by its own doing (D228: a preset loaded in its editor; VST3's
/// `restartComponent(kParamValuesChanged)`, CLAP's `clap_host_params.rescan` of values or
/// `clap_host_state.mark_dirty`), once it has been quiet a moment: every parameter's value
/// now, in its units: `{"notice": "param_values", "instance", "values": [[id, value]]}`.
///
/// Since D234 also when the helper finds them changed without a word (it polls a plugin's
/// values, and its state when that is cheap, while its editor is open), and with its state
/// when that may be wanted: `state` (hex), when values changed by its own doing (not its
/// editor's edits, nor the host's values) or its state changed alone (`alone`: no value nor
/// edit changed since its state was last read); `host`: within a second of a program or a
/// state the host gave it (the host's doing, D228's window as the helper sees it).
pub const NOTICE_PARAM_VALUES: &str = "param_values";

/// A notice the helper sends on its own when a running plugin said its parameters' names,
/// ranges or list changed (VST3's `restartComponent(kParamTitlesChanged)`, CLAP's
/// `clap_host_params.rescan` of info, text or all): `{"notice": "params_changed",
/// "instance"}`. The host lists them again ([`Op::Params`]; PATCHES.md change 6).
pub const NOTICE_PARAMS_CHANGED: &str = "params_changed";

/// What a plugin's own edit of a parameter is (D222): a gesture's beginning or end (VST3's
/// `beginEdit` and `endEdit`, CLAP's gesture events), or a value (`performEdit`, a CLAP
/// parameter value event).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EditKind {
    Begin,
    Value,
    End,
}

/// A plugin's own edit of a parameter (D222), in the plugin's units (`ParamInfo`'s: a
/// VST3 plugin's normalized, a CLAP plugin's plain): a value, or the value it had when a
/// gesture began or ended.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ParamEdit {
    pub param: u32,
    pub kind: EditKind,
    #[serde(default)]
    pub value: Option<f64>,
}

/// The result of `Activate`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Activated {
    pub latency: u32,
    /// Frames after which the plugin's output no longer depends on older input.
    pub tail: u32,
    /// The plugin's names for its keys once the project's values are set too (D211): a
    /// program chosen by a parameter names its keys. None from a helper that does not say.
    #[serde(default)]
    pub note_names: Option<Vec<NoteName>>,
}

/// Encodes a line for the channel.
pub fn encode<T: Serialize>(msg: &T) -> String {
    let body =
        serde_json::to_string(msg).unwrap_or_else(|e| format!("{{\"encode_error\":\"{e}\"}}"));
    format!("{PREFIX}{body}\n")
}

/// Decodes a protocol line; `None` for lines without the prefix (plugin output).
pub fn decode<T: for<'de> Deserialize<'de>>(line: &str) -> Option<Result<T, String>> {
    let body = line.trim_end_matches(['\r', '\n']).strip_prefix(PREFIX)?;
    Some(serde_json::from_str(body).map_err(|e| e.to_string()))
}

/// Lower-case hex.
pub fn hex(bytes: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(D[(b >> 4) as usize] as char);
        s.push(D[(b & 15) as usize] as char);
    }
    s
}

pub fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd hex length".into());
    }
    let digit = |c: u8| -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(format!("bad hex digit {:?}", c as char)),
        }
    };
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| Ok(digit(p[0])? << 4 | digit(p[1])?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_round_trip_and_foreign_lines_are_ignored() {
        let r = Request {
            id: 7,
            op: Op::Load {
                instance: 1,
                path: "/p.clap".into(),
                plugin: "a.b".into(),
                state: Some(hex(&[0, 1, 254, 255])),
            },
        };
        let line = encode(&r);
        assert!(line.ends_with('\n'));
        assert_eq!(decode::<Request>(&line), Some(Ok(r)));
        assert_eq!(decode::<Request>("hello from a plugin\n"), None);
        assert_eq!(unhex("0001feff").unwrap(), vec![0, 1, 254, 255]);
        assert!(unhex("0g").is_err());
        let p = ParamInfo {
            min: 1.0,
            max: 11.0,
            stepped: true,
            ..ParamInfo::default()
        };
        assert_eq!(p.plain(0.44), 5.0);
        assert_eq!(p.normalized(6.0), 0.5);
    }
}
