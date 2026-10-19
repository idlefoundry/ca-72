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

/// Why a drawing is refused: its finding's code (`svg.refused`, or `svg.picture` for a picture
/// it embeds) and why.
type Refusal = (&'static str, String);

/// The drawing's text read strictly, its elements and attributes checked, then the pictures it
/// embeds: `Err` with why not.
fn scan(text: &str, limits: &Limits) -> Result<(), Refusal> {
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: u32::try_from(limits.svg_nodes * 4).unwrap_or(u32::MAX),
        },
    )
    .map_err(|e| {
        (
            "svg.refused",
            format!("not well-formed XML without a DTD: {e}"),
        )
    })?;
    parts(&doc, limits).map_err(|why| ("svg.refused", why))?;
    pictures(&doc, limits).map_err(|why| ("svg.picture", why))
}

/// The drawing's elements and attributes checked: `Err` with why not.
fn parts(doc: &roxmltree::Document<'_>, limits: &Limits) -> Result<(), String> {
    let root = doc.root_element();
    if root.tag_name().name() != "svg" || root.tag_name().namespace() != Some(SVG_NS) {
        return Err("not an SVG document (its root is not `svg` in SVG's namespace)".into());
    }
    // The ids references resolve to: SVG's elements outside `metadata`, as usvg keeps them;
    // one given twice, or one inside `metadata`, refused (either would let the count follow
    // another element than the one usvg draws).
    let mut ids: HashMap<&str, roxmltree::Node<'_, '_>> = HashMap::new();
    for node in doc.descendants() {
        if node.is_pi() {
            return Err("a processing instruction".into());
        }
        if !node.is_element() {
            continue;
        }
        // Only the plain `id`, as usvg reads it; one in another namespace (`xml:id`) refused,
        // so that the count and usvg never resolve a name to different elements.
        if node
            .attributes()
            .any(|a| a.name() == "id" && a.namespace().is_some())
        {
            return Err("an `id` in a namespace (`xml:id`): give elements a plain `id`".into());
        }
        let Some(id) = node.attribute("id") else {
            continue;
        };
        let in_metadata = node
            .ancestors()
            .skip(1)
            .any(|a| a.tag_name().name() == "metadata");
        if in_metadata || node.tag_name().namespace() != Some(SVG_NS) {
            return Err(format!(
                "`{id}`: an id on an element that is not SVG's, or inside `metadata`"
            ));
        }
        if ids.insert(id, node).is_some() {
            return Err(format!("`{id}`: an id given to two elements"));
        }
    }
    for node in doc.descendants().filter(roxmltree::Node::is_element) {
        let tag = node.tag_name();
        let name = tag.name();
        // A style sheet or a script by its name in any namespace, anywhere: usvg reads every
        // `style` element it finds, wherever it is.
        if matches!(name, "style" | "script") {
            return Err(format!("`{name}`: {}", refused(name)));
        }
        // Inside `metadata`, a licence's RDF in its own namespaces, not drawn: nothing of SVG's
        // may hide there.
        if node
            .ancestors()
            .skip(1)
            .any(|a| a.tag_name().name() == "metadata")
        {
            if tag.namespace() == Some(SVG_NS) {
                return Err(format!(
                    "`{name}` inside `metadata`: only other namespaces' data"
                ));
            }
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
                // (An `<image>`'s, its picture, checked with the pictures.)
                let v = attr.value().trim();
                if name != "image" && !internal(v) {
                    return Err(format!(
                        "`{name}` refers to `{}`: a drawing refers only to its own parts",
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
    // What the references repeat: each element counted as many times as it is drawn or used.
    let mut memo: HashMap<roxmltree::NodeId, usize> = HashMap::new();
    let total = expanded(root, &ids, &mut memo, &mut Vec::new(), limits.svg_nodes)?;
    if total > limits.svg_nodes {
        return Err(format!(
            "{total} elements drawn (each reference, a `use` or a `url(#...)`, counted as what it \
             repeats), more than {}",
            limits.svg_nodes
        ));
    }
    Ok(())
}

/// Checks the pictures a drawing embeds as a pack's own pictures are checked: each `<image>`'s
/// `href` PNG data inside the drawing (`data:image/png;base64,...`), read by usvg's own parser of
/// `data:` URLs, and decoded whole within the pictures' limits, all of them together within the
/// pack's pixels (each refused from its header, before it is decoded, past what is left).
fn pictures(doc: &roxmltree::Document<'_>, limits: &Limits) -> Result<(), String> {
    let mut left = limits.picture_pixels;
    let images = doc.descendants().filter(|n| {
        n.is_element() && n.tag_name().name() == "image" && n.tag_name().namespace() == Some(SVG_NS)
    });
    for node in images {
        let hrefs = node.attributes().filter(|a| {
            a.name() == "href" && (a.namespace().is_none() || a.namespace() == Some(XLINK_NS))
        });
        for attr in hrefs {
            let v = attr.value().trim();
            let shown = v.chars().take(40).collect::<String>();
            if !v.starts_with("data:") {
                return Err(format!(
                    "`image` refers to `{shown}`, outside the drawing (a file or a URL): embed \
                     the picture in it as PNG data, `data:image/png;base64,...`"
                ));
            }
            if !v.starts_with("data:image/png;base64,") {
                return Err(format!(
                    "`image` holds `{shown}...`: a drawing's pictures are PNG, embedded as \
                     `data:image/png;base64,...`"
                ));
            }
            let data = data_url::DataUrl::process(v)
                .ok()
                .and_then(|url| url.decode_to_vec().ok())
                .ok_or_else(|| "an embedded picture whose base64 cannot be read".to_owned())?
                .0;
            let (w, h) = crate::picture::decode_within(&data, limits, left)
                .map_err(|e| format!("an embedded picture: {e}"))?;
            left -= u64::from(w) * u64::from(h);
        }
    }
    Ok(())
}

/// The ids an element refers to: its `href` (a `use`'s part, a gradient's or a pattern's
/// template) and each `url(#name)` in its attributes (a fill, a stroke, a clip, a mask, a
/// filter, its style).
fn references<'a>(node: roxmltree::Node<'a, '_>) -> Vec<&'a str> {
    let mut out = Vec::new();
    for attr in node.attributes() {
        let v = attr.value();
        if attr.name() == "href" {
            if let Some(id) = v.trim().strip_prefix('#') {
                out.push(id);
            }
            continue;
        }
        let mut rest = v;
        while let Some(i) = rest.find("url(") {
            let after = rest[i + 4..].trim_start().trim_start_matches(['\'', '"']);
            if let Some(id) = after.strip_prefix('#') {
                let end = id
                    .find(|c: char| c == ')' || c == '\'' || c == '"' || c.is_whitespace())
                    .unwrap_or(id.len());
                out.push(&id[..end]);
            }
            rest = &rest[i + 4..];
        }
    }
    out
}

/// How many elements `node` draws, each of its references counted as what it repeats (`Err` on
/// a loop or past `most`).
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
        return Err("a reference that repeats itself (a `use` or a `url(#...)`)".into());
    }
    path.push(node.id());
    let mut n = 1usize;
    for child in node.children().filter(roxmltree::Node::is_element) {
        n = n.saturating_add(expanded(child, ids, memo, path, most)?);
        if n > most {
            break;
        }
    }
    for id in references(node) {
        if n > most {
            break;
        }
        if let Some(target) = ids.get(id) {
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
    /// A pattern's tiles over what it fills, all its uses together, and the elements those
    /// tiles draw (each tile draws the pattern's).
    pub tiles: f64,
    pub tile_nodes: f64,
    /// At the test drawing's size: the pixel rows the edges of every path cross (what drawing
    /// a path costs, more than its segments' count: a long line crosses many), and the pixels
    /// painted (fills, strokes, pictures, and each layer a group draws into: a filter's region
    /// once a primitive, a clip's, a mask's, a group's opacity).
    pub edges: f64,
    pub area: f64,
    /// What drawing it costs, in pixel-equivalents at the test drawing's size: each pixel row an
    /// edge crosses [`EDGE`], each pixel painted 1, each pixel of a layer [`LAYER`], of a filter's
    /// region [`FILTER`] a primitive, each of an embedded picture's own pixels [`DECODE`]
    /// (measured on an M4 Pro: about 1.6 ms a million).
    pub work: f64,
    /// The pixels of the pictures embedded in it, decoded each time it is drawn, each counted as
    /// often as it is drawn (counted with the pack's own).
    pub pixels: u64,
}

/// What an edge's pixel row, a layer's pixel, a filter primitive's pixel and an embedded
/// picture's pixel decoded cost against a pixel painted (measured, decisions.md R-SKIN-PACKS).
pub const EDGE: f64 = 100.0;
pub const LAYER: f64 = 6.0;
pub const FILTER: f64 = 10.0;
pub const DECODE: f64 = 5.0;

impl Cost {
    /// Why it is past `limits`, if it is.
    fn over(&self, limits: &Limits) -> Result<(), String> {
        if self.nodes > limits.svg_nodes {
            return Err(format!("more than {} elements drawn", limits.svg_nodes));
        }
        if self.segments > limits.svg_segments {
            return Err(format!("more than {} path segments", limits.svg_segments));
        }
        if self.filters > limits.svg_filters {
            return Err(format!(
                "more than {} filter primitives",
                limits.svg_filters
            ));
        }
        if self.blur > limits.svg_blur {
            return Err(format!(
                "a blur of {}, more than {}",
                self.blur, limits.svg_blur
            ));
        }
        if self.tiles > limits.svg_tiles as f64 {
            return Err(format!("a pattern's tiles, more than {}", limits.svg_tiles));
        }
        if self.tile_nodes > limits.svg_nodes as f64 {
            return Err(format!(
                "more than {} elements drawn in a pattern's tiles",
                limits.svg_nodes
            ));
        }
        if self.work > limits.svg_work {
            return Err(format!(
                "drawing it costs about {:.0} million pixel-equivalents (its edges {:.1} million \
                 pixel rows, {:.0} million pixels painted), more than {:.0} million",
                self.work / 1e6,
                self.edges / 1e6,
                self.area / 1e6,
                limits.svg_work / 1e6
            ));
        }
        Ok(())
    }
}

/// The pixel rows a path's edges cross and its length in pixels, `k` pixels a unit, its points
/// placed by `t` (a curve's by its control points' extent and polygon, which hold it).
fn edges_of(path: &resvg::tiny_skia::Path, t: Transform, k: f64) -> (f64, f64, usize) {
    use resvg::tiny_skia::{PathSegment, Point};
    let at = |mut p: Point| {
        t.map_point(&mut p);
        (f64::from(p.x), f64::from(p.y))
    };
    let (mut rows, mut length, mut count) = (0.0, 0.0, 0usize);
    let (mut last, mut start) = ((0.0, 0.0), (0.0, 0.0));
    for seg in path.segments() {
        let ps: Vec<(f64, f64)> = match seg {
            PathSegment::MoveTo(p) => {
                last = at(p);
                start = last;
                continue;
            }
            PathSegment::LineTo(p) => vec![at(p)],
            PathSegment::QuadTo(a, b) => vec![at(a), at(b)],
            PathSegment::CubicTo(a, b, c) => vec![at(a), at(b), at(c)],
            PathSegment::Close => vec![start],
        };
        let lo = ps.iter().map(|p| p.1).fold(last.1, f64::min);
        let hi = ps.iter().map(|p| p.1).fold(last.1, f64::max);
        rows += (hi - lo) * k;
        let mut from = last;
        for p in &ps {
            length += (p.0 - from.0).hypot(p.1 - from.1) * k;
            from = *p;
        }
        last = ps.last().copied().unwrap_or(last);
        count += 1;
    }
    (rows, length, count)
}

/// A rectangle's area, `k` pixels a unit.
fn area(w: f32, h: f32, k: f64) -> f64 {
    f64::from(w.max(0.0)) * f64::from(h.max(0.0)) * k * k
}

/// The elements a group draws (a pattern's tile's).
fn nodes_in(group: &usvg::Group) -> usize {
    1 + group
        .children()
        .iter()
        .map(|n| match n {
            usvg::Node::Group(g) => nodes_in(g),
            _ => 1,
        })
        .sum::<usize>()
}

/// Counts what `group` costs into `cost`, `k` pixels a unit at the test drawing's size, stopping
/// as soon as it is past `limits`. `in_pattern`: inside a pattern's tile, where another pattern
/// is refused and the parts' cost is the tiles' (counted where the pattern is used).
fn cost_of(
    group: &usvg::Group,
    cost: &mut Cost,
    limits: &Limits,
    (k, in_pattern): (f64, bool),
) -> Result<(), String> {
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
    // A layer of its own, drawn and then composed: a filter's region once a primitive, a clip's,
    // a mask's, an opacity's.
    if !in_pattern {
        let layers = group
            .filters()
            .iter()
            .map(|f| f.primitives().len())
            .sum::<usize>()
            + usize::from(group.clip_path().is_some())
            + usize::from(group.mask().is_some())
            + usize::from(group.opacity().get() < 1.0);
        if layers > 0 {
            let b = group.abs_layer_bounding_box();
            let px = area(b.width(), b.height(), k);
            let filters = group
                .filters()
                .iter()
                .map(|f| f.primitives().len())
                .sum::<usize>() as f64;
            cost.area += layers as f64 * px;
            cost.work += px * (filters * FILTER + (layers as f64 - filters) * LAYER);
        }
    }
    cost.over(limits)?;
    if let Some(c) = group.clip_path() {
        cost_of(c.root(), cost, limits, (k, in_pattern))?;
    }
    if let Some(m) = group.mask() {
        cost_of(m.root(), cost, limits, (k, in_pattern))?;
    }
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => cost_of(g, cost, limits, (k, in_pattern))?,
            usvg::Node::Path(p) => {
                cost.nodes += 1;
                cost.segments += p.data().len();
                if !in_pattern {
                    // A fill: its edges once, its box; a stroke: its outline both sides of each
                    // segment and its width at each, its box.
                    let (rows, length, count) = edges_of(p.data(), p.abs_transform(), k);
                    if p.fill().is_some() {
                        let b = p.abs_bounding_box();
                        let px = area(b.width(), b.height(), k);
                        cost.edges += rows;
                        cost.area += px;
                        cost.work += rows * EDGE + px;
                    }
                    if let Some(st) = p.stroke() {
                        let t = p.abs_transform();
                        let scale = f64::from((t.sx * t.sy - t.kx * t.ky).abs().sqrt());
                        let width = f64::from(st.width().get()) * scale * k;
                        // Its outline's two sides (and each segment's ends, across its width, however
                        // level it runs), and the band it paints: its length by its width (at least a
                        // pixel: its edges are drawn however thin).
                        let rows = 2.0 * rows + count as f64 * width.max(1.0);
                        cost.edges += rows;
                        cost.area += length * width.max(1.0);
                        cost.work += rows * EDGE + length * width.max(1.0);
                    }
                }
                cost.over(limits)?;
                // Each paint with what it covers: a fill its path's box, a stroke its stroke's
                // (half its width beyond the path all round).
                for (paint, covered) in [
                    p.fill().map(|f| (f.paint(), p.bounding_box())),
                    p.stroke().map(|s| (s.paint(), p.stroke_bounding_box())),
                ]
                .into_iter()
                .flatten()
                {
                    if let usvg::Paint::Pattern(pat) = paint {
                        if in_pattern {
                            return Err("a pattern inside a pattern".into());
                        }
                        // Its tiles over what it covers, and what they draw.
                        let (r, t) = (pat.rect(), pat.transform());
                        let tile = f64::from(r.width())
                            * f64::from(r.height())
                            * f64::from((t.sx * t.sy - t.kx * t.ky).abs());
                        let tiles = if tile > 0.0 {
                            (f64::from(covered.width()) * f64::from(covered.height()) / tile)
                                .ceil()
                                .max(1.0)
                        } else {
                            f64::INFINITY
                        };
                        cost.tiles += tiles;
                        cost.tile_nodes += tiles * nodes_in(pat.root()) as f64;
                        cost.over(limits)?;
                        cost_of(pat.root(), cost, limits, (k, true))?;
                    }
                }
            }
            usvg::Node::Image(i) => {
                cost.nodes += 1;
                // Its own pixels, decoded each time it is drawn (in a pattern's tile, once for
                // each of the pattern's uses).
                let own = i.size();
                let decoded = (f64::from(own.width()) * f64::from(own.height())).ceil();
                cost.pixels = cost.pixels.saturating_add(decoded as u64);
                cost.work += decoded * DECODE;
                if !in_pattern {
                    // (A picture resampled as it is drawn: twice a fill's pixel.)
                    let b = i.abs_bounding_box();
                    let px = area(b.width(), b.height(), k);
                    cost.area += px;
                    cost.work += 2.0 * px;
                }
                cost.over(limits)?;
            }
            usvg::Node::Text(_) => cost.nodes += 1,
        }
    }
    cost.over(limits)
}

/// A pack's drawing read: its tree, within `limits`, its cost counted at its own size (its
/// longer side at most `svg_test_side` pixels) (`Err` with why not). The plug-in reads a pack's
/// drawings only through this or [`parse_at`].
pub fn parse(bytes: &[u8], limits: &Limits) -> Result<(usvg::Tree, Cost), String> {
    parse_at(bytes, limits, None)
}

/// The side, in pixels, a drawing's cost is counted at: `side` if given, else its own size's
/// longer side, at most `svg_test_side`.
fn side_of(tree: &usvg::Tree, limits: &Limits, side: Option<u32>) -> u32 {
    let size = tree.size();
    side.unwrap_or_else(|| {
        (size.width().max(size.height()).ceil() as u32).min(limits.svg_test_side)
    })
    .max(1)
}

/// A pack's drawing read as [`parse`] reads it, its cost counted drawn with its longer side
/// `side` pixels (the size a plug-in draws it at) if given.
pub fn parse_at(
    bytes: &[u8],
    limits: &Limits,
    side: Option<u32>,
) -> Result<(usvg::Tree, Cost), String> {
    read_at(bytes, limits, side).map_err(|(_, why)| why)
}

/// [`parse_at`], refused with its finding's code.
fn read_at(
    bytes: &[u8],
    limits: &Limits,
    side: Option<u32>,
) -> Result<(usvg::Tree, Cost), Refusal> {
    let refused = |why: String| ("svg.refused", why);
    let text = std::str::from_utf8(bytes).map_err(|_| refused("not UTF-8 text".into()))?;
    scan(text, limits)?;
    let tree = usvg::Tree::from_str(text, &options())
        .map_err(|e| refused(format!("cannot be read: {e}")))?;
    let mut cost = Cost::default();
    // Counted at the size it is drawn.
    let size = tree.size();
    let k = f64::from(side_of(&tree, limits, side)) / f64::from(size.width().max(size.height()));
    cost_of(tree.root(), &mut cost, limits, (k, false)).map_err(refused)?;
    if cost.pixels > limits.picture_pixels {
        return Err((
            "svg.picture",
            format!(
                "its embedded pictures are {} pixels as drawn (each counted as often as it is \
                 drawn), more than {}",
                cost.pixels, limits.picture_pixels
            ),
        ));
    }
    Ok((tree, cost))
}

/// How a test drawing went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drawn {
    /// Within its deadline, in this many milliseconds.
    In(u64),
    /// Not done by its deadline (left to finish on its own thread, its result unused).
    Late,
    /// Not drawn: an earlier test drawing ran past its deadline and is still running.
    Refused,
    /// The drawing has no size to draw at.
    Sizeless,
}

/// A test drawing that ran past its deadline is still running: no other is started until it
/// ends, so that at most one runaway drawing ever runs at a time (in a process: a plug-in's
/// instances share it).
static RUNAWAY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Draws `tree` with its longer side `side` pixels on a thread of its own, waiting for it no
/// longer than `deadline_ms`: the waiting itself is bounded, whatever the drawing costs. While
/// an earlier drawing runs past its deadline, none is drawn ([`Drawn::Refused`]).
pub fn test_draw(tree: &usvg::Tree, side: u32, deadline_ms: u64) -> Drawn {
    use std::sync::atomic::Ordering;
    if RUNAWAY.load(Ordering::Acquire) {
        return Drawn::Refused;
    }
    let size = tree.size();
    let k = side as f32 / size.width().max(size.height());
    let (w, h) = (
        (size.width() * k).ceil().max(1.0) as u32,
        (size.height() * k).ceil().max(1.0) as u32,
    );
    let Some(mut pixmap) = Pixmap::new(w, h) else {
        return Drawn::Sizeless;
    };
    let tree = tree.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    // Whether the drawing ended within its deadline: it clears RUNAWAY as it ends if not.
    let late = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let late_here = std::sync::Arc::clone(&late);
    let spawned = std::thread::Builder::new()
        .name("skin-pack test drawing".into())
        .spawn(move || {
            let t = Instant::now();
            resvg::render(&tree, Transform::from_scale(k, k), &mut pixmap.as_mut());
            let _ = tx.send(t.elapsed().as_millis() as u64);
            if late_here.swap(true, Ordering::AcqRel) {
                RUNAWAY.store(false, Ordering::Release);
            }
        });
    if spawned.is_err() {
        return Drawn::Refused;
    }
    match rx.recv_timeout(std::time::Duration::from_millis(deadline_ms.max(1))) {
        Ok(ms) => Drawn::In(ms),
        Err(_) => {
            // Marked late; if the drawing ended in between, it has marked it first, and
            // nothing is running.
            RUNAWAY.store(true, Ordering::Release);
            if late.swap(true, Ordering::AcqRel) {
                RUNAWAY.store(false, Ordering::Release);
            }
            Drawn::Late
        }
    }
}

/// Checks a drawing of a pack, drawing it as a test if `test`: the pixels of the pictures
/// embedded in it (counted with the pack's), if it is not refused.
pub fn check(path: &str, bytes: &[u8], limits: &Limits, test: bool, report: &mut Report) -> u64 {
    let parsed = read_at(bytes, limits, None);
    let pixels = parsed.as_ref().map_or(0, |(_, cost)| cost.pixels);
    match parsed {
        Err((code, why)) => report.reject(code, Some(path), why),
        // (Waited for twice its limit, so that a drawing just over it is measured.)
        Ok((tree, _)) if test => {
            let side = side_of(&tree, limits, None);
            match test_draw(&tree, side, limits.svg_test_ms * 2 + 1) {
                Drawn::In(ms) if ms <= limits.svg_test_ms => {}
                Drawn::In(ms) => report.reject(
                    "svg.slow",
                    Some(path),
                    format!(
                        "drawn {side} pixels across in {ms} ms, more than {} ms",
                        limits.svg_test_ms
                    ),
                ),
                Drawn::Late => report.reject(
                    "svg.slow",
                    Some(path),
                    format!(
                        "not drawn {side} pixels across within {} ms",
                        limits.svg_test_ms * 2 + 1
                    ),
                ),
                // (Its cost is bounded by its counted work, which it passed: the timed test is
                // a backstop, and another pack's runaway does not refuse this one.)
                Drawn::Refused => report.note(
                    "svg.untested",
                    Some(path),
                    "not drawn as a test: an earlier drawing's test is still running past its \
                     deadline (its counted cost is within the limit)",
                ),
                Drawn::Sizeless => {
                    report.reject("svg.size", Some(path), "it has no size to draw at")
                }
            }
        }
        Ok(_) => {}
    }
    pixels
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

    /// `metadata` holds other namespaces' data, never SVG's: usvg reads a style sheet wherever
    /// it is, so one hidden there would be applied unchecked.
    #[test]
    fn nothing_of_svgs_hides_in_metadata() {
        assert!(
            refused("<metadata><style>rect{fill:url(x.svg#a)}</style></metadata>")
                .contains("style sheet")
        );
        assert!(
            refused("<metadata><x:style xmlns:x='urn:x'>rect{}</x:style></metadata>")
                .contains("style sheet")
        );
        assert!(refused("<metadata><rect width='1' height='1'/></metadata>").contains("metadata"));
        // A licence's RDF stays allowed.
        let rdf = svg(
            "<metadata><rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>             <rdf:Description/></rdf:RDF></metadata><rect width='9' height='9'/>",
        );
        assert!(parse(rdf.as_bytes(), &Limits::default()).is_ok());
    }

    /// Patterns filled with patterns, ten references a level: counted as they repeat, and a
    /// pattern inside a pattern refused.
    #[test]
    fn patterns_that_multiply_are_refused() {
        let mut body = String::from(
            "<defs><pattern id='p0' width='1' height='1' patternUnits='userSpaceOnUse'>\
             <rect width='1' height='1'/></pattern>",
        );
        for k in 1..7 {
            body.push_str(&format!(
                "<pattern id='p{k}' width='10' height='10' patternUnits='userSpaceOnUse'>"
            ));
            for _ in 0..10 {
                body.push_str(&format!(
                    "<rect width='10' height='10' fill='url(#p{})'/>",
                    k - 1
                ));
            }
            body.push_str("</pattern>");
        }
        body.push_str("</defs><rect width='100' height='100' fill='url(#p6)'/>");
        assert!(
            refused(&body).contains("elements drawn"),
            "{}",
            refused(&body)
        );
        let nested = "<defs><pattern id='a' width='5' height='5' patternUnits='userSpaceOnUse'>\
                      <rect width='5' height='5'/></pattern>\
                      <pattern id='b' width='10' height='10' patternUnits='userSpaceOnUse'>\
                      <rect width='10' height='10' fill='url(#a)'/></pattern></defs>\
                      <rect width='100' height='100' fill='url(#b)'/>";
        assert!(refused(nested).contains("pattern inside a pattern"));
    }

    /// Ids resolve as usvg resolves them: one given twice, or hidden in `metadata`, is refused.
    #[test]
    fn ids_are_given_once_and_only_to_svgs_elements() {
        assert!(refused("<rect id='a'/><circle id='a'/>").contains("two elements"));
        assert!(refused("<rect xml:id='p' id='q'/><rect fill='url(#p)'/>").contains("namespace"));
        assert!(
            refused("<metadata><x:d xmlns:x='urn:x' id='p'/></metadata><rect fill='url(#p)'/>")
                .contains("inside `metadata`")
        );
    }

    /// A pattern's tiles under a stroke are counted over the stroke, however thin the path.
    #[test]
    fn a_stroked_patterns_tiles_are_counted_over_the_stroke() {
        let line = "<defs><pattern id='p' width='0.1' height='0.1' patternUnits='userSpaceOnUse'>\
                    <rect width='0.1' height='0.1'/></pattern></defs>\
                    <line x1='0' y1='50' x2='100' y2='50' stroke='url(#p)' stroke-width='60'/>";
        assert!(refused(line).contains("tiles"), "{}", refused(line));
    }

    /// A tiny tile over a large fill: millions of tiles, refused before anything is drawn.
    #[test]
    fn a_patterns_tiles_are_counted() {
        let tiny = "<defs><pattern id='p' width='0.01' height='0.01' patternUnits='userSpaceOnUse'>\
                    <rect width='0.01' height='0.01'/></pattern></defs>\
                    <rect width='100' height='100' fill='url(#p)'/>";
        assert!(refused(tiny).contains("tiles"));
        let fine = "<defs><pattern id='p' width='10' height='10' patternUnits='userSpaceOnUse'>\
                    <rect width='5' height='5'/></pattern></defs>\
                    <rect width='100' height='100' fill='url(#p)'/>";
        assert!(parse(svg(fine).as_bytes(), &Limits::default()).is_ok());
    }

    #[test]
    fn references_outside_the_drawing_are_refused() {
        assert!(refused("<image href='/etc/passwd' width='1' height='1'/>").contains("refers"));
        assert!(
            refused("<image xlink:href='https://example.com/x.png' width='1' height='1'/>")
                .contains("refers")
        );
        assert!(refused("<image href='data:image/svg+xml;base64,AAAA'/>").contains("are PNG"));
        assert!(refused("<use href='other.svg#a'/>").contains("refers"));
        assert!(refused("<rect fill='url(other.svg#g)'/>").contains("outside"));
        assert!(refused("<rect style=\"fill:url('file:///x')\"/>").contains("outside"));
    }

    /// `data` in base64, as a drawing embeds it.
    fn base64(data: &[u8]) -> String {
        const DIGITS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let n = chunk
                .iter()
                .enumerate()
                .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
            for i in 0..4 {
                out.push(if i <= chunk.len() {
                    char::from(DIGITS[(n >> (18 - 6 * i)) as usize & 63])
                } else {
                    '='
                });
            }
        }
        out
    }

    /// A PNG `w` by `h` of one colour.
    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(png::ColorType::Rgba);
        let mut writer = e.write_header().expect("header");
        writer
            .write_image_data(&rgba.repeat((w * h) as usize))
            .expect("data");
        drop(writer);
        out
    }

    fn image(png: &[u8], attrs: &str) -> String {
        format!(
            "<image {attrs} href='data:image/png;base64,{}'/>",
            base64(png)
        )
    }

    /// What `check` finds in a drawing of `body`.
    fn found(body: &str, limits: &Limits) -> Report {
        let mut r = Report::default();
        check("art/panel.svg", svg(body).as_bytes(), limits, false, &mut r);
        r
    }

    /// A PNG embedded in a drawing is drawn, its pixels counted with the pack's and its decoding
    /// as drawing work.
    #[test]
    fn an_embedded_png_is_drawn() {
        let body = image(
            &solid(20, 10, [200, 30, 40, 255]),
            "x='10' y='10' width='80' height='40'",
        );
        let (tree, cost) = parse(svg(&body).as_bytes(), &Limits::default()).expect("read");
        assert_eq!(cost.pixels, 200);
        assert!(cost.work >= 200.0 * DECODE, "{cost:?}");
        let mut pixmap = Pixmap::new(100, 100).expect("a pixmap");
        resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
        let at = |x: u32, y: u32| pixmap.pixel(x, y).expect("a pixel");
        let inside = at(50, 30);
        assert_eq!(
            (inside.red(), inside.green(), inside.blue(), inside.alpha()),
            (200, 30, 40, 255)
        );
        assert_eq!(at(50, 80).alpha(), 0);
        assert!(found(&body, &Limits::default()).to_string().is_empty());
        // Used twice, drawn twice: counted twice.
        let twice = format!(
            "<defs>{}</defs><use href='#p'/><use href='#p' x='40'/>",
            image(&solid(20, 10, [0; 4]), "id='p' width='20' height='10'")
        );
        let (_, cost) = parse(svg(&twice).as_bytes(), &Limits::default()).expect("read");
        assert_eq!(cost.pixels, 400);
    }

    /// An embedded picture is checked as a pack's own: past its limits, damaged, or not PNG,
    /// refused, and its finding says so (`svg.picture`).
    #[test]
    fn an_embedded_picture_past_its_limits_or_not_png_is_refused() {
        let limits = Limits::default();
        let code = |body: &str, limits: &Limits, says: &str| {
            let r = found(body, limits);
            assert!(r.has("svg.picture"), "{body:.80}: {r}");
            assert!(r.to_string().contains(says), "{says}: {r}");
        };
        // Wider than a picture may be, refused from its header.
        code(
            &image(&solid(4097, 1, [0; 4]), "width='9' height='9'"),
            &limits,
            "at most 4096",
        );
        // Together past the pack's pixels, the second refused before it is decoded.
        let small = Limits {
            picture_pixels: 100,
            ..Limits::default()
        };
        let one = image(&solid(8, 8, [0; 4]), "width='8' height='8'");
        assert!(found(&one, &small).to_string().is_empty());
        code(
            &format!("{one}{one}"),
            &small,
            "left of the pictures' limit",
        );
        // Drawn more often than the pixels allow.
        let uses = format!("<defs>{}</defs>", image(&solid(8, 8, [0; 4]), "id='p'"))
            + &"<use href='#p'/>".repeat(2);
        code(&uses, &small, "as drawn");
        // Damaged: truncated, or its base64 unreadable.
        let png = solid(64, 64, [9; 4]);
        code(
            &image(&png[..png.len() / 2], ""),
            &limits,
            "embedded picture",
        );
        code(
            "<image href='data:image/png;base64,iVBOR%%%'/>",
            &limits,
            "base64",
        );
        // A JPEG, as itself or labelled PNG; a GIF; a drawing.
        let jpeg = [0xff, 0xd8, 0xff, 0xe0, 0, 16, b'J', b'F', b'I', b'F', 0];
        code(
            &format!("<image href='data:image/jpeg;base64,{}'/>", base64(&jpeg)),
            &limits,
            "are PNG",
        );
        code(&image(&jpeg, ""), &limits, "not a PNG");
        code(
            "<image href='data:image/gif;base64,R0lGODlh'/>",
            &limits,
            "are PNG",
        );
        code(
            "<image href='data:image/svg+xml,%3Csvg/%3E'/>",
            &limits,
            "are PNG",
        );
        code("<image href='data:image/png,%89PNG'/>", &limits, "are PNG");
    }

    /// An `<image>` refers to nothing outside the drawing: a file, a URL, either `href`.
    #[test]
    fn an_external_picture_is_refused() {
        for href in [
            "href='pictures/screw.png'",
            "xlink:href='https://example.com/screw.png'",
            "href='file:///etc/passwd'",
            "href='../screw.png'",
        ] {
            let r = found(
                &format!("<image {href} width='9' height='9'/>"),
                &Limits::default(),
            );
            assert!(r.has("svg.picture"), "{href}: {r}");
            assert!(r.to_string().contains("outside the drawing"), "{r}");
        }
        // One `href` embedded and the other outside: refused too.
        let both = format!(
            "<image xlink:href='screw.png' href='data:image/png;base64,{}'/>",
            base64(&solid(2, 2, [0; 4]))
        );
        assert!(found(&both, &Limits::default()).has("svg.picture"));
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
        assert!(
            refused("<defs><linearGradient id='g' href='#g'/></defs>").contains("repeats itself")
        );
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
        // (Its own size 2000 pixels: drawn so as its test.)
        let s = svg(
            "<filter id='f'><feGaussianBlur stdDeviation='60'/></filter>\
             <rect width='100' height='100' filter='url(#f)'/>",
        )
        .replace("width='100' height='100'>", "width='2000' height='2000'>");
        let mut r = Report::default();
        // (Its cost not capped here: the timed test, the backstop, is what is tested.)
        let limits = Limits {
            svg_test_ms: 0,
            svg_test_side: 2000,
            svg_work: f64::INFINITY,
            ..Limits::default()
        };
        check("art/slow.svg", s.as_bytes(), &limits, true, &mut r);
        assert!(r.has("svg.slow"), "{r}");
    }
}
