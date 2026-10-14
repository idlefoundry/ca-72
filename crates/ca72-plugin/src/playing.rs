//! Editor-side ownership of pointer and typing notes. Never part of a preset or host state.

use ca72_panel::keyboard::{self, Scene};
use keyboard_types::{Code, Key, KeyState, KeyboardEvent, Modifiers};
use std::sync::Arc;

use crate::local_notes::LocalNotes;

#[derive(Clone, Copy, Debug)]
struct PlayedKey {
    note: u8,
    code: Code,
    sounding: bool,
}

#[derive(Debug)]
pub struct Playing {
    pub notes: Arc<LocalNotes>,
    pub visible: bool,
    pub typing: bool,
    pub focused: bool,
    pub root: u8,
    pub typing_root: u8,
    pub hover: Option<keyboard::Target>,
    pub wants_keys: bool,
    pointer_down: bool,
    pointer_note: Option<u8>,
    typed: [Option<PlayedKey>; 17],
    octave_keys: [Option<Code>; 2],
    owned: [u8; 128],
    generation: u64,
}

impl Default for Playing {
    fn default() -> Self {
        Self {
            notes: Arc::new(LocalNotes::default()),
            visible: false,
            typing: false,
            focused: true,
            root: 41,
            typing_root: keyboard::TYPE_ROOT,
            hover: None,
            wants_keys: false,
            pointer_down: false,
            pointer_note: None,
            typed: [None; 17],
            octave_keys: [None; 2],
            owned: [0; 128],
            generation: 0,
        }
    }
}

impl Playing {
    pub fn sync(&mut self) {
        let generation = self.notes.generation();
        if generation != self.generation {
            self.clear();
            self.generation = generation;
        }
    }

    fn clear(&mut self) {
        self.owned = [0; 128];
        self.typed = [None; 17];
        self.octave_keys = [None; 2];
        self.pointer_note = None;
        self.pointer_down = false;
    }

    pub fn release(&mut self) {
        self.clear();
        self.notes.release();
        self.generation = self.notes.generation();
    }

    pub fn panic(&mut self) {
        self.clear();
        self.notes.panic();
        self.generation = self.notes.generation();
    }

    pub fn panic_standalone(&mut self) {
        self.notes.await_standalone_panic();
        self.panic();
    }

    pub fn shift(&mut self, up: bool) {
        self.release_pointer();
        self.root = keyboard::shifted(self.root, up);
    }

    pub fn shift_typing(&mut self, up: bool) {
        let root = keyboard::shifted_typing(self.typing_root, up);
        if root == self.typing_root {
            return;
        }
        // Only TYPE notes change octave. A pointer or physical MIDI owner of
        // the same pitch keeps playing until its own release.
        for index in 0..self.typed.len() {
            if let Some(held) = &mut self.typed[index]
                && held.sounding
            {
                let note = held.note;
                held.sounding = false;
                self.own(note, false);
            }
        }
        // Retain each physical key until keyup, including hosts that forward
        // repeating keydowns without setting the repeat flag.
        self.typing_root = root;
    }

    fn own(&mut self, note: u8, on: bool) -> bool {
        if note > 127 {
            return false;
        }
        let count = &mut self.owned[usize::from(note)];
        let changed = if on {
            *count = count.saturating_add(1);
            *count == 1
        } else if *count > 0 {
            *count -= 1;
            *count == 0
        } else {
            false
        };
        if changed && !self.notes.note(note, on) {
            self.sync();
            return false;
        }
        true
    }

    pub fn pointer(&mut self, note: Option<u8>) {
        if self.pointer_note == note {
            return;
        }
        if let Some(old) = self.pointer_note.take() {
            self.own(old, false);
        }
        if let Some(note) = note
            && self.own(note, true)
        {
            self.pointer_note = Some(note);
        }
    }

    pub fn press(&mut self, note: u8) {
        self.sync();
        self.pointer_down = true;
        self.pointer(Some(note));
    }

    pub fn drag(&mut self, note: Option<u8>) {
        self.sync();
        if self.pointer_down {
            self.pointer(note);
        }
    }

    pub fn release_pointer(&mut self) {
        self.sync();
        self.pointer(None);
        self.pointer_down = false;
    }

    pub fn dragging(&self) -> bool {
        self.pointer_down
    }

    pub fn active(&self) -> bool {
        self.pointer_down || self.owned.iter().any(|count| *count != 0)
    }

    pub fn key(&mut self, event: &KeyboardEvent, allowed: bool) -> bool {
        self.sync();
        let modified = event
            .modifiers
            .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
        if event.code != Code::Unidentified
            && let Some(index) = self
                .octave_keys
                .iter()
                .position(|held| *held == Some(event.code))
        {
            if event.state == KeyState::Up {
                self.octave_keys[index] = None;
                return true;
            }
            return allowed && !modified;
        }
        // A key's character may change while held (Shift turns ';' into ':', a
        // layout/dead key can even produce a named key). Its physical release
        // still belongs to the recorded pitch, before any modifier/takeover check.
        if event.code != Code::Unidentified
            && let Some(index) = self
                .typed
                .iter()
                .position(|held| held.is_some_and(|held| held.code == event.code))
        {
            if event.state == KeyState::Up {
                let held = self.typed[index].take().expect("matched held key");
                if held.sounding {
                    self.own(held.note, false);
                }
                return true;
            }
            return allowed && !modified;
        }
        let Key::Character(s) = &event.key else {
            return false;
        };
        if s.len() != 1 {
            return false;
        }
        let character = match s.as_bytes()[0].to_ascii_lowercase() {
            b':' if event.state == KeyState::Up => b';', // Forwarded VST3 release: no code.
            character => character,
        };
        if let Some(index) = b"zx".iter().position(|c| *c == character) {
            if event.state == KeyState::Up {
                if self.octave_keys[index].is_some_and(|code| {
                    event.code == Code::Unidentified || code == Code::Unidentified
                }) {
                    self.octave_keys[index] = None;
                    return true;
                }
                return allowed && !modified;
            }
            if !allowed || modified {
                return false;
            }
            if self.octave_keys[index].is_none() && !event.repeat {
                self.octave_keys[index] = Some(event.code);
                self.shift_typing(index == 1);
            }
            return true;
        }
        let Some(index) = keyboard::MAPPING.bytes().position(|c| c == character) else {
            return false;
        };
        if event.state == KeyState::Up {
            if self.typed[index].is_some_and(|held| {
                event.code == Code::Unidentified || held.code == Code::Unidentified
            }) {
                let held = self.typed[index].take().expect("character fallback");
                if held.sounding {
                    self.own(held.note, false);
                }
                return true;
            }
            return allowed;
        }
        if !allowed || modified {
            return false;
        }
        if self.typed[index].is_some() || event.repeat {
            return true;
        }
        let note = self.typing_root.saturating_add(index as u8);
        self.typed[index] = Some(PlayedKey {
            note,
            code: event.code,
            sounding: note <= 127,
        });
        if note <= 127 {
            self.own(note, true);
        }
        true
    }

    pub fn holds_keys(&self) -> bool {
        self.visible && self.typing && self.focused
    }

    pub fn scene(&self, skin: ca72_panel::Skin) -> Option<Scene> {
        self.visible.then(|| Scene {
            root: self.root,
            typing: self.typing,
            typing_root: self.typing_root,
            held: self.notes.held(),
            hover: self
                .hover
                .filter(|t| !matches!(t, keyboard::Target::Note(_) | keyboard::Target::Back)),
            skin,
        })
    }
}

impl Drop for Playing {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(state: KeyState, character: &str) -> KeyboardEvent {
        KeyboardEvent {
            state,
            key: Key::Character(character.into()),
            ..Default::default()
        }
    }

    #[test]
    fn pointer_and_typing_share_a_note_and_glissando_can_reenter_after_leaving() {
        let mut p = Playing::default();
        p.press(48);
        p.key(&key(KeyState::Down, "a"), true);
        assert!(p.notes.pop().expect("first attack").on);
        assert!(
            p.notes.pop().is_none(),
            "same note only attacked once locally"
        );
        p.release_pointer();
        assert!(p.notes.pop().is_none(), "typing still owns it");
        p.key(&key(KeyState::Up, "A"), true);
        assert!(!p.notes.pop().expect("last owner released").on);
        p.press(48);
        p.drag(None);
        assert!(p.dragging());
        p.drag(Some(50));
        p.release_pointer();
        let events: Vec<_> = (0..4)
            .map(|_| p.notes.pop().expect("glissando event"))
            .collect();
        assert_eq!(
            events.iter().map(|e| (e.note, e.on)).collect::<Vec<_>>(),
            [(48, true), (48, false), (50, true), (50, false)]
        );
    }

    #[test]
    fn changing_shift_or_other_modifiers_releases_the_recorded_physical_key() {
        for code in [Code::Semicolon, Code::Unidentified] {
            let mut p = Playing::default();
            let mut down = key(KeyState::Down, ";");
            down.code = code;
            assert!(p.key(&down, true));
            assert_eq!(p.notes.pop().expect("semicolon attack").note, 64);
            let mut up = key(KeyState::Up, ":");
            up.code = code;
            up.modifiers = Modifiers::SHIFT | Modifiers::ALT | Modifiers::CONTROL;
            assert!(p.key(&up, false));
            let off = p.notes.pop().expect("shifted semicolon release");
            assert_eq!((off.note, off.on), (64, false));
            assert!(!p.active());
        }
        let mut p = Playing::default();
        let mut down = key(KeyState::Down, "a");
        down.code = Code::KeyQ; // Layout character chooses the legend; code owns its release.
        assert!(p.key(&down, true));
        assert_eq!(p.notes.pop().expect("layout attack").note, 48);
        let mut repeat = key(KeyState::Down, "q");
        repeat.code = Code::KeyQ;
        repeat.repeat = true;
        assert!(p.key(&repeat, true));
        assert!(p.notes.pop().is_none());
        repeat.modifiers = Modifiers::CONTROL;
        assert!(!p.key(&repeat, true), "modified repeats remain the host's");
        let mut up = key(KeyState::Up, "q");
        up.code = Code::KeyQ;
        up.modifiers = Modifiers::META;
        assert!(p.key(&up, false));
        assert!(!p.notes.pop().expect("recorded code release").on);
        assert!(!p.active());
        let mut unmapped = key(KeyState::Down, "q");
        unmapped.code = Code::KeyA;
        assert!(
            !p.key(&unmapped, true),
            "physical code does not replace legend mapping"
        );
        assert!(
            !p.key(&key(KeyState::Down, ":"), true),
            "a new shifted character remains unmapped"
        );
    }

    #[test]
    fn repeat_modifiers_and_takeover_never_retrigger_or_strand_notes() {
        let mut p = Playing::default();
        let mut e = key(KeyState::Down, "a");
        assert!(p.key(&e, true));
        assert!(p.key(&e, true));
        e.repeat = true;
        assert!(p.key(&e, true));
        assert!(p.notes.pop().is_some());
        assert!(p.notes.pop().is_none());
        let mut up = key(KeyState::Up, "a");
        up.modifiers = Modifiers::CONTROL;
        assert!(p.key(&up, false), "release precedes modifiers and takeover");
        assert!(!p.notes.pop().expect("off").on);
        p.key(&key(KeyState::Down, "s"), true);
        assert_eq!(p.notes.pop().expect("D attack").note, 50);
        p.shift_typing(true);
        assert!(!p.notes.pop().expect("octave shift releases typed note").on);
        assert!(p.key(&key(KeyState::Up, "s"), false));
        assert_eq!(p.owned, [0; 128]);
        assert!(p.key(&e, true));
        assert_eq!(
            p.owned, [0; 128],
            "repeat after takeover cannot start a phantom note"
        );
    }

    #[test]
    fn computer_keys_start_on_c_and_follow_the_piano_layout() {
        let mut p = Playing::default();
        for (character, expected) in [
            ("a", 48),
            ("w", 49),
            ("s", 50),
            ("e", 51),
            ("d", 52),
            ("f", 53),
            ("t", 54),
            ("g", 55),
            ("y", 56),
            ("h", 57),
            ("u", 58),
            ("j", 59),
            ("k", 60),
            ("o", 61),
            ("l", 62),
            ("p", 63),
            (";", 64),
        ] {
            assert!(p.key(&key(KeyState::Down, character), true));
            let on = p.notes.pop().expect("attack");
            assert_eq!((on.note, on.on), (expected, true), "{character}");
            assert!(p.key(&key(KeyState::Up, character), true));
            let off = p.notes.pop().expect("release");
            assert_eq!((off.note, off.on), (expected, false), "{character}");
        }
    }

    #[test]
    fn typing_and_visible_octaves_preserve_the_other_sources_notes() {
        let mut p = Playing::default();
        p.key(&key(KeyState::Down, "a"), true);
        assert_eq!(p.notes.pop().expect("type C").note, 48);
        p.press(50);
        assert_eq!(p.notes.pop().expect("pointer D").note, 50);
        p.shift(true);
        assert_eq!((p.root, p.typing_root), (53, 48));
        let off = p.notes.pop().expect("only pointer released");
        assert_eq!((off.note, off.on), (50, false));
        assert!(p.notes.pop().is_none(), "TYPE remains held");
        p.press(48);
        assert!(p.notes.pop().is_none(), "shared pitch already held");
        p.key(&key(KeyState::Down, "x"), true);
        assert_eq!((p.root, p.typing_root), (53, 60));
        assert!(p.notes.pop().is_none(), "pointer retains shared pitch");
        p.release_pointer();
        let off = p.notes.pop().expect("last pointer owner released");
        assert_eq!((off.note, off.on), (48, false));
        p.key(&key(KeyState::Up, "x"), true);
        p.key(&key(KeyState::Up, "a"), true);
        p.key(&key(KeyState::Down, "a"), true);
        assert_eq!(p.notes.pop().expect("new octave C").note, 60);
    }

    #[test]
    fn octave_shortcuts_shift_once_a_press_and_respect_takeover_and_modifiers() {
        for code in [Code::KeyZ, Code::Unidentified] {
            let mut p = Playing::default();
            let mut down = key(KeyState::Down, "z");
            down.code = code;
            assert!(!p.key(&down, false));
            assert_eq!(p.typing_root, 48);
            down.modifiers = Modifiers::META;
            assert!(!p.key(&down, true));
            assert_eq!(p.typing_root, 48);
            down.modifiers = Modifiers::empty();
            assert!(p.key(&down, true));
            assert_eq!(p.typing_root, 36);
            assert!(p.key(&down, true));
            down.repeat = true;
            assert!(p.key(&down, true));
            assert_eq!(p.typing_root, 36);
            let mut up = key(KeyState::Up, "Z");
            up.code = code;
            up.modifiers = Modifiers::CONTROL;
            assert!(p.key(&up, false), "release before takeover/modifiers");
            down.repeat = false;
            assert!(p.key(&down, true));
            assert_eq!(p.typing_root, 24);
            p.release();
            down.repeat = true;
            assert!(p.key(&down, true));
            assert_eq!(p.typing_root, 24, "no repeat after focus loss");
        }
    }

    #[test]
    fn an_unflagged_repeat_after_transposing_cannot_restart_a_held_key() {
        for code in [Code::KeyA, Code::Unidentified] {
            let mut p = Playing::default();
            let mut down = key(KeyState::Down, "a");
            down.code = code;
            p.key(&down, true);
            assert_eq!(p.notes.pop().expect("C3 attack").note, 48);
            p.press(48);
            p.key(&key(KeyState::Down, "x"), true);
            assert!(p.notes.pop().is_none(), "pointer retains C3");
            assert!(p.key(&down, true));
            assert!(p.notes.pop().is_none(), "unflagged repeat cannot attack C4");
            let mut up = key(KeyState::Up, "A");
            up.code = code;
            up.modifiers = Modifiers::SHIFT | Modifiers::CONTROL;
            assert!(p.key(&up, false));
            assert!(
                p.notes.pop().is_none(),
                "silent typed key cannot release pointer"
            );
            p.key(&down, true);
            assert_eq!(p.notes.pop().expect("fresh C4 attack").note, 60);
            p.release_pointer();
            let off = p.notes.pop().expect("pointer C3 released");
            assert_eq!((off.note, off.on), (48, false));
        }
    }

    #[test]
    fn an_out_of_range_key_stays_silent_until_released_after_shifting_down() {
        for code in [Code::KeyY, Code::Unidentified] {
            let mut p = Playing::default();
            for _ in 0..6 {
                p.shift_typing(true);
            }
            let mut down = key(KeyState::Down, "y");
            down.code = code;
            assert!(p.key(&down, true));
            assert!(p.notes.pop().is_none(), "MIDI 128 is silent");
            p.key(&key(KeyState::Down, "z"), true);
            assert!(p.key(&down, true));
            assert!(
                p.notes.pop().is_none(),
                "held out-of-range key cannot repeat into range"
            );
            let mut up = key(KeyState::Up, "y");
            up.code = code;
            assert!(p.key(&up, true));
            assert!(p.notes.pop().is_none());
            assert!(p.key(&down, true));
            assert_eq!(p.notes.pop().expect("fresh in-range attack").note, 116);
        }
    }

    #[test]
    fn typing_octaves_clamp_to_c_roots_and_out_of_range_notes_do_not_wrap() {
        let mut p = Playing::default();
        for _ in 0..20 {
            p.shift_typing(false);
        }
        assert_eq!(p.typing_root, 0);
        p.key(&key(KeyState::Down, "a"), true);
        assert_eq!(p.notes.pop().expect("lowest MIDI note").note, 0);
        p.shift_typing(false);
        assert!(
            p.notes.pop().is_none(),
            "limit does not release a held note"
        );
        p.shift_typing(true);
        assert!(!p.notes.pop().expect("shift releases original pitch").on);
        for _ in 0..20 {
            p.shift_typing(true);
        }
        assert_eq!(p.typing_root, 120);
        p.key(&key(KeyState::Down, "g"), true);
        assert_eq!(p.notes.pop().expect("highest MIDI note").note, 127);
        p.shift_typing(true);
        assert!(p.notes.pop().is_none());
        for character in ["y", "h", "k", ";"] {
            assert!(p.key(&key(KeyState::Down, character), true));
            assert!(p.notes.pop().is_none(), "out of range: {character}");
        }
        p.key(&key(KeyState::Up, "g"), true);
        assert!(!p.notes.pop().expect("highest note released").on);
        assert!(!p.active());
    }

    #[test]
    fn overflow_or_reset_cannot_resurrect_a_held_pointer_and_a_fresh_press_works() {
        let mut p = Playing::default();
        for _ in 0..crate::local_notes::CAPACITY {
            assert!(p.notes.note(60, true));
        }
        p.press(60);
        assert!(!p.dragging(), "overflow cleared the UI's captured note");
        for _ in 0..crate::local_notes::CAPACITY {
            p.notes.pop();
        }
        p.press(60);
        assert!(p.notes.pop().expect("fresh attack after overflow").on);
        p.notes.release();
        p.drag(Some(62));
        assert!(!p.dragging());
        assert!(
            p.notes.pop().is_none(),
            "a stale gesture does not restart after reset"
        );
        p.press(62);
        assert_eq!(p.notes.pop().expect("a new gesture still plays").note, 62);
    }
}
