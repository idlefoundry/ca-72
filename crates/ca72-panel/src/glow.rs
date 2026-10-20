//! Phosphor light drawn by code, not by SVG filters (decisions.md R-LFO): a stroke's or a
//! disc's coverage as a mask, blurred by three box passes (a Gaussian's likeness, its spread
//! `sigma` pixels), then laid in a colour. resvg's blurs cost milliseconds a scope (the mock-ups'
//! 9.6 ms a screen at 0.44, 35 ms at 0.87); these a fraction of one, so that a scope's trace is
//! drawn again as a knob turns.

use resvg::tiny_skia::{
    FillRule, LineCap, LineJoin, Mask, Paint, Path, PathBuilder, Pixmap, Stroke, Transform,
};

/// A coverage mask `w` by `h` pixels.
pub(crate) struct Coverage {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) a: Vec<f32>,
}

impl Coverage {
    pub(crate) fn new(w: u32, h: u32) -> Self {
        Self {
            w: w as usize,
            h: h as usize,
            a: vec![0.0; w as usize * h as usize],
        }
    }

    /// `path` stroked `width` pixels wide into it, at `alpha`.
    pub(crate) fn stroke(&mut self, path: &Path, width: f32, alpha: f32) {
        let Some(mut m) = Mask::new(self.w as u32, self.h as u32) else {
            return;
        };
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        let mut p = Pixmap::new(self.w as u32, self.h as u32).expect("a mask's size");
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        paint.anti_alias = true;
        p.stroke_path(path, &paint, &stroke, Transform::identity(), None);
        m.data_mut()
            .iter_mut()
            .zip(p.data().chunks_exact(4))
            .for_each(|(d, s)| *d = s[3]);
        self.add(&m, alpha);
    }

    /// A disc of radius `r` about (`x`, `y`) filled into it, at `alpha`.
    pub(crate) fn disc(&mut self, (x, y): (f32, f32), r: f32, alpha: f32) {
        let Some(path) = PathBuilder::from_circle(x, y, r.max(0.1)) else {
            return;
        };
        let mut p = Pixmap::new(self.w as u32, self.h as u32).expect("a mask's size");
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        paint.anti_alias = true;
        p.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        let Some(mut m) = Mask::new(self.w as u32, self.h as u32) else {
            return;
        };
        m.data_mut()
            .iter_mut()
            .zip(p.data().chunks_exact(4))
            .for_each(|(d, s)| *d = s[3]);
        self.add(&m, alpha);
    }

    fn add(&mut self, m: &Mask, alpha: f32) {
        for (d, s) in self.a.iter_mut().zip(m.data()) {
            *d = (*d + f32::from(*s) / 255.0 * alpha).min(1.0);
        }
    }

    /// Blurred, about `sigma` pixels (three box passes each way).
    pub(crate) fn blur(&mut self, sigma: f32) {
        if sigma < 0.3 {
            return;
        }
        // (Three boxes' widths for a Gaussian's sigma.)
        let r = (((12.0 * sigma * sigma / 3.0 + 1.0).sqrt() - 1.0) / 2.0)
            .round()
            .max(1.0) as usize;
        let mut tmp = vec![0.0f32; self.a.len()];
        for _ in 0..3 {
            box_pass(&self.a, &mut tmp, self.w, self.h, r, true);
            box_pass(&tmp, &mut self.a, self.w, self.h, r, false);
        }
    }

    /// Laid over `dst` (premultiplied) in `rgb` at `opacity`, its top left corner at `at` in
    /// `dst`'s pixels, only where `within` (the same size as this) has coverage, if given.
    pub(crate) fn tint(&self, dst: &mut Pixmap, rgb: [u8; 3], opacity: f32, at: (i32, i32)) {
        let (dw, dh) = (dst.width() as i32, dst.height() as i32);
        let data = dst.data_mut();
        for y in 0..self.h as i32 {
            let ty = y + at.1;
            if !(0..dh).contains(&ty) {
                continue;
            }
            for x in 0..self.w as i32 {
                let tx = x + at.0;
                if !(0..dw).contains(&tx) {
                    continue;
                }
                let a = self.a[y as usize * self.w + x as usize] * opacity;
                if a <= 0.0 {
                    continue;
                }
                let i = ((ty * dw + tx) * 4) as usize;
                let keep = 1.0 - a;
                for c in 0..3 {
                    let v = f32::from(rgb[c]) * a + f32::from(data[i + c]) * keep;
                    data[i + c] = v.round().min(255.0) as u8;
                }
                let v = 255.0 * a + f32::from(data[i + 3]) * keep;
                data[i + 3] = v.round().min(255.0) as u8;
            }
        }
    }
}

/// One box pass of radius `r`, across (`across`) or down, `from` into `to`.
fn box_pass(from: &[f32], to: &mut [f32], w: usize, h: usize, r: usize, across: bool) {
    let (n, lines, step, line) = if across { (w, h, 1, w) } else { (h, w, w, 1) };
    let k = 1.0 / (2 * r + 1) as f32;
    for l in 0..lines {
        let base = l * line;
        let at = |i: isize| -> f32 {
            if i < 0 || i >= n as isize {
                0.0
            } else {
                from[base + i as usize * step]
            }
        };
        let mut sum: f32 = (-(r as isize)..=r as isize).map(at).sum();
        for i in 0..n {
            to[base + i * step] = sum * k;
            sum += at(i as isize + r as isize + 1) - at(i as isize - r as isize);
        }
    }
}

/// `src` (premultiplied) laid over `dst` with its top left corner at `at`, only inside `clip`
/// (left, top, right, bottom: `dst`'s pixels).
pub(crate) fn lay(dst: &mut Pixmap, src: &Pixmap, at: (i32, i32), clip: [i32; 4]) {
    let (w, h) = (dst.width() as i32, dst.height() as i32);
    let (sw, sh) = (src.width() as i32, src.height() as i32);
    let x0 = clip[0].max(0).max(at.0);
    let x1 = clip[2].min(w).min(at.0 + sw);
    let y0 = clip[1].max(0).max(at.1);
    let y1 = clip[3].min(h).min(at.1 + sh);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let n = (x1 - x0) as usize * 4;
    for y in y0..y1 {
        let s0 = (((y - at.1) * sw + x0 - at.0) * 4) as usize;
        let d0 = ((y * w + x0) * 4) as usize;
        let (s, d) = (&src.data()[s0..s0 + n], &mut dst.data_mut()[d0..d0 + n]);
        for (d, s) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)) {
            match s[3] {
                0 => {}
                255 => d.copy_from_slice(s),
                a => {
                    let keep = 255 - u16::from(a);
                    for c in 0..4 {
                        d[c] =
                            (u16::from(s[c]) + (u16::from(d[c]) * keep + 127) / 255).min(255) as u8;
                    }
                }
            }
        }
    }
}

/// `src` copied into `dst` (opaque or not, replacing what is there) with its top left corner at
/// `at`, only inside `clip`.
pub(crate) fn copy_in(dst: &mut Pixmap, src: &Pixmap, at: (i32, i32), clip: [i32; 4]) {
    let (w, h) = (dst.width() as i32, dst.height() as i32);
    let (sw, sh) = (src.width() as i32, src.height() as i32);
    let x0 = clip[0].max(0).max(at.0);
    let x1 = clip[2].min(w).min(at.0 + sw);
    let y0 = clip[1].max(0).max(at.1);
    let y1 = clip[3].min(h).min(at.1 + sh);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let n = (x1 - x0) as usize * 4;
    for y in y0..y1 {
        let s0 = (((y - at.1) * sw + x0 - at.0) * 4) as usize;
        let d0 = ((y * w + x0) * 4) as usize;
        dst.data_mut()[d0..d0 + n].copy_from_slice(&src.data()[s0..s0 + n]);
    }
}

/// `src`'s pixels inside `r` (its own pixels), as a pixmap of their own; none if `r` is outside.
pub(crate) fn crop(src: &Pixmap, r: [i32; 4]) -> Option<Pixmap> {
    let r = [
        r[0].max(0),
        r[1].max(0),
        r[2].min(src.width() as i32),
        r[3].min(src.height() as i32),
    ];
    if r[2] <= r[0] || r[3] <= r[1] {
        return None;
    }
    let mut p = Pixmap::new((r[2] - r[0]) as u32, (r[3] - r[1]) as u32)?;
    copy_in(
        &mut p,
        src,
        (-r[0], -r[1]),
        [0, 0, r[2] - r[0], r[3] - r[1]],
    );
    Some(p)
}

/// The colours a phosphor of glow `rgb` lights with: its trace's white-hot core, its bloom (the
/// colour itself), its halo (deeper, the colour squared), its unlit face (the colour, faint).
pub(crate) struct Phosphor {
    pub(crate) core: [u8; 3],
    pub(crate) bloom: [u8; 3],
    pub(crate) halo: [u8; 3],
    pub(crate) face: ([u8; 3], [u8; 3]),
}

impl Phosphor {
    pub(crate) fn of(rgb: [u8; 3]) -> Self {
        let f = |k: f64| rgb.map(|c| (f64::from(c) * k).round() as u8);
        Self {
            core: rgb.map(|c| (f64::from(c) + (255.0 - f64::from(c)) * 0.72).round() as u8),
            bloom: rgb,
            halo: rgb.map(|c| (f64::from(c) * f64::from(c) / 255.0).round() as u8),
            face: (f(0.078), f(0.02)),
        }
    }
}

/// `#rrggbb`.
pub(crate) fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The blur spreads a point's light without losing it (its sum kept within a few per cent,
    /// none past the edges), and the amber default comes out near the mock-ups' colours.
    #[test]
    fn the_blur_keeps_the_light_and_amber_is_the_mock_ups() {
        let mut c = Coverage::new(41, 41);
        c.a[20 * 41 + 20] = 1.0;
        c.blur(3.0);
        let sum: f32 = c.a.iter().sum();
        assert!((sum - 1.0).abs() < 0.03, "{sum}");
        assert!(c.a[20 * 41 + 20] < 0.1 && c.a[20 * 41 + 23] > 0.0);
        let p = Phosphor::of([0xff, 0xb0, 0x20]);
        assert_eq!(p.bloom, [0xff, 0xb0, 0x20]);
        assert!(p.core[1] > 0xe0 && p.halo[1] < 0x80);
    }
}
