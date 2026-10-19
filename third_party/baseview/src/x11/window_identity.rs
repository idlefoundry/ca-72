use x11rb::connection::RequestConnection;
use x11rb::errors::ConnectionError;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode, Window};
use x11rb::wrapper::ConnectionExt as _;

pub(crate) fn set_identity_and_map(
    conn: &impl RequestConnection, window: Window, title: &str, top_level: bool,
) -> Result<(), ConnectionError> {
    if top_level {
        // ICCCM: STRING/8, instance and class each terminated by NUL, set before mapping.
        // Both standalone wrappers use the plug-in's name as their initial title. This
        // matches the desktop entry's StartupWMClass without giving hosted editors an
        // application identity of their own.
        let wm_class = format!("{title}\0{title}\0");
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            wm_class.as_bytes(),
        )?;
    }
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        title.as_bytes(),
    )?;
    conn.map_window(window)?;
    Ok(())
}

#[cfg(test)]
#[path = "window_identity/tests.rs"]
mod tests;
