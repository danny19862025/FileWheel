use crate::{
    AppearanceDialog, Ui,
    appearance::{ACCENTS, Color, Palette, Preferences, Theme},
    label,
};
use gtk::{gdk, gio, glib, prelude::*};
use std::rc::Rc;

impl Ui {
    pub(crate) fn palette(&self) -> Palette {
        Palette::new(self.appearance.get(), self.system_dark.get())
    }

    fn style_windows(&self) {
        for window in self.app.windows() {
            if self.appearance.get().hover_glow {
                window.remove_css_class("no-hover-glow");
            } else {
                window.add_css_class("no-hover-glow");
            }
        }
    }
    pub(crate) fn update_theme(&self) {
        self.css.load_from_data(&self.palette().css());
        if let Some(settings) = gtk::Settings::default() {
            let dark = self.appearance.get().theme.is_dark(self.system_dark.get());
            if let Some(property) = settings.find_property("gtk-interface-color-scheme") {
                if let Some(value) = glib::EnumClass::with_type(property.value_type())
                    .and_then(|class| class.to_value_by_nick(if dark { "dark" } else { "light" }))
                {
                    settings.set_property_from_value("gtk-interface-color-scheme", &value);
                }
            } else {
                settings.set_gtk_application_prefer_dark_theme(dark);
            }
        }
        self.style_windows();
        self.redraw();
    }
    pub(crate) fn set_system_scheme(&self, scheme: u32) {
        match scheme {
            1 => self.system_dark.set(true),
            2 => self.system_dark.set(false),
            _ => {}
        }
        if self.appearance.get().theme == Theme::System {
            self.update_theme();
        }
    }
    pub(crate) fn watch_system_theme(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        gio::bus_get(
            gio::BusType::Session,
            gio::Cancellable::NONE,
            move |result| {
                let (Ok(bus), Some(ui)) = (result, weak.upgrade()) else {
                    return;
                };
                let weak = Rc::downgrade(&ui);
                let subscription = bus.subscribe_to_signal(
                    Some("org.freedesktop.portal.Desktop"),
                    Some("org.freedesktop.portal.Settings"),
                    Some("SettingChanged"),
                    Some("/org/freedesktop/portal/desktop"),
                    Some("org.freedesktop.appearance"),
                    gio::DBusSignalFlags::NONE,
                    move |signal| {
                        if let Some((namespace, key, value)) =
                            signal.parameters.get::<(String, String, glib::Variant)>()
                        {
                            if namespace == "org.freedesktop.appearance" && key == "color-scheme" {
                                if let (Some(scheme), Some(ui)) =
                                    (value.get::<u32>(), weak.upgrade())
                                {
                                    ui.set_system_scheme(scheme);
                                }
                            }
                        }
                    },
                );
                *ui.theme_subscription.borrow_mut() = Some(subscription);
                let weak = Rc::downgrade(&ui);
                bus.call(
                    Some("org.freedesktop.portal.Desktop"),
                    "/org/freedesktop/portal/desktop",
                    "org.freedesktop.portal.Settings",
                    "Read",
                    Some(&("org.freedesktop.appearance", "color-scheme").to_variant()),
                    None,
                    gio::DBusCallFlags::NONE,
                    1000,
                    gio::Cancellable::NONE,
                    move |result| {
                        if let (Ok(value), Some(ui)) = (result, weak.upgrade()) {
                            if let Some((mut value,)) = value.get::<(glib::Variant,)>() {
                                while value.is::<glib::Variant>() {
                                    value = value.as_variant().expect("Variant wrapper");
                                }
                                if let Some(scheme) = value.get::<u32>() {
                                    ui.set_system_scheme(scheme);
                                }
                            }
                        }
                    },
                );
            },
        );
    }
    pub(crate) fn set_appearance(&self, prefs: Preferences) {
        if self.appearance.get() == prefs {
            return;
        }
        self.appearance.set(prefs);
        self.update_theme();
        let saved = self
            .appearance_file
            .borrow()
            .as_deref()
            .map(|path| prefs.save(path));
        if let Some(dialog) = self.appearance_dialog.borrow().as_ref() {
            dialog.theme.set_selected(prefs.theme.index());
            for (color, button) in &dialog.swatches {
                button.set_active(*color == prefs.accent);
            }
            dialog.custom.set_rgba(&rgba(prefs.accent));
            dialog.hex.set_text(&prefs.accent.hex());
            dialog.hex.remove_css_class("invalid");
            dialog.sound.set_active(prefs.hover_sound);
            dialog.glow.set_active(prefs.hover_glow);
            match saved {
                Some(Err(error)) => {
                    dialog.status.set_text(&format!(
                        "Applied for this session. Could not save: {error}"
                    ));
                    dialog.status.add_css_class("fw-error");
                }
                _ => {
                    dialog
                        .status
                        .set_text("Changes apply instantly and save automatically.");
                    dialog.status.remove_css_class("fw-error");
                }
            }
        }
    }
    fn apply_hex(&self) {
        let text = self
            .appearance_dialog
            .borrow()
            .as_ref()
            .map(|d| d.hex.text());
        if let Some(text) = text {
            if let Some(accent) = Color::parse(&text) {
                if accent == self.appearance.get().accent {
                    if let Some(dialog) = self.appearance_dialog.borrow().as_ref() {
                        dialog.hex.set_text(&accent.hex());
                        dialog.hex.remove_css_class("invalid");
                        dialog.status.remove_css_class("fw-error");
                        dialog
                            .status
                            .set_text("Changes apply instantly and save automatically.");
                    }
                    return;
                }
                self.set_appearance(Preferences {
                    accent,
                    ..self.appearance.get()
                });
            } else if let Some(dialog) = self.appearance_dialog.borrow().as_ref() {
                dialog.hex.add_css_class("invalid");
                dialog
                    .status
                    .set_text("Enter a six-digit color, such as #579DF6.");
                dialog.status.add_css_class("fw-error");
            }
        }
    }
    pub(crate) fn play_hover(&self) {
        if self.appearance.get().hover_sound {
            self.hover_sound.play();
        }
    }
    fn preview_effects(self: &Rc<Self>) {
        let dialog = self.appearance_dialog.borrow();
        let Some(dialog) = dialog.as_ref() else {
            return;
        };
        if let Some(timer) = dialog.preview_timer.borrow_mut().take() {
            timer.remove();
        }
        let prefs = self.appearance.get();
        if prefs.hover_sound {
            self.hover_sound.preview();
        }
        if prefs.hover_glow {
            dialog.preview.add_css_class("effect-preview-active");
        } else {
            dialog.preview.remove_css_class("effect-preview-active");
        }
        dialog
            .preview_label
            .set_text(match (prefs.hover_sound, prefs.hover_glow) {
                (true, true) => "Sound + glow",
                (true, false) => "Sound preview",
                (false, true) => "Glow preview",
                (false, false) => "Turn on an effect above.",
            });
        let weak = Rc::downgrade(self);
        *dialog.preview_timer.borrow_mut() = Some(glib::timeout_add_local_once(
            std::time::Duration::from_millis(900),
            move || {
                if let Some(ui) = weak.upgrade() {
                    if let Some(dialog) = ui.appearance_dialog.borrow().as_ref() {
                        dialog.preview.remove_css_class("effect-preview-active");
                        dialog.preview_label.set_text("Effect preview");
                        dialog.preview_timer.borrow_mut().take();
                    }
                }
            },
        ));
    }
    pub(crate) fn attach_feedback(self: &Rc<Self>) {
        self.style_windows();
        let mut widgets = self
            .app
            .windows()
            .into_iter()
            .map(|w| w.upcast::<gtk::Widget>())
            .collect::<Vec<_>>();
        while let Some(widget) = widgets.pop() {
            if (widget.is::<gtk::Button>() || widget.is::<gtk::CheckButton>())
                && !widget.has_css_class("fw-feedback")
            {
                widget.add_css_class("fw-feedback");
                let motion = gtk::EventControllerMotion::new();
                let weak = Rc::downgrade(self);
                motion.connect_enter(move |controller, _, _| {
                    if controller.widget().is_some_and(|w| w.is_sensitive()) {
                        if let Some(ui) = weak.upgrade() {
                            ui.play_hover();
                        }
                    }
                });
                widget.add_controller(motion);
            }
            let mut child = widget.first_child();
            while let Some(next) = child {
                child = next.next_sibling();
                widgets.push(next);
            }
        }
    }
    pub(crate) fn show_appearance(self: &Rc<Self>) {
        if let Some(dialog) = self.appearance_dialog.borrow().as_ref() {
            dialog.window.present();
            return;
        }
        let prefs = self.appearance.get();
        let window = gtk::Window::builder()
            .application(&self.app)
            .transient_for(&self.window)
            .title("Appearance")
            .default_width(490)
            .default_height(720)
            .resizable(true)
            .hide_on_close(true)
            .build();
        let header = gtk::HeaderBar::new();
        header.set_show_title_buttons(false);
        header.set_title_widget(Some(&label("Appearance", "muted")));
        let close = gtk::Button::with_label("×");
        close.set_tooltip_text(Some("Close Appearance"));
        close.update_property(&[gtk::accessible::Property::Label("Close Appearance")]);
        let weak = window.downgrade();
        close.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.close();
            }
        });
        header.pack_end(&close);
        window.set_titlebar(Some(&header));
        let keys = gtk::EventControllerKey::new();
        let weak = window.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(window) = weak.upgrade() {
                    window.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(keys);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.add_css_class("appearance-shell");
        content.append(&label("Make it yours.", "appearance-title"));
        content.append(&label("Your colors. Your kind of quiet.", "muted"));
        let theme_card = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        theme_card.add_css_class("appearance-card");
        let theme_label = label("Theme", "status-title");
        theme_label.set_hexpand(true);
        theme_card.append(&theme_label);
        let theme = gtk::DropDown::from_strings(&["Dark", "Light", "Follow system"]);
        theme.set_selected(prefs.theme.index());
        theme.update_property(&[gtk::accessible::Property::Label("Theme")]);
        theme_card.append(&theme);
        content.append(&theme_card);
        let colors = gtk::Box::new(gtk::Orientation::Vertical, 14);
        colors.add_css_class("appearance-card");
        colors.append(&label("Accent color", "status-title"));
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 7);
        let mut swatches: Vec<(Color, gtk::ToggleButton)> = Vec::new();
        for (name, color) in ACCENTS {
            let button = gtk::ToggleButton::new();
            button.add_css_class("swatch");
            button.set_tooltip_text(Some(name));
            button.update_property(&[gtk::accessible::Property::Label(name)]);
            if let Some((_, first)) = swatches.first() {
                button.set_group(Some(first));
            }
            button.set_active(color == prefs.accent);
            let dot = gtk::DrawingArea::builder()
                .content_width(24)
                .content_height(24)
                .build();
            dot.set_draw_func(move |_, cr, w, h| {
                let (r, g, b) = color.rgb();
                cr.set_source_rgb(r, g, b);
                cr.arc(w as f64 / 2., h as f64 / 2., 10., 0., std::f64::consts::TAU);
                let _ = cr.fill();
            });
            button.set_child(Some(&dot));
            row.append(&button);
            swatches.push((color, button));
        }
        colors.append(&row);
        let custom_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let custom_label = label("Custom", "muted");
        custom_label.set_hexpand(true);
        custom_row.append(&custom_label);
        let dialog = gtk::ColorDialog::builder()
            .title("Choose your accent color")
            .with_alpha(false)
            .build();
        let custom = gtk::ColorDialogButton::new(Some(dialog));
        custom.set_rgba(&rgba(prefs.accent));
        custom.set_tooltip_text(Some("Choose any accent color"));
        custom.update_property(&[gtk::accessible::Property::Label("Custom accent color")]);
        custom_row.append(&custom);
        let hex = gtk::Entry::builder()
            .text(prefs.accent.hex())
            .placeholder_text("#RRGGBB")
            .max_length(7)
            .width_chars(9)
            .build();
        hex.update_property(&[gtk::accessible::Property::Label("Accent hex color")]);
        custom_row.append(&hex);
        let apply = gtk::Button::with_label("Apply");
        custom_row.append(&apply);
        colors.append(&custom_row);
        content.append(&colors);
        let feedback = gtk::Box::new(gtk::Orientation::Vertical, 10);
        feedback.add_css_class("appearance-card");
        feedback.append(&label("Hover feedback", "status-title"));
        let sound = gtk::CheckButton::with_label("Soft hover sound");
        sound.set_active(prefs.hover_sound);
        let glow = gtk::CheckButton::with_label("Accent glow");
        glow.set_active(prefs.hover_glow);
        feedback.append(&sound);
        feedback.append(&glow);
        let sound_note = label(
            "Sound follows your desktop's event volume. Requires libcanberra.",
            "muted",
        );
        sound_note.set_wrap(true);
        sound_note.set_max_width_chars(48);
        feedback.append(&sound_note);
        content.append(&feedback);
        let preview = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        preview.add_css_class("appearance-preview");
        let preview_label = label("Effect preview", "status-title");
        preview_label.set_hexpand(true);
        preview_label.set_wrap(true);
        preview.append(&preview_label);
        let preview_button = gtk::Button::with_label("Test effects");
        preview_button.set_tooltip_text(Some("Click to replay your enabled sound and glow"));
        preview_button.add_css_class("primary");
        let weak = Rc::downgrade(self);
        preview_button.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                ui.preview_effects();
            }
        });
        preview.append(&preview_button);
        content.append(&preview);
        let status = label("Changes apply instantly and save automatically.", "muted");
        status.set_wrap(true);
        status.set_max_width_chars(50);
        content.append(&status);
        let reset = gtk::Button::with_label("Restore defaults");
        reset.set_halign(gtk::Align::Start);
        reset.add_css_class("quiet");
        content.append(&reset);
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&content)
            .build();
        window.set_child(Some(&scroll));
        let weak = Rc::downgrade(self);
        theme.connect_selected_notify(move |dropdown| {
            if let Some(ui) = weak.upgrade() {
                ui.set_appearance(Preferences {
                    theme: Theme::from_index(dropdown.selected()),
                    ..ui.appearance.get()
                });
            }
        });
        for (color, button) in &swatches {
            let (color, weak) = (*color, Rc::downgrade(self));
            button.connect_toggled(move |b| {
                if b.is_active() {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_appearance(Preferences {
                            accent: color,
                            ..ui.appearance.get()
                        });
                    }
                }
            });
        }
        let weak = Rc::downgrade(self);
        custom.connect_rgba_notify(move |button| {
            if let Some(ui) = weak.upgrade() {
                let c = button.rgba();
                let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
                ui.set_appearance(Preferences {
                    accent: Color(byte(c.red()), byte(c.green()), byte(c.blue())),
                    ..ui.appearance.get()
                });
            }
        });
        let weak = Rc::downgrade(self);
        apply.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                ui.apply_hex();
            }
        });
        let weak = Rc::downgrade(self);
        hex.connect_activate(move |_| {
            if let Some(ui) = weak.upgrade() {
                ui.apply_hex();
            }
        });
        let weak = Rc::downgrade(self);
        reset.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                ui.set_appearance(Preferences::default());
            }
        });
        let weak = Rc::downgrade(self);
        sound.connect_toggled(move |b| {
            if let Some(ui) = weak.upgrade() {
                ui.set_appearance(Preferences {
                    hover_sound: b.is_active(),
                    ..ui.appearance.get()
                });
            }
        });
        let weak = Rc::downgrade(self);
        glow.connect_toggled(move |b| {
            if let Some(ui) = weak.upgrade() {
                ui.set_appearance(Preferences {
                    hover_glow: b.is_active(),
                    ..ui.appearance.get()
                });
            }
        });
        *self.appearance_dialog.borrow_mut() = Some(AppearanceDialog {
            window: window.clone(),
            theme,
            swatches,
            custom,
            hex,
            reset,
            status,
            sound,
            glow,
            preview: preview_button,
            preview_label,
            preview_timer: std::cell::RefCell::new(None),
        });
        self.attach_feedback();
        window.present();
    }
}

fn rgba(color: Color) -> gdk::RGBA {
    let (r, g, b) = color.rgb();
    gdk::RGBA::new(r as f32, g as f32, b as f32, 1.)
}
