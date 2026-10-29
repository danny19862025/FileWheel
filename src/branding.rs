use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc, time::Duration};

const BRAND_FONT: &[u8] = include_bytes!("../data/fonts/Ribeye-Regular.ttf");

/// Register the bundled font in GTK's actual font map, including modern
/// GTK builds that use a map separate from Fontconfig's global configuration.
pub fn load_brand_font(map: &gtk::pango::FontMap) -> Result<(), Box<dyn std::error::Error>> {
    let directory = glib::user_cache_dir().join("filewheel/fonts");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("Ribeye-Regular.ttf");
    if std::fs::read(&path).ok().as_deref() != Some(BRAND_FONT) {
        std::fs::write(&path, BRAND_FONT)?;
    }
    map.add_font_file(&path)?;
    Ok(())
}

/// Type the wordmark once, then leave the completed text without a caret.
pub fn animate_title(title: &gtk::Label) {
    if gtk::Settings::default().is_some_and(|settings| !settings.is_gtk_enable_animations()) {
        title.set_text("FileWheel");
        return;
    }
    title.set_width_chars(10);
    title.set_text("▏");
    let weak = title.downgrade();
    let mut letters = 0;
    glib::timeout_add_local(Duration::from_millis(143), move || {
        let Some(title) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        letters += 1;
        if letters == "FileWheel".len() {
            title.set_text("FileWheel");
            return glib::ControlFlow::Break;
        }
        title.set_text(&format!("{}▏", &"FileWheel"[..letters]));
        glib::ControlFlow::Continue
    });
}

/// Run once using GTK's frame clock, then remove the intro and stop ticking.
pub fn opening_animation(overlay: &gtk::Overlay, title: &gtk::Label) {
    if gtk::Settings::default().is_some_and(|settings| !settings.is_gtk_enable_animations()) {
        animate_title(title);
        return;
    }
    let Ok(loader) = gtk::gdk_pixbuf::PixbufLoader::with_type("svg") else {
        animate_title(title);
        return;
    };
    loader.set_size(512, 512);
    if loader
        .write(include_bytes!("../data/filewheel.svg"))
        .is_err()
        || loader.close().is_err()
    {
        animate_title(title);
        return;
    }
    let Some(logo) = loader.pixbuf() else {
        animate_title(title);
        return;
    };
    let intro = gtk::Overlay::builder().hexpand(true).vexpand(true).build();
    intro.set_widget_name("filewheel-opening");
    let skip_keys = gtk::EventControllerKey::new();
    let cleanup_keys = skip_keys.clone();
    let cleanup_overlay = overlay.downgrade();
    let weak_title = title.downgrade();
    let once = Cell::new(false);
    intro.connect_unmap(move |_| {
        if let Some(overlay) = cleanup_overlay.upgrade() {
            overlay.remove_controller(&cleanup_keys);
        }
        if !once.replace(true) {
            if let Some(title) = weak_title.upgrade() {
                animate_title(&title);
            }
        }
    });
    let canvas = gtk::DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    let elapsed = Rc::new(Cell::new(0.0_f64));
    let progress = elapsed.clone();
    canvas.set_draw_func(move |_, cr, width, height| {
        use gtk::gdk::prelude::GdkCairoContextExt;
        let t = progress.get();
        let x = f64::from(width) / 2.0;
        let y = f64::from(height) / 2.0 - 55.0;
        let reveal = (t / 0.8).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - reveal).powi(3);
        cr.set_source_rgb(0.035, 0.055, 0.09);
        let _ = cr.paint();
        let glow = gtk::cairo::RadialGradient::new(x, y, 0.0, x, y, 290.0);
        glow.add_color_stop_rgba(0.0, 1.0, 0.45, 0.16, 0.32 * ease);
        glow.add_color_stop_rgba(1.0, 1.0, 0.45, 0.16, 0.0);
        if cr.set_source(&glow).is_ok() {
            let _ = cr.paint();
        }
        for offset in [0.0, 0.32] {
            let pulse = ((t - offset) / 1.5).clamp(0.0, 1.0);
            cr.set_source_rgba(1.0, 0.55, 0.25, ease * (1.0 - pulse) * 0.28);
            cr.set_line_width(1.5);
            cr.arc(x, y, 105.0 + pulse * 145.0, 0.0, std::f64::consts::TAU);
            let _ = cr.stroke();
        }
        let size = 220.0 * (0.72 + 0.28 * ease);
        if cr.save().is_ok() {
            cr.translate(x - size / 2.0, y - size / 2.0 + (1.0 - ease) * 22.0);
            cr.scale(
                size / f64::from(logo.width()),
                size / f64::from(logo.height()),
            );
            cr.set_source_pixbuf(&logo, 0.0, 0.0);
            let _ = cr.paint_with_alpha(ease);
            let _ = cr.restore();
        }
    });
    intro.set_child(Some(&canvas));
    let wordmark = gtk::Box::new(gtk::Orientation::Vertical, 8);
    wordmark.set_halign(gtk::Align::Center);
    wordmark.set_valign(gtk::Align::Center);
    wordmark.set_margin_top(235);
    let title = gtk::Label::new(Some("FileWheel"));
    title.add_css_class("opening-title");
    wordmark.append(&title);
    let subtitle = gtk::Label::new(Some("Local files. Less friction."));
    subtitle.add_css_class("opening-subtitle");
    wordmark.append(&subtitle);
    wordmark.set_opacity(0.0);
    intro.add_overlay(&wordmark);
    overlay.add_overlay(&intro);

    let weak_overlay = overlay.downgrade();
    let weak_intro = intro.downgrade();
    let start = Cell::new(None);
    canvas.add_tick_callback(move |canvas, clock| {
        let Some(intro) = weak_intro.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let now = clock.frame_time();
        let first = start.get().unwrap_or(now);
        start.set(Some(first));
        let t = (now - first) as f64 / 1_000_000.0;
        elapsed.set(t);
        wordmark.set_opacity(((t - 0.25) / 0.5).clamp(0.0, 1.0));
        intro.set_opacity(1.0 - ((t - 1.35) / 0.45).clamp(0.0, 1.0));
        if t >= 1.8 {
            if let Some(overlay) = weak_overlay.upgrade() {
                overlay.remove_overlay(&intro);
            }
            return glib::ControlFlow::Break;
        }
        canvas.queue_draw();
        glib::ControlFlow::Continue
    });
    let click = gtk::GestureClick::new();
    let weak_overlay = overlay.downgrade();
    let weak_intro = intro.downgrade();
    click.connect_pressed(move |_, _, _, _| {
        if let (Some(overlay), Some(intro)) = (weak_overlay.upgrade(), weak_intro.upgrade()) {
            overlay.remove_overlay(&intro);
        }
    });
    intro.add_controller(click);
    let keys = skip_keys;
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak_overlay = overlay.downgrade();
    let weak_intro = intro.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape
            && let (Some(overlay), Some(intro)) = (weak_overlay.upgrade(), weak_intro.upgrade())
        {
            overlay.remove_overlay(&intro);
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    overlay.add_controller(keys);
}
