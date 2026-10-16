//! A pack's drawings: a strict subset of SVG, read first by an XML parser that refuses a DTD
//! (and so every entity), and checked element by element before usvg sees it.
//!
//! Allowed: shapes, paths, groups, gradients, clipping, masks, patterns, `use` of the drawing's
//! own parts, a few filters, and pictures embedded as PNG data. Refused: scripts, foreign
//! content, animation, text (a pack's legends are its layout's, printed by the plug-in), style
//! sheets, markers, event attributes, and any reference outside the drawing (a file, a URL).
//! Then its cost: elements (each `use` counted as all it repeats), path segments, filters and
//! blurs, each within [`Limits`], and, when asked, a test drawing at the largest size the
//! editor shows it, against a time limit.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

use crate::{Limits, Report};

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// The elements a drawing may hold.
const ALLOWED: &[&str] = &[
    "svg",
    "g",
    "defs",
    "title",
    "desc",
    "metadata",
    "path",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polyline",
    "polygon",
    "linearGradient",
    "radialGradient",
    "stop",
    "clipPath",
    "mask",
    "pattern",
    "symbol",
    "use",
    "image",
    "filter",
    "feGaussianBlur",
    "feOffset",
    "feFlood",
    "feComposite",
    "feMerge",
    "feMergeNode",
    "feColorMatrix",
    "feBlend",
    "feDropShadow",
];

/// Why an element is refused, for those a creator might try.
fn refused(name: &str) -> &'static str {
    match name {
        "script" | "foreignObject" | "iframe" | "handler" | "listener" => {
            "code or foreign content: a pack holds no code"
        }
        "animate" | "animateMotion" | "animateTransform" | "animateColor" | "set" => {
            "animation: a pack's drawings are still (what moves is the plug-in's)"
        }
        "text" | "tspan" | "textPath" | "tref" | "font" | "font-face" => {
            "text: a pack's legends are its layout's, printed by the plug-in in the pack's fonts"
        }
        "style" => "a style sheet: give each element its own attributes",
        "marker" => "markers: draw each mark as its own shape",
        "a" | "view" | "switch" | "cursor" => "an interactive element",
        _ => "an element a pack's drawing may not hold",
    }
}

/// Whether a reference stays inside the drawing (`#name`).
fn internal(href: &str) -> bool {
    href.starts_with('#')
}

/// Checks every `url(...)` in an attribute's value: each must be `#name`.
fn urls_internal(value: &str) -> bool {
    let mut rest = value;
    while let Some(i) = rest.find("url(") {
        let after = rest[i + 4..].trim_start();
        let after = after.trim_start_matches(['\'', '"']);
        if !after.starts_with('#') {
            return false;
        }
        rest = &rest[i + 4..];
    }
    true
}

/// The drawing's text read strictly, its elements and attributes checked: `Err` with why not.
fn scan(text: &str, limits: &Limits) -> Result<(), String> {
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: u32::try_from(limits.svg_nodes * 4).unwrap_or(u32::MAX),
        },
    )
    .map_err(|e| format!("not well-formed XML without a DTD: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG_NS) {
        return Err("not an SVG document (its root is not `svg` in SVG's namespace)".into());
    }
    let mut ids: HashMap<&str, roxmltree::Node<'_, '_>> = HashMap::new();
    for node in doc.descendants() {
        if node.is_element() {
            if let Some(id) = node.attribute("id") {
                ids.insert(id, node);
            }
        } else if node.is_pi() {
            return Err("a processing instruction".into());
        }
    }
    for node in doc.descendants().filter(roxmltree::Node::is_element) {
        let tag = node.tag_name();
        let name = tag.name();
        // (Inside `metadata`, a licence's RDF in its own namespaces: not drawn, not checked.)
        if node
            .ancestors()
            .skip(1)
            .any(|a| a.tag_name().name() == "metadata")
        {
            continue;
        }
        if tag.namespace() != Some(SVG_NS) {
            return Err(format!("`{name}`: an element outside SVG's namespace"));
        }
        if !ALLOWED.contains(&name) {
            return Err(format!("`{name}`: {}", refused(name)));
        }
        for attr in node.attributes() {
            let an = attr.name();
            let ns = attr.namespace();
            if an.to_ascii_lowercase().starts_with("on") {
                return Err(format!(
                    "`{name}` has `{an}`, an event handler: a pack holds no code"
                ));
            }
            if an == "href" && (ns.is_none() || ns == Some(XLINK_NS)) {
                let v = attr.value().trim();
                let ok = if name == "image" {
                    v.starts_with("data:image/png;base64,")
                } else {
                    internal(v)
                };
                if !ok {
                    return Err(format!(
                        "`{name}` refers to `{}`: a drawing refers only to its own parts (and \
                         its pictures are PNG data inside it)",
                        v.chars().take(60).collect::<String>()
                    ));
                }
            } else if !urls_internal(attr.value()) {
                return Err(format!(
                    "`{name}`'s `{an}` refers outside the drawing: only `url(#name)` is allowed"
                ));
            }
            if an == "style" {
                let v = attr.value().to_ascii_lowercase();
                if v.contains("@import") || v.contains("expression(") {
                    return Err(format!("`{name}`'s style imports or computes"));
                }
            }
        }
    }
    // What the `use`s repeat: each element counted as many times as it is drawn.
    let mut memo: HashMap<roxmltree::NodeId, usize> = HashMap::new();
    let total = expanded(root, &ids, &mut memo, &mut Vec::new(), limits.svg_nodes)?;
    if total > limits.svg_nodes {
        return Err(format!(
            "{total} elements drawn (its `use`s counted as what they repeat), more than {}",
            limits.svg_nodes
        ));
    }
    Ok(())
}

/// How many elements `node` draws, its `use`s counted as what they repeat (`Err` on a loop or
/// past `most`).
fn expanded<'a, 'i>(
    node: roxmltree::Node<'a, 'i>,
    ids: &HashMap<&str, roxmltree::Node<'a, 'i>>,
    memo: &mut HashMap<roxmltree::NodeId, usize>,
    path: &mut Vec<roxmltree::NodeId>,
    most: usize,
) -> Result<usize, String> {
    if let Some(&n) = memo.get(&node.id()) {
        return Ok(n);
    }
    if path.contains(&node.id()) {
        return Err("a `use` that repeats itself".into());
    }
    path.push(node.id());
    let mut n = 1usize;
    for child in node.children().filter(roxmltree::Node::is_element) {
        n = n.saturating_add(expanded(child, ids, memo, path, most)?);
        if n > most {
            break;
        }
    }
    if node.tag_name().name() == "use" {
        let href = node
            .attribute("href")
            .or_else(|| node.attribute((XLINK_NS, "href")));
        if let Some(target) = href
            .and_then(|h| h.strip_prefix('#'))
            .and_then(|id| ids.get(id))
        {
            n = n.saturating_add(expanded(*target, ids, memo, path, most)?);
        }
    }
    path.pop();
    memo.insert(node.id(), n);
    Ok(n)
}

/// usvg's options for a pack's drawing: no file is ever read for it (a reference that the scan
/// let past, which none should, resolves to nothing), no font, and pictures as PNG data only.
pub fn options() -> usvg::Options<'static> {
    usvg::Options {
        resources_dir: None,
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|mime, data: Arc<Vec<u8>>, _| {
                (mime == "image/png").then_some(usvg::ImageKind::PNG(data))
            }),
            resolve_string: Box::new(|_, _| None),
        },
        ..usvg::Options::default()
    }
}

/// What a drawing's tree costs to draw.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cost {
    pub nodes: usize,
    pub segments: usize,
    pub filters: usize,
    pub blur: f32,
}

fn cost_of(group: &usvg::Group, cost: &mut Cost, pictures: &mut Vec<Arc<Vec<u8>>>) {
    cost.nodes += 1;
    for f in group.filters() {
        cost.filters += f.primitives().len();
        for p in f.primitives() {
            match p.kind() {
                usvg::filter::Kind::GaussianBlur(b) => {
                    cost.blur = cost.blur.max(b.std_dev_x().get()).max(b.std_dev_y().get());
                }
                usvg::filter::Kind::DropShadow(d) => {
                    cost.blur = cost.blur.max(d.std_dev_x().get()).max(d.std_dev_y().get());
                }
                _ => {}
            }
        }
    }
    if let Some(c) = group.clip_path() {
        cost_of(c.root(), cost, pictures);
    }
    if let Some(m) = group.mask() {
        cost_of(m.root(), cost, pictures);
    }
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => cost_of(g, cost, pictures),
            usvg::Node::Path(p) => {
                cost.nodes += 1;
                cost.segments += p.data().len();
                for paint in [
                    p.fill().map(usvg::Fill::paint),
                    p.stroke().map(usvg::Stroke::paint),
                ]
                .into_iter()
                .flatten()
                {
                    if let usvg::Paint::Pattern(pat) = paint {
                        cost_of(pat.root(), cost, pictures);
                    }
                }
            }
            usvg::Node::Image(i) => {
                cost.nodes += 1;
                if let usvg::ImageKind::PNG(data) = i.kind() {
                    pictures.push(data.clone());
                }
            }
            usvg::Node::Text(_) => cost.nodes += 1,
        }
    }
}

/// A pack's drawing read: its tree, within `limits` (`Err` with why not). The plug-in reads a
/// pack's drawings only through this.
pub fn parse(bytes: &[u8], limits: &Limits) -> Result<(usvg::Tree, Cost), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "not UTF-8 text".to_owned())?;
    scan(text, limits)?;
    let tree =
        usvg::Tree::from_str(text, &options()).map_err(|e| format!("cannot be read: {e}"))?;
    let mut cost = Cost::default();
    let mut pictures = Vec::new();
    cost_of(tree.root(), &mut cost, &mut pictures);
    if cost.nodes > limits.svg_nodes {
        return Err(format!(
            "{} elements, more than {}",
            cost.nodes, limits.svg_nodes
        ));
    }
    if cost.segments > limits.svg_segments {
        return Err(format!(
            "{} path segments, more than {}",
            cost.segments, limits.svg_segments
        ));
    }
    if cost.filters > limits.svg_filters {
        return Err(format!(
            "{} filter primitives, more than {}",
            cost.filters, limits.svg_filters
        ));
    }
    if cost.blur > limits.svg_blur {
        return Err(format!(
            "a blur of {}, more than {}",
            cost.blur, limits.svg_blur
        ));
    }
    for p in pictures {
        crate::picture::decode(&p, limits).map_err(|e| format!("an embedded picture: {e}"))?;
    }
    Ok((tree, cost))
}

/// Draws `tree` with its longer side `side` pixels: the time it took, in milliseconds.
pub fn test_draw(tree: &usvg::Tree, side: u32) -> Option<u64> {
    let size = tree.size();
    let k = side as f32 / size.width().max(size.height());
    let (w, h) = (
        (size.width() * k).ceil().max(1.0) as u32,
        (size.height() * k).ceil().max(1.0) as u32,
    );
    let mut pixmap = Pixmap::new(w, h)?;
    let t = Instant::now();
    resvg::render(tree, Transform::from_scale(k, k), &mut pixmap.as_mut());
    Some(t.elapsed().as_millis() as u64)
}

/// Checks a drawing of a pack, drawing it as a test if `test`.
pub fn check(path: &str, bytes: &[u8], limits: &Limits, test: bool, report: &mut Report) {
    match parse(bytes, limits) {
        Err(why) => report.reject("svg.refused", Some(path), why),
        Ok((tree, _)) if test => match test_draw(&tree, limits.svg_test_side) {
            Some(ms) if ms > limits.svg_test_ms => report.reject(
                "svg.slow",
                Some(path),
                format!(
                    "drawn {} pixels across in {ms} ms, more than {} ms",
                    limits.svg_test_side, limits.svg_test_ms
                ),
            ),
            Some(_) => {}
            None => report.reject("svg.size", Some(path), "it has no size to draw at"),
        },
        Ok(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svg(body: &str) -> String {
        format!(
            "<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' \
             viewBox='0 0 100 100' width='100' height='100'>{body}</svg>"
        )
    }

    fn refused(body: &str) -> String {
        parse(svg(body).as_bytes(), &Limits::default())
            .err()
            .unwrap_or_else(|| panic!("accepted: {body}"))
    }

    #[test]
    fn a_plain_drawing_is_read() {
        let s = svg(
            "<defs><linearGradient id='g'><stop offset='0' stop-color='#000'/></linearGradient>\
             <filter id='f'><feGaussianBlur stdDeviation='2'/></filter></defs>\
             <rect width='100' height='100' fill='url(#g)' filter='url(#f)'/>\
             <circle cx='50' cy='50' r='20' style='fill:url(#g)'/>",
        );
        let (_, cost) = parse(s.as_bytes(), &Limits::default()).expect("read");
        assert_eq!(cost.filters, 1);
        assert_eq!(cost.blur, 2.0);
    }

    #[test]
    fn code_animation_and_text_are_refused() {
        assert!(refused("<script>alert(1)</script>").contains("no code"));
        assert!(refused("<foreignObject><div/></foreignObject>").contains("no code"));
        assert!(refused("<rect width='1' height='1' onclick='x()'/>").contains("event handler"));
        assert!(refused("<rect><animate attributeName='x'/></rect>").contains("animation"));
        assert!(refused("<text>MOOG</text>").contains("legends"));
        assert!(refused("<style>rect{fill:red}</style>").contains("style sheet"));
    }

    #[test]
    fn references_outside_the_drawing_are_refused() {
        assert!(refused("<image href='/etc/passwd' width='1' height='1'/>").contains("refers"));
        assert!(
            refused("<image xlink:href='https://example.com/x.png' width='1' height='1'/>")
                .contains("refers")
        );
        assert!(refused("<image href='data:image/svg+xml;base64,AAAA'/>").contains("refers"));
        assert!(refused("<use href='other.svg#a'/>").contains("refers"));
        assert!(refused("<rect fill='url(other.svg#g)'/>").contains("outside"));
        assert!(refused("<rect style=\"fill:url('file:///x')\"/>").contains("outside"));
    }

    #[test]
    fn a_dtd_and_its_entities_are_refused() {
        let s = "<?xml version='1.0'?><!DOCTYPE svg [<!ENTITY a 'aaaaaaaaaa'><!ENTITY b '&a;&a;&a;&a;'>]>\
                 <svg xmlns='http://www.w3.org/2000/svg'>&b;</svg>";
        assert!(parse(s.as_bytes(), &Limits::default()).is_err());
    }

    /// Ten `use`s of ten `use`s of ... : a few hundred bytes drawing millions of shapes.
    #[test]
    fn uses_that_multiply_are_counted_as_drawn() {
        let mut body = String::from("<defs><rect id='l0' width='1' height='1'/>");
        for k in 1..8 {
            body.push_str(&format!("<g id='l{k}'>"));
            for _ in 0..10 {
                body.push_str(&format!("<use href='#l{}'/>", k - 1));
            }
            body.push_str("</g>");
        }
        body.push_str("</defs><use href='#l7'/>");
        assert!(refused(&body).contains("elements drawn"));
        assert!(refused("<g id='a'><use href='#a'/></g>").contains("repeats itself"));
    }

    #[test]
    fn costs_past_their_limits_are_refused() {
        let mut d = String::from("M0 0");
        for i in 0..2000 {
            d.push_str(&format!(" L{} {}", i % 100, i % 97));
        }
        let limits = Limits {
            svg_segments: 1000,
            ..Limits::default()
        };
        let s = svg(&format!("<path d='{d}'/>"));
        assert!(
            parse(s.as_bytes(), &limits)
                .unwrap_err()
                .contains("segments")
        );
        let blur = svg(
            "<filter id='f'><feGaussianBlur stdDeviation='500'/></filter><rect width='9' height='9' filter='url(#f)'/>",
        );
        assert!(
            parse(blur.as_bytes(), &Limits::default())
                .unwrap_err()
                .contains("blur")
        );
    }

    #[test]
    fn a_slow_drawing_is_refused_by_its_test() {
        let s = svg(
            "<filter id='f'><feGaussianBlur stdDeviation='60'/></filter>\
             <rect width='100' height='100' filter='url(#f)'/>",
        );
        let mut r = Report::default();
        let limits = Limits {
            svg_test_ms: 0,
            svg_test_side: 2000,
            ..Limits::default()
        };
        check("art/slow.svg", s.as_bytes(), &limits, true, &mut r);
        assert!(r.has("svg.slow"), "{r}");
    }
}
