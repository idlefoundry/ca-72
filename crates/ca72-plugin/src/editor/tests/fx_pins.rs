use super::*;
use crate::fx_view::{Effects, fake::Fake};
use crate::lfo::{Source, Target as T};

fn pinned() -> (Editing, Arc<Fake>) {
    let (mut e, fake) = editing_effects();
    fake.parameters(0, &["Mix", "Rate", "Depth", "Tone", "Width"]);
    fake.parameters(1, &["Mix", "Decay", "Damping", "Size", "Pre-delay"]);
    fake.pin(1, 100).unwrap();
    fake.pin(1, 101).unwrap();
    fake.take();
    e.fx_view.poll(Instant::now() + Duration::from_secs(1));
    lower_open(&mut e, Rail::Effects, BarTarget::Fx);
    poll(&mut e);
    (e, fake)
}

fn poll(e: &mut Editing) {
    // Polling clocks in the view belong to wall time, not the frame's clock.
    std::thread::sleep(Duration::from_millis(110));
    e.draw();
}

fn open_lfos(e: &mut Editing) {
    let p = at(
        e,
        strip::centre(Rail::Effects, StripTarget::Bar(BarTarget::Lfo)),
    );
    click(e, p);
    e.draw();
    assert!(e.page_shows());
}

fn pin(e: &Editing, target: usize) -> Point {
    at(e, e.fx_pin_at(target).unwrap())
}

fn whole(e: &mut Editing) {
    e.draw();
    let mut all = None;
    compose(
        &mut all,
        &e.renderer,
        e.strip.frame(),
        lower_of(e),
        None,
        (
            e.page_shows().then(|| e.lfo.frame()),
            e.macros_show()
                .then(|| (e.macros.frame(), e.renderer.left_rect())),
            &e.modulation,
        ),
        None,
        true,
        (e.physical_width, e.physical_height()),
    );
    assert!(
        e.composed == all,
        "the editor drawn in parts equals the editor drawn whole"
    );
}

#[test]
fn a_pin_turns_and_steps_and_follows_its_effects_window() {
    let (mut e, fake) = pinned();
    assert_eq!(e.fx_scene.depth(), fx_ui::PIN_DEPTH);
    let p = pin(&e, 0);
    go(&mut e, p);
    press(&mut e);
    assert!(e.fx_pin_drag.is_some());
    go(
        &mut e,
        Point {
            x: p.x,
            y: p.y - 60.0,
        },
    );
    release(&mut e);
    assert!((fake.pins.lock().unwrap()[0].value - 0.60).abs() < 1e-8);
    whole(&mut e);
    fake.pins.lock().unwrap()[0].steps = 4;
    poll(&mut e);
    let p = pin(&e, 0);
    go(&mut e, p);
    press(&mut e);
    go(
        &mut e,
        Point {
            x: p.x,
            y: p.y - 30.0,
        },
    );
    release(&mut e);
    assert_eq!(fake.pins.lock().unwrap()[0].value, 0.75);
    {
        let mut pins = fake.pins.lock().unwrap();
        pins[1].value = 0.63;
        pins[1].plain = Some(3.6);
    }
    poll(&mut e);
    assert_eq!(e.fx_scene.pins[1].value, 0.63);
    poll(&mut e);
    assert_eq!(e.fx_view.pin_text(1), "3.60 s");
    whole(&mut e);
}

#[test]
fn a_pin_takes_cords_rings_popups_tips_and_its_own_midi_learn() {
    let (mut e, fake) = pinned();
    open_lfos(&mut e);
    let jack = at(&e, lfo_ui::places(0).jack);
    let to = pin(&e, 1);
    go(&mut e, jack);
    press(&mut e);
    go(&mut e, to);
    e.draw();
    assert_eq!(
        e.mod_scene.cord.as_ref().unwrap().over,
        Some((e.fx_pin_at(1).unwrap(), art::STD.skirt))
    );
    whole(&mut e);
    release(&mut e);
    assert_eq!(
        e.params.lfo_patch.depth(Source::Lfo(0), T::Fx(1)),
        Some(0.25)
    );
    e.params.lfo_patch.set_depth(Source::Lfo(0), T::Fx(1), 0.30);
    e.params.lfo_patch.set_name(1, "SPACE");
    e.strip_press(StripTarget::Bar(BarTarget::Macro));
    e.draw();
    let macro_jack = at(&e, macro_ui::jack_at(1));
    go(&mut e, macro_jack);
    press(&mut e);
    go(&mut e, to);
    release(&mut e);
    assert_eq!(
        e.params.lfo_patch.depth(Source::Macro(1), T::Fx(1)),
        Some(0.25)
    );
    e.params
        .lfo_patch
        .set_depth(Source::Macro(1), T::Fx(1), 0.4);
    e.modulating.open = false;
    e.draw();
    go(&mut e, to);
    e.draw();
    let ring = e
        .mod_scene
        .knobs
        .iter()
        .find(|k| k.control == crate::editor::fx_pins::FIRST + 1)
        .unwrap();
    assert_eq!(ring.rings.len(), 2);
    assert!(ring.rings[1].one_way);
    let popup = e.mod_scene.popup.as_ref().unwrap();
    assert_eq!(popup.dials[1].label, "SPACE");
    let (_, dials) = ca72_panel::modulation::popup_layout(popup);
    let dial = at(&e, dials[0].0);
    go(&mut e, dial);
    e.draw();
    assert!(
        e.mod_scene.popup.is_some(),
        "the pop-up keeps the pointer above the lower page"
    );
    press(&mut e);
    go(
        &mut e,
        Point {
            x: dial.x,
            y: dial.y - 24.0,
        },
    );
    release(&mut e);
    assert!((e.params.lfo_patch.depth(Source::Lfo(0), T::Fx(1)).unwrap() - 0.4).abs() < 1e-6);
    go(&mut e, to);
    let tip = e.fx_pin_tip().unwrap().0;
    assert!(tip.contains("DECAY, HALL REVERB: 2.40 s"), "{tip}");
    assert!(tip.contains("SPACE +40 %"), "{tip}");
    whole(&mut e);
    right_click(&mut e, to);
    let items = menu_items(&e);
    assert!(items.iter().any(|(t, _)| t.contains("LFO 1")));
    assert!(items.iter().any(|(t, _)| t.contains("SPACE")));
    assert!(
        items.iter().position(|(t, _)| t == "UNPIN")
            < items.iter().position(|(t, _)| t == "MIDI LEARN")
    );
    menu_item(&mut e, "MIDI LEARN");
    assert_eq!(e.params.midi_map.armed(), crate::learn::index("fx2"));
    e.learning.cancel(&e.params.midi_map);
    let list = e.learning.list_scene(&e.params.midi_map);
    assert!(list.rows.iter().any(|r| r.name == "DECAY, HALL REVERB"));
    {
        let (mut plugin, _, _) = editing();
        assert!(plugin.fx.is_none());
        assert!(
            plugin
                .learning
                .list_scene(&plugin.params.midi_map)
                .rows
                .iter()
                .all(|r| !r.name.starts_with("EFFECT PIN"))
        );
    }
    {
        let mut pins = fake.pins.lock().unwrap();
        pins[1].live = false;
        pins[1].missing = true;
    }
    poll(&mut e);
    go(&mut e, to);
    e.draw();
    assert_eq!(e.fx_scene.pins[1].name, "MISSING");
    assert!(
        e.mod_scene
            .knobs
            .iter()
            .find(|k| k.control == crate::editor::fx_pins::FIRST + 1)
            .unwrap()
            .rings
            .iter()
            .all(|r| r.dim)
    );
    let old = e.fx_view.pin_info(1).unwrap().value;
    go(&mut e, to);
    press(&mut e);
    go(
        &mut e,
        Point {
            x: to.x,
            y: to.y - 80.0,
        },
    );
    release(&mut e);
    assert_eq!(e.fx_view.pin_info(1).unwrap().value, old);
    open_lfos(&mut e);
    let another = at(&e, lfo_ui::places(2).jack);
    go(&mut e, another);
    press(&mut e);
    go(&mut e, to);
    e.draw();
    assert!(e.mod_scene.cord.as_ref().unwrap().over.is_none());
    release(&mut e);
    assert!(e.params.lfo_patch.depth(Source::Lfo(2), T::Fx(1)).is_none());
    assert!(e.params.lfo_patch.depth(Source::Lfo(0), T::Fx(1)).is_some());
    right_click(&mut e, to);
    menu_item(&mut e, "UNPIN");
    effects_done(&mut e, &fake, 1);
    poll(&mut e);
    assert!(e.fx_view.pin_info(1).is_none());
    whole(&mut e);
}

#[test]
fn pins_keep_their_targets_when_a_slot_moves_and_the_first_free_place_opens_parameters() {
    let (mut e, fake) = pinned();
    fake.move_to(1, 0).unwrap();
    fake.take();
    poll(&mut e);
    assert_eq!(e.fx_view.pin_info(1).unwrap().slot, 0);
    let plus = at(&e, {
        let (x, y, w, h) = fx_ui::pin_screen(0, 2);
        (x + w / 2.0, y + h / 2.0)
    });
    click(&mut e, plus);
    poll(&mut e);
    assert!(e.fx_view.list_open());
    assert!(
        e.fx_scene
            .list
            .as_ref()
            .unwrap()
            .title
            .contains("HALL REVERB")
    );
    assert!(e.holds_keys());
    keyed(&mut e, Key::Escape);
    assert!(!e.fx_view.list_open());
    for t in [0, 1] {
        e.fx_view.unpin(t);
    }
    effects_done(&mut e, &fake, 2);
    poll(&mut e);
    assert_eq!(e.fx_scene.depth(), fx_ui::PIN_DEPTH);
    whole(&mut e);
}

#[test]
fn replacing_a_pin_keeps_its_sources_and_midi_and_hover_shows_its_value() {
    let (mut e, fake) = pinned();
    let target = 1;
    e.params
        .lfo_patch
        .assign(Source::Lfo(0), T::Fx(target), 0.3)
        .unwrap();
    e.params
        .lfo_patch
        .assign(Source::Macro(1), T::Fx(target), 0.4)
        .unwrap();
    let midi = crate::learn::index("fx2").unwrap();
    let cc = crate::learn::Cc { channel: 2, cc: 74 };
    e.params.midi_map.assign(midi, cc);
    let p = pin(&e, target);
    go(&mut e, p);
    poll(&mut e);
    assert_eq!(e.fx_scene.pins[1].text.as_deref(), Some("2.40 s"));
    press(&mut e);
    e.draw();
    assert!(e.fx_scene.pins[1].text.is_some(), "turning shows the value");
    release(&mut e);
    right_click(&mut e, p);
    assert!(
        menu_items(&e).iter().position(|(s, _)| s == "REPLACE…")
            < menu_items(&e).iter().position(|(s, _)| s == "UNPIN")
    );
    menu_item(&mut e, "REPLACE…");
    poll(&mut e);
    assert_eq!(e.fx_scene.list.as_ref().unwrap().title, "REPLACE DECAY");
    keyed(&mut e, Key::Character("size".into()));
    poll(&mut e);
    keyed(&mut e, Key::Enter);
    effects_done(&mut e, &fake, 1);
    poll(&mut e);
    assert_eq!(e.fx_view.pin_info(target).unwrap().name, "Size");
    assert_eq!(
        e.params.lfo_patch.depth(Source::Lfo(0), T::Fx(target)),
        Some(0.3)
    );
    assert_eq!(
        e.params.lfo_patch.depth(Source::Macro(1), T::Fx(target)),
        Some(0.4)
    );
    assert_eq!(e.params.midi_map.assignment(midi), Some(cc));
    e.fx_view.close_list();
    whole(&mut e);
}

/// The real editor's composed frames, using the fake chain rather than a mock renderer.
#[test]
#[ignore = "writes review images"]
fn the_fx_pin_pngs() {
    let Some(out) = std::env::var_os("CA72_FX_PIN_PNG").map(std::path::PathBuf::from) else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    for (scale, backing, size) in [
        (0.44, 1.0, "0.44"),
        (1.0, 1.0, "1.0"),
        (0.3875, 2.0, "mac-0.3875-2x"),
    ] {
        for (skin, look) in [(Skin::Worn, "worn"), (Skin::Drawn, "drawn")] {
            let (mut e, fake) = pinned();
            fake.slots.lock().unwrap().push(crate::fx_view::SlotInfo {
                name: "Tape Delay".into(),
                ..Default::default()
            });
            fake.parameters(2, &["Feedback", "Time", "Mix"]);
            fake.pin(2, 200).unwrap();
            {
                let mut pins = fake.pins.lock().unwrap();
                pins[2].value = 0.45;
                pins[2].plain = Some(45.0);
            }
            poll(&mut e);
            let width = (art::W * scale).round() as u32;
            e.fit(backing, width, (f64::from(width) * backing).round() as u32);
            e.set_look(skin);
            e.params
                .lfo_patch
                .assign(Source::Lfo(0), T::Fx(1), 0.30)
                .unwrap();
            e.params.lfo_patch.set_name(1, "SPACE");
            e.params
                .lfo_patch
                .assign(Source::Macro(1), T::Fx(2), 0.40)
                .unwrap();
            open_lfos(&mut e);
            let save = |e: &mut Editing, state: &str| {
                window_frame(e)
                    .save_png(out.join(format!("{look}-{size}-{state}.png")))
                    .unwrap();
            };
            let pins = fake.pins.lock().unwrap().clone();
            fake.pins.lock().unwrap().clear();
            poll(&mut e);
            save(&mut e, "new-chain");
            fake.touched.lock().unwrap().insert(0, vec![1]);
            poll(&mut e);
            poll(&mut e);
            save(&mut e, "offer");
            *fake.pins.lock().unwrap() = pins;
            fake.slots.lock().unwrap()[1].name = "ValhallaRoom".into();
            fake.parameters(1, &["Mix", "Decay", "Depth", "Size", "Pre-delay"]);
            fake.pin(1, 103).unwrap();
            fake.touched
                .lock()
                .unwrap()
                .insert(1, vec![102, 101, 103, 100]);
            poll(&mut e);
            poll(&mut e);
            save(&mut e, "three-pins-last-offer");
            let (x, y, w, h) = fx_ui::pin_screen(1, 1);
            let field = at(&e, (x + w / 2.0, y + h / 2.0));
            go(&mut e, field);
            save(&mut e, "swap-hover");
            let value = pin(&e, 1);
            go(&mut e, value);
            press(&mut e);
            save(&mut e, "value-while-turning");
            release(&mut e);
            e.fx_view.open_repin(1);
            poll(&mut e);
            let list = e.fx_scene.list.as_ref().unwrap();
            let (x, y, _, _) = fx_ui::list_rect(list);
            let title = at(&e, (x + 20.0, y + 20.0));
            go(&mut e, title);
            save(&mut e, "replace-list");
            e.fx_view.close_list();
            let jack = at(&e, lfo_ui::places(0).jack);
            let to = pin(&e, 1);
            go(&mut e, jack);
            press(&mut e);
            go(&mut e, to);
            save(&mut e, "cord");
            e.modulating.patching = None;
            let p = pin(&e, 1);
            go(&mut e, p);
            save(&mut e, "popup");
            right_click(&mut e, p);
            save(&mut e, "menu");
            menu_item(&mut e, "MIDI LEARN");
            save(&mut e, "learn");
            e.learning.cancel(&e.params.midi_map);
            e.learning.close_menu();
            e.fx_view.open_parameters(1);
            poll(&mut e);
            let list = e.fx_scene.list.as_ref().unwrap();
            let (x, y, _, _) = fx_ui::list_rect(list);
            let title = at(&e, (x + 20.0, y + 20.0));
            go(&mut e, title);
            save(&mut e, "just-touched-list");
            keyed(&mut e, Key::Character("d".into()));
            save(&mut e, "find");
            e.fx_view.close_list();
            fake.slots.lock().unwrap()[1].health = crate::fx_view::Health::Crashed;
            {
                let mut pins = fake.pins.lock().unwrap();
                pins[0].live = false;
                pins[1].live = false;
            }
            poll(&mut e);
            save(&mut e, "dimmed");
            fake.slots.lock().unwrap()[1].health = crate::fx_view::Health::Running;
            {
                let mut pins = fake.pins.lock().unwrap();
                pins[0].live = true;
                pins[1].missing = true;
            }
            poll(&mut e);
            save(&mut e, "missing");
            {
                let mut pins = fake.pins.lock().unwrap();
                pins[1].missing = false;
                pins[1].live = true;
            }
            poll(&mut e);
            e.set_lettering(Lettering::Large);
            if let Some(fx) = &mut e.fx {
                fx.set_look(skin, Lettering::Large);
            }
            save(&mut e, "large");
        }
    }
}
