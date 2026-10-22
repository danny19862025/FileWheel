use crate::appearance::Palette;
use gtk::cairo::{
    Content, Context, FontSlant, FontWeight, LineCap, LineJoin, LinearGradient, RecordingSurface,
};
use std::f64::consts::{PI, TAU};

// Keep the visible rounded corners and the pointer/drop target identical.
fn segment(cr: &Context, size: f64, count: usize, index: usize) {
    let outer = size * 0.465;
    let inner = size * 0.205;
    let step = TAU / count as f64;
    let angle = -PI / 2. + index as f64 * step;
    let start = angle - step / 2. + 0.025;
    let end = angle + step / 2. - 0.025;
    let corner = (size * 0.038).min(inner * (end - start) / 4.);
    let point = |radius: f64, angle: f64| (radius * angle.cos(), radius * angle.sin());
    let quad = |control: (f64, f64), end: (f64, f64)| {
        let (x, y) = cr.current_point().unwrap_or_default();
        cr.curve_to(
            x + (control.0 - x) * 2. / 3.,
            y + (control.1 - y) * 2. / 3.,
            end.0 + (control.0 - end.0) * 2. / 3.,
            end.1 + (control.1 - end.1) * 2. / 3.,
            end.0,
            end.1,
        );
    };
    cr.new_path();
    cr.arc(0., 0., outer, start + corner / outer, end - corner / outer);
    quad(point(outer, end), point(outer - corner, end));
    let p = point(inner + corner, end);
    cr.line_to(p.0, p.1);
    quad(point(inner, end), point(inner, end - corner / inner));
    cr.arc_negative(0., 0., inner, end - corner / inner, start + corner / inner);
    quad(point(inner, start), point(inner + corner, start));
    let p = point(outer - corner, start);
    cr.line_to(p.0, p.1);
    quad(point(outer, start), point(outer, start + corner / outer));
    cr.close_path();
}

pub fn hit(x: f64, y: f64, size: f64, count: usize) -> Option<usize> {
    if count == 0 || size <= 0. || !size.is_finite() || !x.is_finite() || !y.is_finite() {
        return None;
    }
    let dx = x - size / 2.;
    let dy = y - size / 2.;
    let radius = dx.hypot(dy);
    let outer = size * 0.465;
    let inner = size * 0.205;
    if radius < inner || radius > outer {
        return None;
    }
    let step = TAU / count as f64;
    let angle = (dy.atan2(dx) + PI / 2. + step / 2.).rem_euclid(TAU);
    let index = (angle / step).floor() as usize;
    let index = index.min(count - 1);
    let surface = RecordingSurface::create(Content::ColorAlpha, None).ok()?;
    let cr = Context::new(&surface).ok()?;
    segment(&cr, size, count, index);
    cr.in_fill(dx, dy).ok()?.then_some(index)
}
pub fn text(
    cr: &Context,
    text: &str,
    x: f64,
    y: f64,
    size: f64,
    bold: bool,
    color: (f64, f64, f64),
) {
    cr.select_font_face(
        "Sans",
        FontSlant::Normal,
        if bold {
            FontWeight::Bold
        } else {
            FontWeight::Normal
        },
    );
    cr.set_font_size(size);
    cr.set_source_rgb(color.0, color.1, color.2);
    if let Ok(ext) = cr.text_extents(text) {
        cr.move_to(x - ext.width() / 2. - ext.x_bearing(), y);
        let _ = cr.show_text(text);
    }
}
pub fn fan(cr: &Context, x: f64, y: f64, r: f64) {
    let _ = cr.save();
    cr.translate(x, y);
    for i in 0..3 {
        let a = i as f64 * TAU / 3.;
        cr.move_to(0., 0.);
        cr.arc(0., 0., r, a, a + 1.45);
        cr.curve_to(-r * 0.15, r * 0.15, -r * 0.1, -r * 0.1, 0., 0.);
        cr.close_path();
        let _ = cr.fill();
    }
    cr.arc(0., 0., r * 0.17, 0., TAU);
    let _ = cr.fill();
    let _ = cr.restore();
}

fn lines(cr: &Context, points: &[(f64, f64)]) {
    if let Some((x, y)) = points.first() {
        cr.move_to(*x, *y);
        for (x, y) in &points[1..] {
            cr.line_to(*x, *y);
        }
    }
}

// Small native vector icons stay crisp in both themes and follow the accent ink.
fn icon(cr: &Context, label: &str, x: f64, y: f64, scale: f64, color: (f64, f64, f64)) {
    let _ = cr.save();
    cr.translate(x, y);
    cr.scale(scale, scale);
    cr.set_source_rgb(color.0, color.1, color.2);
    cr.set_line_width(1.6);
    cr.set_line_cap(LineCap::Round);
    cr.set_line_join(LineJoin::Round);
    cr.new_path();
    match label {
        "Copy path" | "Copy text" => {
            lines(
                cr,
                &[
                    (-6., -8.),
                    (2., -8.),
                    (6., -4.),
                    (6., 8.),
                    (-6., 8.),
                    (-6., -8.),
                ],
            );
            lines(cr, &[(2., -8.), (2., -4.), (6., -4.)]);
            lines(cr, &[(-3., 0.), (3., 0.)]);
            lines(cr, &[(-3., 3.5), (3., 3.5)]);
        }
        "Reveal" => {
            lines(
                cr,
                &[
                    (-9., -5.),
                    (-3., -5.),
                    (-1., -8.),
                    (4., -8.),
                    (6., -5.),
                    (9., -5.),
                    (9., 7.),
                    (-9., 7.),
                    (-9., -5.),
                ],
            );
            lines(cr, &[(-9., -2.), (9., -2.)]);
        }
        "Open" => {
            lines(cr, &[(-1., -7.), (-8., -7.), (-8., 8.), (7., 8.), (7., 1.)]);
            lines(cr, &[(-1., 1.), (8., -8.), (8., -2.)]);
            lines(cr, &[(2., -8.), (8., -8.)]);
        }
        "Preview" => {
            cr.move_to(-10., 0.);
            cr.curve_to(-4., -9., 4., -9., 10., 0.);
            cr.curve_to(4., 9., -4., 9., -10., 0.);
            cr.new_sub_path();
            cr.arc(0., 0., 2.8, 0., TAU);
        }
        "Trash" => {
            lines(cr, &[(-6., -4.), (-5., 8.), (5., 8.), (6., -4.)]);
            lines(cr, &[(-8., -5.), (8., -5.)]);
            lines(cr, &[(-3., -5.), (-3., -8.), (3., -8.), (3., -5.)]);
            lines(cr, &[(-2., -1.), (-2., 4.)]);
            lines(cr, &[(2., -1.), (2., 4.)]);
        }
        "Resize 1280" => {
            lines(cr, &[(-8., -2.), (-8., -8.), (-2., -8.)]);
            lines(cr, &[(-8., -8.), (-1., -1.)]);
            lines(cr, &[(8., 2.), (8., 8.), (2., 8.)]);
            lines(cr, &[(8., 8.), (1., 1.)]);
        }
        "Compress" => {
            lines(cr, &[(-9., -9.), (-2., -2.), (-2., -7.)]);
            lines(cr, &[(-2., -2.), (-7., -2.)]);
            lines(cr, &[(9., 9.), (2., 2.), (2., 7.)]);
            lines(cr, &[(2., 2.), (7., 2.)]);
        }
        "Rotate 90" => {
            cr.arc(0., 0., 7.5, -PI * 0.65, PI * 0.9);
            lines(cr, &[(-8., 0.), (-7., 5.), (-2., 4.)]);
        }
        "Strip metadata" => {
            lines(
                cr,
                &[
                    (-8., 3.),
                    (1., -7.),
                    (8., -1.),
                    (0., 8.),
                    (-4., 8.),
                    (-8., 3.),
                ],
            );
            lines(cr, &[(-3., -2.), (4., 4.)]);
            lines(cr, &[(2., 8.), (9., 8.)]);
        }
        "Normalize" | "Mono" => {
            for (x, h) in [(-8., 3.), (-4., 6.), (0., 9.), (4., 5.), (8., 2.)] {
                lines(cr, &[(x, -h), (x, h)]);
            }
        }
        "Mute" => {
            lines(
                cr,
                &[
                    (-8., -3.),
                    (-4., -3.),
                    (1., -7.),
                    (1., 7.),
                    (-4., 3.),
                    (-8., 3.),
                    (-8., -3.),
                ],
            );
            lines(cr, &[(5., -3.), (10., 3.)]);
            lines(cr, &[(10., -3.), (5., 3.)]);
        }
        "First 10s" | "First 30s" => {
            cr.arc(0., 0., 8., 0., TAU);
            lines(cr, &[(0., -5.), (0., 0.), (4., 2.)]);
        }
        "Grayscale" => {
            cr.arc(0., 0., 8., 0., TAU);
            let _ = cr.stroke();
            cr.arc(0., 0., 8., PI / 2., PI * 1.5);
            cr.close_path();
            let _ = cr.fill();
        }
        "MP3" | "M4A" | "WAV" | "FLAC" | "OGG" | "OPUS" | "AIFF" | "WMA" => {
            lines(cr, &[(-3., 5.), (-3., -6.), (7., -8.), (7., 3.)]);
            cr.new_sub_path();
            cr.arc(-6., 5., 3., 0., TAU);
            cr.new_sub_path();
            cr.arc(4., 3., 3., 0., TAU);
        }
        "MP4" | "MOV" | "MKV" | "WEBM" | "AVI" | "WMV" | "GIF" => {
            cr.rectangle(-10., -7., 20., 14.);
            lines(cr, &[(-2., -4.), (4., 0.), (-2., 4.), (-2., -4.)]);
        }
        "JPG" | "PNG" | "WEBP" | "HEIC" | "TIFF" | "AVIF" | "BMP" | "Grab frame" => {
            cr.rectangle(-9., -8., 18., 16.);
            lines(cr, &[(-8., 5.), (-3., -1.), (1., 3.), (4., 0.), (9., 5.)]);
            cr.new_sub_path();
            cr.arc(4., -4., 1.2, 0., TAU);
        }
        "ZIP" | "TAR" | "GZIP" => {
            cr.rectangle(-8., -8., 16., 16.);
            lines(cr, &[(-8., -3.), (8., -3.)]);
            lines(cr, &[(-2., 1.), (2., 1.), (2., 4.), (-2., 4.), (-2., 1.)]);
        }
        "XLSX" | "ODS" | "CSV" | "TSV" => {
            cr.rectangle(-8., -8., 16., 16.);
            lines(cr, &[(-8., -2.), (8., -2.)]);
            lines(cr, &[(-8., 3.), (8., 3.)]);
            lines(cr, &[(-2., -8.), (-2., 8.)]);
        }
        _ => {
            lines(
                cr,
                &[
                    (-6., -8.),
                    (2., -8.),
                    (7., -3.),
                    (7., 8.),
                    (-6., 8.),
                    (-6., -8.),
                ],
            );
            lines(cr, &[(2., -8.), (2., -3.), (7., -3.)]);
            lines(cr, &[(-3., 1.), (3., 1.)]);
            lines(cr, &[(-3., 4.), (3., 4.)]);
        }
    }
    let _ = cr.stroke();
    let _ = cr.restore();
}

fn pill(cr: &Context, width: f64, height: f64) {
    cr.new_path();
    cr.arc((width - height) / 2., 0., height / 2., -PI / 2., PI / 2.);
    cr.arc(-(width - height) / 2., 0., height / 2., PI / 2., PI * 1.5);
    cr.close_path();
}

pub fn draw(
    cr: &Context,
    size: f64,
    items: &[(&str, bool)],
    hover: Option<usize>,
    has_files: bool,
    center: &str,
    subtitle: &str,
    palette: &Palette,
    glow: bool,
) {
    let middle = size / 2.;
    let outer = size * 0.465;
    let inner = size * 0.205;
    let _ = cr.save();
    cr.translate(middle, middle);
    let scale = size / 390.;
    let dark = palette.bg.0 < 128;
    // A recessed circular tray, with each wedge raised slightly above it.
    for (spread, alpha) in [(8., 0.015), (5., 0.025), (2., 0.04)] {
        cr.arc(0., 3. * scale, outer + spread * scale, 0., TAU);
        cr.set_source_rgba(0., 0., 0., alpha);
        let _ = cr.fill();
    }
    let tray = palette.surface.blend(palette.button, 0.5).rgb();
    cr.set_source_rgb(tray.0, tray.1, tray.2);
    cr.arc(0., 0., outer + 5. * scale, 0., TAU);
    let _ = cr.fill();
    let count = items.len().max(1);
    let step = TAU / count as f64;
    for (i, (label, enabled)) in items.iter().enumerate() {
        let angle = -PI / 2. + i as f64 * step;
        let hot = hover == Some(i) && *enabled && has_files;
        let _ = cr.save();
        cr.translate(0., 2. * scale);
        segment(cr, size, count, i);
        cr.set_source_rgba(0., 0., 0., if dark { 0.2 } else { 0.07 });
        let _ = cr.fill();
        let _ = cr.restore();
        segment(cr, size, count, i);
        if hot && glow {
            let (r, g, b) = palette.accent.rgb();
            for (width, alpha) in [(16., 0.035), (10., 0.06), (5., 0.12)] {
                cr.set_source_rgba(r, g, b, alpha);
                cr.set_line_width(width);
                let _ = cr.stroke_preserve();
            }
        }
        let base = if hot {
            palette.accent
        } else {
            palette.surface.blend(palette.button, 0.45)
        };
        let top = base.blend(
            crate::appearance::Color(255, 255, 255),
            if dark { 0.04 } else { 0.18 },
        );
        let bottom = base.blend(palette.border, if hot { 0.04 } else { 0.12 });
        let fill = LinearGradient::new(0., -outer, 0., outer);
        let (r, g, b) = top.rgb();
        fill.add_color_stop_rgb(0., r, g, b);
        let (r, g, b) = bottom.rgb();
        fill.add_color_stop_rgb(1., r, g, b);
        let _ = cr.set_source(&fill);
        let _ = cr.fill_preserve();
        cr.set_line_width(scale);
        let (r, g, b) = if hot { palette.accent } else { palette.border }.rgb();
        cr.set_source_rgba(r, g, b, if hot { 0.8 } else { 0.6 });
        let _ = cr.stroke();
        let radius = (outer + inner) / 2. + 2. * scale;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        let color = if hot {
            palette.accent_ink.rgb()
        } else if *enabled && has_files {
            palette.text.rgb()
        } else {
            palette.disabled.rgb()
        };
        icon(cr, label, x, y - 10. * scale, scale, color);
        let uppercase = label.to_uppercase();
        let label = uppercase.as_str();
        let words = label.split_whitespace().collect::<Vec<_>>();
        if label.len() > 10 && words.len() > 1 {
            let split = words.len().div_ceil(2);
            text(
                cr,
                &words[..split].join(" "),
                x,
                y + 15. * scale,
                10. * scale,
                true,
                color,
            );
            text(
                cr,
                &words[split..].join(" "),
                x,
                y + 27. * scale,
                10. * scale,
                true,
                color,
            );
        } else {
            text(cr, label, x, y + 17. * scale, 10.5 * scale, true, color);
        }
    }
    let (r, g, b) = palette.surface.rgb();
    cr.set_source_rgb(r, g, b);
    cr.arc(0., 0., inner - 5. * scale, 0., TAU);
    let _ = cr.fill();
    let _ = cr.save();
    cr.translate(0., -6. * scale);
    for (spread, alpha) in [(7., 0.015), (4., 0.025), (1., 0.04)] {
        pill(
            cr,
            inner * 1.7 + spread * scale,
            42. * scale + spread * scale,
        );
        cr.set_source_rgba(0., 0., 0., alpha);
        let _ = cr.fill();
    }
    pill(cr, inner * 1.7, 42. * scale);
    let (r, g, b) = palette.surface.blend(palette.bg, 0.12).rgb();
    cr.set_source_rgb(r, g, b);
    let _ = cr.fill();
    let title = if center == "Choose a segment" {
        "CHOOSE".to_string()
    } else if center.chars().count() > 17 {
        format!("{}…", center.chars().take(15).collect::<String>()).to_uppercase()
    } else {
        center.to_uppercase()
    };
    text(
        cr,
        &title,
        0.,
        4. * scale,
        10. * scale,
        true,
        palette.text.rgb(),
    );
    let _ = cr.restore();
    let subtitle = if subtitle.chars().count() > 20 {
        format!("{}…", subtitle.chars().take(18).collect::<String>())
    } else {
        subtitle.to_string()
    };
    text(
        cr,
        &subtitle,
        0.,
        35. * scale,
        9. * scale,
        false,
        palette.muted.rgb(),
    );
    let _ = cr.restore();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rounded_targets_accept_segment_centers_but_reject_cut_corners() {
        let size = 390.;
        for count in 1..=9 {
            let step = TAU / count as f64;
            for index in 0..count {
                let angle = -PI / 2. + index as f64 * step;
                let position = |radius: f64, angle: f64| {
                    (
                        size / 2. + radius * angle.cos(),
                        size / 2. + radius * angle.sin(),
                    )
                };
                let (x, y) = position(size * 0.335, angle);
                assert_eq!(hit(x, y, size, count), Some(index));
                // These points were in the old square corners, outside the
                // new visible buttons. Dropping here must not run an operation.
                for side in [-1., 1.] {
                    let edge = angle + side * (step / 2. - 0.027);
                    let (x, y) = position(size * 0.465 - 0.5, edge);
                    assert_eq!(hit(x, y, size, count), None);
                    let (x, y) = position(size * 0.205 + 0.5, edge);
                    assert_eq!(hit(x, y, size, count), None);
                }
            }
        }
    }
    #[test]
    fn reject_center_outer_ring_and_gaps() {
        assert_eq!(hit(200., 200., 400., 6), None);
        assert_eq!(hit(200., 0., 400., 6), None);
        assert_eq!(hit(200., 55., 400., 6), Some(0));
        let a = -PI / 2. + PI / 6.;
        assert_eq!(
            hit(200. + a.cos() * 150., 200. + a.sin() * 150., 400., 6),
            None
        );
    }
}
