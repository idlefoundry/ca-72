//! A file of 0.1's presets (the factory's format) translated as 0.1's sessions are when they
//! open (`compat::translate_preset`, decisions.md R-TRANSLATE), written in the same format to
//! stdout, its presets in its order, each value by ID: how `sounds/retired.toml` was made from
//! 0.1.5's factory file. `cargo run -p ca72-plugin --example translate_0_1 -- <file>
//! [<name>=<id>=<value>,...]`: the second argument keeps values the translation cannot give
//! (the two presets whose filter's pitch R42 retuned through CUTOFF).

use ca72_plugin::compat::translate_preset;
use ca72_plugin::library::parse_factory;
use toml::Value;

fn number(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 1e6 {
        Value::Integer(x as i64)
    } else {
        Value::Float((x * 1e6).round() / 1e6)
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let file = args.next().expect("a file of presets");
    let kept: Vec<(String, String, f64)> = args
        .next()
        .unwrap_or_default()
        .split(',')
        .filter(|x| !x.is_empty())
        .map(|x| {
            let mut p = x.splitn(3, '=');
            let (n, k, v) = (p.next(), p.next(), p.next());
            (
                n.expect("a name").to_owned(),
                k.expect("an ID").to_owned(),
                v.expect("a value").parse().expect("a number"),
            )
        })
        .collect();
    let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{file}: {e}"));
    let presets = parse_factory(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
    let mut out = String::new();
    for mut s in presets {
        translate_preset(&mut s.values);
        for (n, k, v) in &kept {
            if *n == s.name
                && let Some(x) = s.values.iter_mut().find(|(id, _)| id == k)
            {
                x.1 = *v;
            }
        }
        s.values.sort_by(|a, b| a.0.cmp(&b.0));
        out.push_str("\n[[preset]]\n");
        out.push_str(&format!("name = {}\n", Value::String(s.name.clone())));
        if !s.description.is_empty() {
            out.push_str(&format!(
                "description = {}\n",
                Value::String(s.description.clone())
            ));
        }
        let tags: Vec<Value> = s.tags.iter().cloned().map(Value::String).collect();
        out.push_str(&format!(
            "tags = {}\n\n[preset.values]\n",
            Value::Array(tags)
        ));
        for (k, v) in &s.values {
            out.push_str(&format!("{k} = {}\n", number(*v)));
        }
    }
    print!("{out}");
}
