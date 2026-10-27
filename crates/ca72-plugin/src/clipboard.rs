//! Text put on the computer's clipboard (SETTINGS: AGENTS' setups, decisions.md R-MCP), by the
//! system's own tool: `pbcopy` on macOS, `clip` on Windows (UTF-16 with its mark, so that any
//! path survives; no console shown), and on Linux `wl-copy` under Wayland, else `xclip` or
//! `xsel`. The editor's window library has no clipboard of its own on Windows and Linux.

use std::io::Write;
use std::process::{Command, Stdio};

/// `text` put on the clipboard: whether it got there.
pub fn copy(text: &str) -> bool {
    tools().into_iter().any(|(tool, args, bytes)| {
        let mut command = Command::new(tool);
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let Ok(mut child) = command.spawn() else {
            return false;
        };
        let wrote = child
            .stdin
            .take()
            .is_some_and(|mut input| input.write_all(&bytes(text)).is_ok());
        child.wait().is_ok_and(|s| s.success()) && wrote
    })
}

type Tool = (&'static str, &'static [&'static str], fn(&str) -> Vec<u8>);

fn utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

#[cfg_attr(not(windows), allow(dead_code))]
fn utf16_marked(text: &str) -> Vec<u8> {
    std::iter::once(0xFEFF)
        .chain(text.encode_utf16())
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// The tools to try, in order.
fn tools() -> Vec<Tool> {
    if cfg!(target_os = "macos") {
        vec![("pbcopy", &[], utf8)]
    } else if cfg!(windows) {
        vec![("clip", &[], utf16_marked)]
    } else {
        let mut list: Vec<Tool> = Vec::new();
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            list.push(("wl-copy", &[], utf8));
        }
        list.push(("xclip", &["-selection", "clipboard"], utf8));
        list.push(("xsel", &["--clipboard", "--input"], utf8));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows' `clip` is given UTF-16, little-endian, after its byte-order mark.
    #[test]
    fn clip_is_given_marked_utf16() {
        assert_eq!(utf16_marked("C:é"), [0xFF, 0xFE, b'C', 0, b':', 0, 0xE9, 0]);
    }
}
