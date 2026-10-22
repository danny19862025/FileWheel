use gtk::cairo::{Context, FontSlant, FontWeight};
use std::f64::consts::{PI, TAU};

pub fn hit(x: f64, y: f64, size: f64, count: usize) -> Option<usize> {
    if count == 0 {
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
    let relative = (angle % step) - step / 2.;
    if relative.abs() > step / 2. - 0.035 {
        return None;
    }
    Some(index.min(count - 1))
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
pub fn draw(
    cr: &Context,
    size: f64,
    items: &[(&str, bool)],
    hover: Option<usize>,
    has_files: bool,
    center: &str,
    subtitle: &str,
) {
    let middle = size / 2.;
    let outer = size * 0.465;
    let inner = size * 0.205;
    let _ = cr.save();
    cr.translate(middle, middle);
    let count = items.len().max(1);
    let step = TAU / count as f64;
    for (i, (label, enabled)) in items.iter().enumerate() {
        let angle = -PI / 2. + i as f64 * step;
        let start = angle - step / 2. + 0.035;
        let end = angle + step / 2. - 0.035;
        cr.new_path();
        cr.arc(0., 0., outer, start, end);
        cr.arc_negative(0., 0., inner, end, start);
        cr.close_path();
        let hot = hover == Some(i) && *enabled && has_files;
        if hot {
            cr.set_source_rgb(1., 0.49, 0.235);
        } else {
            cr.set_source_rgb(0.145, 0.155, 0.17);
        }
        let _ = cr.fill_preserve();
        cr.set_line_width(1.);
        cr.set_source_rgba(1., 1., 1., if hot { 0.18 } else { 0.045 });
        let _ = cr.stroke();
        let radius = (outer + inner) / 2.;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        let color = if hot {
            (0.12, 0.075, 0.05)
        } else if *enabled && has_files {
            (0.92, 0.92, 0.92)
        } else {
            (0.43, 0.45, 0.48)
        };
        let words = label.split_whitespace().collect::<Vec<_>>();
        if label.len() > 10 && words.len() > 1 {
            let split = words.len().div_ceil(2);
            text(cr, &words[..split].join(" "), x, y - 2., 12., true, color);
            text(cr, &words[split..].join(" "), x, y + 15., 12., true, color);
        } else {
            text(cr, label, x, y + 5., 13., true, color);
        }
    }
    cr.set_source_rgb(0.09, 0.098, 0.11);
    cr.arc(0., 0., inner - 5., 0., TAU);
    let _ = cr.fill();
    cr.set_source_rgb(1., 0.49, 0.235);
    fan(cr, 0., -25., 15.);
    let title = if center.chars().count() > 20 {
        format!("{}…", center.chars().take(18).collect::<String>())
    } else {
        center.into()
    };
    text(cr, &title, 0., 9., 13., true, (0.96, 0.96, 0.96));
    let subtitle = if subtitle.chars().count() > 20 {
        format!("{}…", subtitle.chars().take(18).collect::<String>())
    } else {
        subtitle.to_string()
    };
    text(cr, &subtitle, 0., 29., 10., false, (0.56, 0.59, 0.63));
    let _ = cr.restore();
}

#[cfg(test)]
mod tests {
    use super::*;
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
