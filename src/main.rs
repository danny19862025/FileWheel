mod appearance;
mod appearance_ui;
mod engine;
mod feedback;
mod layer;
mod smoke;
mod wheel;

use appearance::{Color, Palette, Preferences};
use engine::{Kind, Mode, Tools};
use gtk::{gdk, gio, glib, prelude::*};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

struct Ui {
    app: gtk::Application,
    window: gtk::ApplicationWindow,
    about: gtk::AboutDialog,
    pad: gtk::Window,
    floating: gtk::Window,
    ring: gtk::DrawingArea,
    floating_ring: gtk::DrawingArea,
    files_box: gtk::Box,
    choices_box: gtk::FlowBox,
    file_count: gtk::Label,
    subtitle: gtk::Label,
    status: gtk::Label,
    detail: gtk::Label,
    progress: gtk::ProgressBar,
    stop: gtk::Button,
    reveal: gtk::Button,
    mode_buttons: Vec<gtk::Button>,
    pad_area: gtk::DrawingArea,
    appearance: Cell<Preferences>,
    system_dark: Cell<bool>,
    appearance_file: RefCell<Option<PathBuf>>,
    appearance_dialog: RefCell<Option<AppearanceDialog>>,
    css: gtk::CssProvider,
    theme_subscription: RefCell<Option<gio::SignalSubscription>>,
    hover_sound: feedback::HoverSound,
    paths: RefCell<Vec<PathBuf>>,
    drag_paths: RefCell<Option<Vec<PathBuf>>>,
    results: RefCell<Vec<PathBuf>>,
    mode: Cell<Mode>,
    hover: Cell<Option<usize>>,
    busy: Cell<bool>,
    drag_inside: Cell<u32>,
    cancel: Arc<AtomicBool>,
    tools: Tools,
    layer_supported: bool,
    quitting: Cell<bool>,
}
struct AppearanceDialog {
    window: gtk::Window,
    theme: gtk::DropDown,
    swatches: Vec<(Color, gtk::ToggleButton)>,
    custom: gtk::ColorDialogButton,
    hex: gtk::Entry,
    reset: gtk::Button,
    status: gtk::Label,
    sound: gtk::CheckButton,
    glow: gtk::CheckButton,
    preview: gtk::Button,
    preview_label: gtk::Label,
    preview_timer: RefCell<Option<glib::SourceId>>,
}
enum Event {
    Progress(usize, usize, String),
    Finished(PathBuf, engine::Result<engine::Completed>),
    Done,
}

impl Ui {
    fn new(app: &gtk::Application, testing: bool) -> Rc<Self> {
        let appearance_file = if testing { None } else { Preferences::path() };
        let appearance = appearance_file
            .as_deref()
            .map(Preferences::load)
            .unwrap_or_default();
        let system_dark = gtk::Settings::default().is_some_and(|s| {
            s.is_gtk_application_prefer_dark_theme()
                || s.gtk_theme_name()
                    .is_some_and(|name| name.to_ascii_lowercase().contains("dark"))
        });
        let css = gtk::CssProvider::new();
        css.load_from_data(&Palette::new(appearance, system_dark).css());
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("FileWheel")
            .default_width(820)
            .default_height(740)
            .build();
        window.set_icon_name(Some("filewheel"));
        let header = gtk::HeaderBar::new();
        header.set_show_title_buttons(false);
        let logo = gdk::Texture::from_bytes(&glib::Bytes::from_static(include_bytes!(
            "../data/filewheel.png"
        )))
        .expect("Embedded FileWheel logo");
        let brand = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let brand_logo = gtk::Image::from_paintable(Some(&logo));
        brand_logo.set_pixel_size(28);
        brand.append(&brand_logo);
        let brand_name = gtk::Label::new(Some("FileWheel"));
        brand_name.add_css_class("wordmark");
        brand.append(&brand_name);
        header.set_title_widget(Some(&brand));
        let quit = gtk::Button::with_label("Quit");
        quit.set_tooltip_text(Some("Quit FileWheel"));
        header.pack_end(&quit);
        let about = gtk::Button::with_label("About");
        about.set_tooltip_text(Some("About FileWheel"));
        header.pack_end(&about);
        let formats = gtk::Button::with_label("Formats");
        formats.set_tooltip_text(Some("Supported formats and conversion notes"));
        header.pack_end(&formats);
        let appearance_button = gtk::Button::with_label("Appearance");
        appearance_button.set_tooltip_text(Some("Theme and accent color"));
        header.pack_end(&appearance_button);
        let about_dialog = gtk::AboutDialog::builder()
            .application(app)
            .transient_for(&window)
            .modal(true)
            .hide_on_close(true)
            .title("About FileWheel")
            .program_name("FileWheel")
            .version(env!("CARGO_PKG_VERSION"))
            .comments("Local file conversion, a wheel away.\n\nCreated by Danny")
            .logo(&logo)
            .build();
        about_dialog.connect_close_request(|dialog| {
            dialog.set_visible(false);
            glib::Propagation::Stop
        });
        let about_header = gtk::HeaderBar::new();
        about_header.set_show_title_buttons(false);
        about_header.set_title_widget(Some(&label("About FileWheel", "muted")));
        let about_close = gtk::Button::with_label("×");
        about_close.set_tooltip_text(Some("Close About"));
        about_close.update_property(&[gtk::accessible::Property::Label("Close About")]);
        let dialog = about_dialog.downgrade();
        about_close.connect_clicked(move |_| {
            if let Some(dialog) = dialog.upgrade() {
                dialog.close();
            }
        });
        about_header.pack_end(&about_close);
        about_dialog.set_titlebar(Some(&about_header));
        let keys = gtk::EventControllerKey::new();
        let dialog = about_dialog.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(dialog) = dialog.upgrade() {
                    dialog.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        about_dialog.add_controller(keys);
        let dialog = about_dialog.clone();
        about.connect_clicked(move |_| dialog.present());
        let hide = gtk::Button::with_label("Hide");
        hide.add_css_class("quiet");
        header.pack_start(&hide);
        window.set_titlebar(Some(&header));
        let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
        root.add_css_class("shell");
        window.set_child(Some(&root));
        let eyebrow = label("A LITTLE LESS FILE FRICTION", "eyebrow");
        root.append(&eyebrow);
        let hero = label("Drop. Choose. Done.", "hero");
        root.append(&hero);
        let subtitle = label("Local file conversion, a wheel away.", "muted");
        subtitle.set_wrap(true);
        subtitle.set_max_width_chars(90);
        root.append(&subtitle);
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        toolbar.set_margin_top(8);
        let segments = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        segments.add_css_class("segments");
        let mut mode_buttons = Vec::new();
        for mode in [Mode::Convert, Mode::Tools, Mode::Actions] {
            let button = gtk::Button::with_label(mode.label());
            button.add_css_class("mode");
            segments.append(&button);
            mode_buttons.push(button);
        }
        toolbar.append(&segments);
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&spacer);
        let choose = gtk::Button::with_label("+ Add files");
        choose.add_css_class("primary");
        toolbar.append(&choose);
        root.append(&toolbar);
        let body = gtk::Box::new(gtk::Orientation::Horizontal, 18);
        body.set_vexpand(true);
        root.append(&body);
        let wheel_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        wheel_box.set_hexpand(true);
        wheel_box.set_valign(gtk::Align::Center);
        body.append(&wheel_box);
        let ring = gtk::DrawingArea::builder()
            .content_width(390)
            .content_height(390)
            .focusable(true)
            .build();
        ring.set_tooltip_text(Some(
            "Drop a file onto a segment, or select an operation below.",
        ));
        wheel_box.append(&ring);
        let shortcuts = label("SHIFT  Convert     CTRL  Tools     ALT  Actions", "muted");
        shortcuts.add_css_class("small");
        shortcuts.set_halign(gtk::Align::Center);
        wheel_box.append(&shortcuts);
        let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
        card.add_css_class("file-card");
        card.set_size_request(240, -1);
        card.set_valign(gtk::Align::Fill);
        card.set_margin_top(12);
        card.set_margin_bottom(12);
        body.append(&card);
        let file_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let count = label("YOUR FILES", "section-label");
        count.set_hexpand(true);
        file_header.append(&count);
        let clear = gtk::Button::with_label("Clear");
        clear.add_css_class("quiet");
        clear.set_tooltip_text(Some("Clear selection"));
        file_header.append(&clear);
        card.append(&file_header);
        let files_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let scroll = gtk::ScrolledWindow::builder()
            .child(&files_box)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(150)
            .build();
        card.append(&scroll);
        let original = label("Copies save beside your originals.", "muted");
        original.set_wrap(true);
        card.append(&original);
        let options = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let drop_toggle = gtk::CheckButton::with_label("Floating drop target");
        drop_toggle.set_active(!testing);
        options.append(&drop_toggle);
        card.append(&options);
        let dependencies = gtk::Button::with_label("Conversion engines");
        dependencies.add_css_class("quiet");
        card.append(&dependencies);
        let choices_box = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .max_children_per_line(6)
            .min_children_per_line(3)
            .row_spacing(6)
            .column_spacing(6)
            .homogeneous(false)
            .build();
        root.append(&choices_box);
        let status_card = gtk::Box::new(gtk::Orientation::Vertical, 8);
        status_card.add_css_class("status-card");
        root.append(&status_card);
        let status_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let status = label("Ready when you are", "status-title");
        status.set_hexpand(true);
        status_row.append(&status);
        let reveal = gtk::Button::with_label("Show result");
        reveal.set_visible(false);
        status_row.append(&reveal);
        let stop = gtk::Button::with_label("Cancel");
        stop.set_visible(false);
        status_row.append(&stop);
        status_card.append(&status_row);
        let detail = label("Drop files onto the wheel, or choose Add files.", "muted");
        detail.set_wrap(true);
        detail.set_selectable(true);
        detail.set_max_width_chars(100);
        detail.set_lines(3);
        detail.set_ellipsize(gtk::pango::EllipsizeMode::End);
        status_card.append(&detail);
        let progress = gtk::ProgressBar::new();
        progress.set_visible(false);
        status_card.append(&progress);
        let bottom = label(
            "OFFLINE BY DESIGN  ·  No uploads, no account.",
            "section-label",
        );
        bottom.set_halign(gtk::Align::Center);
        root.append(&bottom);
        let pad = gtk::Window::builder()
            .application(app)
            .title("FileWheel target")
            .decorated(false)
            .resizable(false)
            .default_width(82)
            .default_height(94)
            .build();
        pad.add_css_class("pad");
        let floating = gtk::Window::builder()
            .application(app)
            .title("FileWheel wheel")
            .decorated(false)
            .resizable(false)
            .build();
        floating.add_css_class("overlay");
        let layer_supported = layer::attach(&pad, 14);
        layer::attach(&floating, 108);
        let floating_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        floating_box.add_css_class("floating-wheel");
        floating.set_child(Some(&floating_box));
        let floating_ring = gtk::DrawingArea::builder()
            .content_width(390)
            .content_height(390)
            .focusable(true)
            .build();
        floating_box.append(&floating_ring);
        let hint = label("Shift Convert  ·  Ctrl Tools  ·  Alt Actions", "muted");
        hint.set_halign(gtk::Align::Center);
        floating_box.append(&hint);
        let pad_area = gtk::DrawingArea::builder()
            .content_width(82)
            .content_height(94)
            .build();
        pad.set_child(Some(&pad_area));
        pad_area.set_tooltip_text(Some(
            "Drag files here, then onto the wheel. Click to open FileWheel.",
        ));
        let ui = Rc::new(Self {
            app: app.clone(),
            window,
            about: about_dialog,
            pad,
            floating,
            ring,
            floating_ring,
            files_box,
            choices_box,
            file_count: count,
            subtitle,
            status,
            detail,
            progress,
            stop,
            reveal,
            mode_buttons,
            pad_area: pad_area.clone(),
            appearance: Cell::new(appearance),
            system_dark: Cell::new(system_dark),
            appearance_file: RefCell::new(appearance_file),
            appearance_dialog: RefCell::new(None),
            css,
            theme_subscription: RefCell::new(None),
            hover_sound: feedback::HoverSound::new(testing),
            paths: RefCell::new(vec![]),
            drag_paths: RefCell::new(None),
            results: RefCell::new(vec![]),
            mode: Cell::new(Mode::Convert),
            hover: Cell::new(None),
            busy: Cell::new(false),
            drag_inside: Cell::new(0),
            cancel: Arc::new(AtomicBool::new(false)),
            tools: Tools::discover(),
            layer_supported,
            quitting: Cell::new(false),
        });
        ui.update_theme();
        ui.watch_system_theme();
        let weak = Rc::downgrade(&ui);
        pad_area.set_draw_func(move |_, cr, _, _| {
            let Some(u) = weak.upgrade() else {
                return;
            };
            let p = u.palette();
            let (r, g, b) = p.bg.rgb();
            cr.set_source_rgba(r, g, b, 0.96);
            cr.arc(41., 39., 34., 0., std::f64::consts::TAU);
            let _ = cr.fill_preserve();
            let (r, g, b) = p.border.rgb();
            cr.set_source_rgb(r, g, b);
            cr.set_line_width(1.);
            let _ = cr.stroke();
            let (r, g, b) = p.accent.rgb();
            cr.set_source_rgb(r, g, b);
            wheel::fan(cr, 41., 39., 22.);
            // Put the label on its own opaque pill so light text stays legible
            // against both dark and light desktop wallpaper.
            cr.set_source_rgb(p.bg.rgb().0, p.bg.rgb().1, p.bg.rgb().2);
            cr.rectangle(14., 76., 54., 17.);
            let _ = cr.fill();
            wheel::text(cr, "DROP", 41., 88., 9., true, p.accent_label.rgb());
        });
        let u = ui.clone();
        appearance_button.connect_clicked(move |_| u.show_appearance());
        let u = ui.clone();
        formats.connect_clicked(move |_| u.show_formats());
        for (i, mode) in [Mode::Convert, Mode::Tools, Mode::Actions]
            .into_iter()
            .enumerate()
        {
            let u = ui.clone();
            ui.mode_buttons[i].connect_clicked(move |_| {
                u.mode.set(mode);
                u.hover.set(None);
                u.refresh();
            });
        }
        let u = ui.clone();
        choose.connect_clicked(move |_| u.choose());
        let u = ui.clone();
        clear.connect_clicked(move |_| {
            u.paths.borrow_mut().clear();
            *u.drag_paths.borrow_mut() = None;
            u.refresh();
        });
        let u = ui.clone();
        quit.connect_clicked(move |_| u.quit());
        let u = ui.clone();
        hide.connect_clicked(move |_| u.window.set_visible(false));
        let u = ui.clone();
        ui.window.connect_close_request(move |_| {
            u.window.set_visible(false);
            u.floating.set_visible(false);
            glib::Propagation::Stop
        });
        let u = ui.clone();
        ui.stop.connect_clicked(move |_| {
            u.cancel.store(true, Ordering::Relaxed);
            u.status.set_text("Cancelling…");
        });
        let u = ui.clone();
        ui.reveal
            .connect_clicked(move |_| u.reveal_paths(&u.results.borrow()));
        let u = ui.clone();
        drop_toggle.connect_toggled(move |b| {
            u.pad.set_visible(b.is_active());
            if !b.is_active() {
                u.floating.set_visible(false);
            }
        });
        let u = ui.clone();
        dependencies.connect_clicked(move |_| u.show_dependencies());
        let gesture = gtk::GestureClick::new();
        let u = ui.clone();
        gesture.connect_released(move |_, _, _, _| u.window.present());
        pad_area.add_controller(gesture);
        for area in [&ui.ring, &ui.floating_ring] {
            let u = ui.clone();
            area.set_draw_func(move |_, cr, w, h| u.draw(cr, w.min(h) as f64));
            let motion = gtk::EventControllerMotion::new();
            let u = ui.clone();
            motion.connect_motion(move |controller, x, y| {
                u.modifiers(controller.current_event_state());
                let size = controller
                    .widget()
                    .map(|w| w.width().min(w.height()))
                    .unwrap_or(390) as f64;
                let next = wheel::hit(x, y, size, u.choices().len());
                if next != u.hover.get()
                    && next.is_some_and(|i| {
                        u.choices()
                            .get(i)
                            .is_some_and(|op| u.disabled(op).is_none())
                    })
                {
                    u.play_hover();
                }
                u.hover.set(next);
                u.redraw();
            });
            let u = ui.clone();
            motion.connect_leave(move |_| {
                u.hover.set(None);
                u.redraw();
            });
            area.add_controller(motion);
            let click = gtk::GestureClick::new();
            click.set_button(1);
            let u = ui.clone();
            click.connect_released(move |controller, _, x, y| {
                let size = controller
                    .widget()
                    .map(|w| w.width().min(w.height()))
                    .unwrap_or(390) as f64;
                if let Some(index) = wheel::hit(x, y, size, u.choices().len()) {
                    if let Some(op) = u.choices().get(index) {
                        u.start(op);
                    }
                } else if (x - size / 2.).hypot(y - size / 2.) < size * 0.2 {
                    u.choose();
                }
            });
            area.add_controller(click);
            let key = gtk::EventControllerKey::new();
            let u = ui.clone();
            key.connect_key_pressed(move |_, key, _, state| {
                u.modifiers(state);
                if key == gdk::Key::Escape {
                    u.floating.set_visible(false);
                    u.hover.set(None);
                } else if key == gdk::Key::Right || key == gdk::Key::Down {
                    let n = u.choices().len();
                    if n > 0 {
                        u.hover
                            .set(Some(u.hover.get().map(|i| (i + 1) % n).unwrap_or(0)));
                    }
                } else if key == gdk::Key::Left || key == gdk::Key::Up {
                    let n = u.choices().len();
                    if n > 0 {
                        u.hover.set(Some(
                            u.hover.get().map(|i| (i + n - 1) % n).unwrap_or(n - 1),
                        ));
                    }
                } else if key == gdk::Key::Return || key == gdk::Key::space {
                    if let Some(op) = u.hover.get().and_then(|i| u.choices().get(i).copied()) {
                        u.start(op);
                    }
                } else {
                    return glib::Propagation::Proceed;
                }
                u.redraw();
                glib::Propagation::Stop
            });
            area.add_controller(key);
        }
        ui.install_drop(&ui.ring, false, true);
        ui.install_drop(&ui.floating_ring, false, true);
        ui.install_drop(&pad_area, true, false);
        ui.install_drop(&ui.files_box, false, false);
        ui.refresh();
        tray(&ui);
        if !testing {
            ui.pad.set_visible(true);
        }
        ui
    }
    fn quit(&self) {
        self.cancel.store(true, Ordering::Relaxed);
        if self.busy.get() {
            self.quitting.set(true);
            self.status.set_text("Cancelling before quitting…");
        } else {
            self.app.quit();
        }
    }
    fn effective(&self) -> Vec<PathBuf> {
        self.drag_paths
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.paths.borrow().clone())
    }
    fn choices(&self) -> Vec<&'static str> {
        let paths = self.effective();
        if self.mode.get() == Mode::Actions {
            return engine::menu(Kind::Other, Mode::Actions);
        }
        let Some(first) = paths.first() else {
            return engine::menu(Kind::Image, self.mode.get());
        };
        let common = engine::menu(Kind::of(first), self.mode.get())
            .into_iter()
            .filter(|op| {
                paths
                    .iter()
                    .all(|p| engine::menu(Kind::of(p), self.mode.get()).contains(op))
            })
            .collect::<Vec<_>>();
        if common.is_empty() {
            if paths.iter().all(|p| p.is_file()) {
                vec!["ZIP", "SHA-256"]
            } else {
                vec!["ZIP"]
            }
        } else {
            common
        }
    }
    fn disabled(&self, op: &str) -> Option<String> {
        if self.busy.get() {
            return Some("Wait for the current job or cancel it.".into());
        }
        let paths = self.effective();
        if paths.is_empty() {
            return Some("Add a file first".into());
        }
        paths
            .iter()
            .find_map(|p| self.tools.unavailable_for(p, self.mode.get(), op))
    }
    fn redraw(&self) {
        self.ring.queue_draw();
        self.floating_ring.queue_draw();
        self.pad_area.queue_draw();
    }
    fn draw(&self, cr: &gtk::cairo::Context, size: f64) {
        let choices = self.choices();
        let items = choices
            .iter()
            .map(|op| (*op, self.disabled(op).is_none()))
            .collect::<Vec<_>>();
        let paths = self.effective();
        let selected = self.hover.get().and_then(|i| choices.get(i).copied());
        let center = selected.unwrap_or(if paths.is_empty() {
            "Drop files"
        } else {
            "Choose a segment"
        });
        let sub = if paths.is_empty() {
            "or click to browse".into()
        } else if paths.len() == 1 {
            paths[0]
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            format!("{} files", paths.len())
        };
        wheel::draw(
            cr,
            size,
            &items,
            self.hover.get(),
            !paths.is_empty(),
            center,
            &sub,
            &self.palette(),
            self.appearance.get().hover_glow,
        );
    }
    fn refresh(self: &Rc<Self>) {
        self.redraw();
        for (i, mode) in [Mode::Convert, Mode::Tools, Mode::Actions]
            .into_iter()
            .enumerate()
        {
            if self.mode.get() == mode {
                self.mode_buttons[i].add_css_class("active");
            } else {
                self.mode_buttons[i].remove_css_class("active");
            }
        }
        while let Some(child) = self.files_box.first_child() {
            self.files_box.remove(&child);
        }
        let paths = self.effective();
        self.file_count
            .set_text(&format!("YOUR FILES  ·  {}", paths.len()));
        if paths.is_empty() {
            let hint = label(
                "Drop a file here.\nImages, audio, video, PDFs,\ndocuments, slides, spreadsheets,\nsubtitles and archives.",
                "muted",
            );
            hint.set_wrap(true);
            hint.set_margin_top(32);
            self.files_box.append(&hint);
        } else {
            for path in paths.iter().take(40) {
                let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
                row.add_css_class("file-row");
                let name = label(
                    &path.file_name().unwrap_or_default().to_string_lossy(),
                    "file-title",
                );
                name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                name.set_max_width_chars(27);
                name.set_tooltip_text(Some(&path.display().to_string()));
                row.append(&name);
                let size = path
                    .metadata()
                    .map(|m| format!("{}  ·  {}", Kind::of(path).label(), human_size(m.len())))
                    .unwrap_or_else(|_| "Unavailable".into());
                row.append(&label(&size, "file-kind"));
                self.files_box.append(&row);
            }
        }
        while let Some(child) = self.choices_box.first_child() {
            self.choices_box.remove(&child);
        }
        for op in self.choices() {
            let button = gtk::Button::with_label(op);
            button.add_css_class("operation");
            let reason = self.disabled(op);
            button.set_sensitive(reason.is_none());
            button.set_tooltip_text(reason.as_deref().or(Some("Apply to the selected files")));
            let u = self.clone();
            button.connect_clicked(move |_| u.start(op));
            self.choices_box.insert(&button, -1);
        }
        self.subtitle.set_text(if paths.is_empty() {
            "Local file conversion, a wheel away."
        } else if paths.iter().all(|p| Kind::of(p) == Kind::of(&paths[0])) {
            engine::format_note(Kind::of(&paths[0]))
        } else {
            "Pick a segment or use the buttons below. Originals stay in place."
        });
        self.attach_feedback();
    }
    fn set_paths(self: &Rc<Self>, paths: Vec<PathBuf>, append: bool) {
        let mut selected = if append {
            self.paths.borrow().clone()
        } else {
            vec![]
        };
        for path in paths {
            if path.exists() && !selected.contains(&path) {
                selected.push(path);
            }
        }
        *self.paths.borrow_mut() = selected;
        *self.drag_paths.borrow_mut() = None;
        self.hover.set(None);
        self.refresh();
    }
    fn modifiers(self: &Rc<Self>, flags: gdk::ModifierType) {
        let mode = if flags.contains(gdk::ModifierType::ALT_MASK) {
            Some(Mode::Actions)
        } else if flags.contains(gdk::ModifierType::CONTROL_MASK) {
            Some(Mode::Tools)
        } else if flags.contains(gdk::ModifierType::SHIFT_MASK) {
            Some(Mode::Convert)
        } else {
            None
        };
        if let Some(mode) = mode {
            if self.mode.replace(mode) != mode {
                self.hover.set(None);
                self.refresh();
            }
        }
    }
    fn install_drop(self: &Rc<Self>, widget: &impl IsA<gtk::Widget>, pad: bool, execute: bool) {
        let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        target.set_preload(true);
        let u = self.clone();
        target.connect_value_notify(move |t| {
            if let Some(paths) = drop_paths(t.value().as_ref()) {
                *u.drag_paths.borrow_mut() = Some(paths);
                u.refresh();
                if pad {
                    u.floating.set_visible(true);
                }
            }
        });
        let u = self.clone();
        target.connect_enter(move |t, _, _| {
            u.drag_inside.set(u.drag_inside.get() + 1);
            u.modifiers(t.current_event_state());
            if pad {
                u.floating.set_visible(true);
            }
            gdk::DragAction::COPY
        });
        let u = self.clone();
        target.connect_motion(move |t, x, y| {
            u.modifiers(t.current_event_state());
            if execute {
                let size = t.widget().map(|w| w.width().min(w.height())).unwrap_or(390) as f64;
                let next = wheel::hit(x, y, size, u.choices().len());
                if next != u.hover.get()
                    && next.is_some_and(|i| {
                        u.choices()
                            .get(i)
                            .is_some_and(|op| u.disabled(op).is_none())
                    })
                {
                    u.play_hover();
                }
                u.hover.set(next);
                u.redraw();
            }
            gdk::DragAction::COPY
        });
        let u = self.clone();
        target.connect_leave(move |_| {
            u.drag_inside.set(u.drag_inside.get().saturating_sub(1));
            u.hover.set(None);
            u.redraw();
            let u = u.clone();
            glib::timeout_add_local_once(Duration::from_millis(900), move || {
                if u.drag_inside.get() == 0 {
                    *u.drag_paths.borrow_mut() = None;
                    u.floating.set_visible(false);
                    u.refresh();
                }
            });
        });
        let u = self.clone();
        target.connect_drop(move |t, value, x, y| {
            let Some(paths) = drop_paths(Some(value)) else {
                return false;
            };
            u.modifiers(t.current_event_state());
            let size = t.widget().map(|w| w.width().min(w.height())).unwrap_or(390) as f64;
            u.set_paths(paths, false);
            let op = if execute {
                wheel::hit(x, y, size, u.choices().len()).and_then(|i| u.choices().get(i).copied())
            } else {
                None
            };
            u.drag_inside.set(0);
            u.floating.set_visible(false);
            if let Some(op) = op {
                u.start(op);
            } else {
                u.window.present();
            }
            true
        });
        widget.add_controller(target);
    }
    fn choose(self: &Rc<Self>) {
        let dialog = gtk::FileDialog::builder()
            .title("Choose files for FileWheel")
            .build();
        let u = self.clone();
        dialog.open_multiple(Some(&self.window), gio::Cancellable::NONE, move |result| {
            if let Ok(model) = result {
                let paths = (0..model.n_items())
                    .filter_map(|i| model.item(i).and_downcast::<gio::File>())
                    .filter_map(|f| f.path())
                    .collect();
                u.set_paths(paths, true);
            }
        });
    }
    fn say(&self, title: &str, detail: &str, error: bool) {
        self.status.set_text(title);
        self.detail.set_text(detail);
        self.detail.set_tooltip_text(Some(detail));
        self.status.remove_css_class("fw-success");
        self.status.remove_css_class("fw-error");
        self.status
            .add_css_class(if error { "fw-error" } else { "fw-success" });
    }
    fn start(self: &Rc<Self>, op: &str) {
        if let Some(reason) = self.disabled(op) {
            self.say("Not available", &reason, true);
            return;
        }
        if self.mode.get() == Mode::Actions {
            self.action(op);
            return;
        }
        let paths = self.paths.borrow().clone();
        if paths.is_empty() {
            return;
        }
        let mode = self.mode.get();
        let op = op.to_string();
        let tools = self.tools.clone();
        let cancel = self.cancel.clone();
        cancel.store(false, Ordering::Relaxed);
        self.busy.set(true);
        self.results.borrow_mut().clear();
        self.reveal.set_visible(false);
        self.stop.set_visible(true);
        self.progress.set_fraction(0.);
        self.progress.set_visible(true);
        self.refresh();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let total = paths.len();
            for (i, path) in paths.into_iter().enumerate() {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let _ = tx.send(Event::Progress(
                    i,
                    total,
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                ));
                let result = engine::run(&path, mode, &op, &tools, &cancel);
                engine::log_job(&op, &path, &result);
                let _ = tx.send(Event::Finished(path, result));
            }
            let _ = tx.send(Event::Done);
        });
        let u = self.clone();
        let mut successes = 0usize;
        let mut failures = Vec::new();
        let mut clipboards = Vec::new();
        glib::timeout_add_local(Duration::from_millis(90), move || {
            while let Ok(event) = rx.try_recv() {
                match event {
                    Event::Progress(i, total, name) => {
                        u.status
                            .set_text(&format!("Working · {} of {}", i + 1, total));
                        u.detail.set_text(&name);
                        u.progress.set_fraction(i as f64 / total as f64);
                    }
                    Event::Finished(path, result) => match result {
                        Ok(done) => {
                            successes += 1;
                            if let Some(output) = done.output {
                                u.results.borrow_mut().push(output);
                            }
                            if let Some(text) = done.clipboard {
                                clipboards.push(text);
                            }
                            u.detail.set_text(&done.detail);
                        }
                        Err(error) => failures.push(format!(
                            "{}: {error}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        )),
                    },
                    Event::Done => {
                        u.busy.set(false);
                        u.stop.set_visible(false);
                        u.progress.set_fraction(1.);
                        u.progress.set_visible(false);
                        if !clipboards.is_empty() {
                            if let Some(display) = gdk::Display::default() {
                                display.clipboard().set_text(&clipboards.join("\n"));
                            }
                        }
                        let outputs = u.results.borrow();
                        let detail = if failures.is_empty() {
                            if outputs.is_empty() {
                                "Result copied to your clipboard.".into()
                            } else {
                                outputs
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        } else {
                            failures.join("\n")
                        };
                        let title = if u.cancel.load(Ordering::Relaxed) {
                            format!("Cancelled · {successes} completed")
                        } else if failures.is_empty() {
                            format!(
                                "Done · {successes} {}",
                                if successes == 1 { "file" } else { "files" }
                            )
                        } else {
                            format!("{successes} completed · {} failed", failures.len())
                        };
                        drop(outputs);
                        u.say(&title, &detail, !failures.is_empty());
                        u.reveal.set_visible(!u.results.borrow().is_empty());
                        u.refresh();
                        if u.quitting.get() {
                            u.app.quit();
                        }
                        return glib::ControlFlow::Break;
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }
    fn action(self: &Rc<Self>, op: &str) {
        let paths = self.paths.borrow().clone();
        if paths.is_empty() {
            return;
        }
        match op {
            "Copy path" => {
                if let Some(display) = gdk::Display::default() {
                    display.clipboard().set_text(
                        &paths
                            .iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
                self.say("Paths copied", "Ready to paste.", false);
            }
            "Reveal" => self.reveal_paths(&paths),
            "Open" => {
                for path in paths {
                    open(&path);
                }
            }
            "Preview" => self.preview(&paths[0]),
            "Trash" => {
                let dialog = gtk::AlertDialog::builder()
                    .modal(true)
                    .message(format!(
                        "Move {} {} to Trash?",
                        paths.len(),
                        if paths.len() == 1 { "file" } else { "files" }
                    ))
                    .detail("You can restore them from your desktop's Trash.")
                    .buttons(["Cancel", "Move to Trash"])
                    .cancel_button(0)
                    .default_button(0)
                    .build();
                let u = self.clone();
                dialog.choose(Some(&self.window), gio::Cancellable::NONE, move |result| {
                    if result == Ok(1) {
                        let mut errors = vec![];
                        for path in &paths {
                            if let Err(e) = gio::File::for_path(path).trash(gio::Cancellable::NONE)
                            {
                                errors.push(e.to_string());
                            }
                        }
                        if errors.is_empty() {
                            u.paths.borrow_mut().clear();
                            u.say(
                                "Moved to Trash",
                                "Your desktop's Trash can restore these files.",
                                false,
                            );
                        } else {
                            u.say("Some files could not be trashed", &errors.join("\n"), true);
                        }
                        u.refresh();
                    }
                });
            }
            _ => {}
        }
    }
    fn reveal_paths(&self, paths: &[PathBuf]) {
        if paths.is_empty() {
            return;
        }
        let uris = paths
            .iter()
            .map(|p| gio::File::for_path(p).uri().to_string())
            .collect::<Vec<_>>();
        let result =
            gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).and_then(|bus| {
                bus.call_sync(
                    Some("org.freedesktop.FileManager1"),
                    "/org/freedesktop/FileManager1",
                    "org.freedesktop.FileManager1",
                    "ShowItems",
                    Some(&(uris, String::new()).to_variant()),
                    None,
                    gio::DBusCallFlags::NONE,
                    1500,
                    gio::Cancellable::NONE,
                )
            });
        if result.is_err() {
            if let Some(parent) = paths[0].parent() {
                open(parent);
            }
        }
    }
    fn preview(&self, path: &std::path::Path) {
        let window = gtk::Window::builder()
            .application(&self.app)
            .transient_for(&self.window)
            .title(path.file_name().unwrap_or_default().to_string_lossy())
            .default_width(760)
            .default_height(560)
            .build();
        match Kind::of(path) {
            Kind::Image => {
                let picture = gtk::Picture::for_filename(path);
                picture.set_can_shrink(true);
                window.set_child(Some(&picture));
            }
            Kind::Video | Kind::Audio => {
                let video = gtk::Video::for_filename(Some(path));
                video.set_autoplay(false);
                window.set_child(Some(&video));
            }
            Kind::Document
                if path
                    .extension()
                    .map(|e| e == "txt" || e == "md")
                    .unwrap_or(false) =>
            {
                use std::io::Read;
                let mut text = String::new();
                if let Ok(f) = std::fs::File::open(path) {
                    let _ = f.take(128 * 1024).read_to_string(&mut text);
                }
                let view = gtk::TextView::builder()
                    .editable(false)
                    .monospace(true)
                    .wrap_mode(gtk::WrapMode::WordChar)
                    .left_margin(20)
                    .right_margin(20)
                    .top_margin(20)
                    .build();
                view.buffer().set_text(&text);
                window.set_child(Some(&gtk::ScrolledWindow::builder().child(&view).build()));
            }
            _ => {
                open(path);
                return;
            }
        }
        window.present();
    }
    fn show_formats(self: &Rc<Self>) {
        let window = gtk::Window::builder()
            .application(&self.app)
            .transient_for(&self.window)
            .modal(true)
            .title("Supported formats")
            .default_width(620)
            .default_height(650)
            .build();
        let header = gtk::HeaderBar::new();
        header.set_show_title_buttons(false);
        header.set_title_widget(Some(&label("Supported formats", "status-title")));
        let close = gtk::Button::with_label("×");
        close.set_tooltip_text(Some("Close formats"));
        let w = window.downgrade();
        close.connect_clicked(move |_| {
            if let Some(w) = w.upgrade() {
                w.close();
            }
        });
        header.pack_end(&close);
        window.set_titlebar(Some(&header));
        let keys = gtk::EventControllerKey::new();
        let w = window.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(w) = w.upgrade() {
                    w.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(keys);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(20);
        content.set_margin_bottom(24);
        for (title, kind) in [
            ("Images", Kind::Image),
            ("Audio", Kind::Audio),
            ("Video", Kind::Video),
            ("PDF", Kind::Pdf),
            ("Documents", Kind::Document),
            ("Slides", Kind::Slides),
            ("Spreadsheets", Kind::Spreadsheet),
            ("Subtitles", Kind::Subtitle),
            ("Archives", Kind::Archive),
        ] {
            let section = gtk::Box::new(gtk::Orientation::Vertical, 4);
            section.append(&label(title, "status-title"));
            let note = label(engine::format_note(kind), "muted");
            note.set_wrap(true);
            note.set_max_width_chars(68);
            section.append(&note);
            content.append(&section);
        }
        let note = label(
            "Pages import depends on the file version. EPUB import/export uses reflowed text; RTFD imports its RTF content. Text clippings may need their macOS ._ sidecar. PDF text exports lose layout; XLSX uses one text line per row. Scanned PDFs need OCR before text export. Media codecs depend on the installed engines.",
            "muted",
        );
        note.set_wrap(true);
        content.append(&note);
        window.set_child(Some(
            &gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&content)
                .build(),
        ));
        self.attach_feedback();
        window.present();
    }
    fn show_dependencies(&self) {
        let mut lines = self
            .tools
            .0
            .iter()
            .map(|(name, path)| format!("✓ {name}  ·  {}", path.display()))
            .collect::<Vec<_>>();
        for name in [
            "magick",
            "ffmpeg",
            "libreoffice",
            "pdftoppm",
            "pdftotext",
            "python3",
            "qpdf",
            "bsdtar",
            "sha256sum",
        ] {
            if !self.tools.has(name) {
                lines.push(format!("— {name}  ·  not installed"));
            }
        }
        lines.push(
            if self.layer_supported {
                "✓ Wayland floating target supported"
            } else {
                "Floating target uses a regular window on this compositor"
            }
            .into(),
        );
        let dialog = gtk::AlertDialog::builder()
            .message("Conversion engines")
            .detail(lines.join("\n"))
            .buttons(["Close"])
            .build();
        dialog.show(Some(&self.window));
    }
}
fn label(text: &str, class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.);
    label.add_css_class(class);
    label
}
fn drop_paths(value: Option<&glib::Value>) -> Option<Vec<PathBuf>> {
    let list = value?.get::<gdk::FileList>().ok()?;
    let paths = list
        .files()
        .iter()
        .filter_map(|f| f.path())
        .collect::<Vec<_>>();
    if paths.is_empty() { None } else { Some(paths) }
}
fn open(path: &std::path::Path) {
    let _ = gio::AppInfo::launch_default_for_uri(
        &gio::File::for_path(path).uri(),
        gio::AppLaunchContext::NONE,
    );
}
fn human_size(size: u64) -> String {
    if size >= 1_000_000 {
        format!("{:.1} MB", size as f64 / 1_000_000.)
    } else if size >= 1000 {
        format!("{} KB", size / 1000)
    } else {
        format!("{size} B")
    }
}

fn tray(ui: &Rc<Ui>) {
    let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
        return;
    };
    let xml = r#"<node><interface name="org.kde.StatusNotifierItem"><method name="Activate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method><method name="SecondaryActivate"><arg type="i" direction="in"/><arg type="i" direction="in"/></method><method name="ContextMenu"><arg type="i" direction="in"/><arg type="i" direction="in"/></method><property name="Category" type="s" access="read"/><property name="Id" type="s" access="read"/><property name="Title" type="s" access="read"/><property name="Status" type="s" access="read"/><property name="IconName" type="s" access="read"/><property name="IconThemePath" type="s" access="read"/><property name="ItemIsMenu" type="b" access="read"/><property name="Menu" type="o" access="read"/></interface></node>"#;
    let Ok(info) = gio::DBusNodeInfo::for_xml(xml) else {
        return;
    };
    let Some(interface) = info.lookup_interface("org.kde.StatusNotifierItem") else {
        return;
    };
    let u = ui.clone();
    let registered = bus
        .register_object("/StatusNotifierItem", &interface)
        .method_call(move |_, _, _, _, method, _, invocation| {
            if method == "SecondaryActivate" {
                u.pad.set_visible(!u.pad.is_visible());
            } else {
                u.window.present();
            }
            invocation.return_value(None);
        })
        .property(|_, _, _, _, name| match name {
            "Category" => "ApplicationStatus".to_variant(),
            "Id" => "filewheel".to_variant(),
            "Title" => "FileWheel".to_variant(),
            "Status" => "Active".to_variant(),
            "IconName" => "filewheel".to_variant(),
            "IconThemePath" => format!(
                "{}/.local/share/icons/hicolor/scalable/apps",
                std::env::var("HOME").unwrap_or_default()
            )
            .to_variant(),
            "ItemIsMenu" => false.to_variant(),
            "Menu" => glib::variant::ObjectPath::try_from("/")
                .unwrap()
                .to_variant(),
            _ => "".to_variant(),
        })
        .build();
    if registered.is_ok() {
        bus.call(
            Some("org.kde.StatusNotifierWatcher"),
            "/StatusNotifierWatcher",
            "org.kde.StatusNotifierWatcher",
            "RegisterStatusNotifierItem",
            Some(&("/StatusNotifierItem",).to_variant()),
            None,
            gio::DBusCallFlags::NONE,
            2000,
            gio::Cancellable::NONE,
            |_| {},
        );
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|a| a == "--sound-test") {
        match feedback::self_test() {
            Ok(()) => println!("PASS: desktop hover-sound playback."),
            Err(error) => {
                eprintln!("FAIL: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    if args.iter().any(|a| a == "--self-test") {
        match smoke::backend() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("FAIL: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    if args.iter().any(|a| a == "--help") {
        println!(
            "FileWheel — local file conversion for Linux\nUsage: filewheel [FILES...] [--background]\n       filewheel --self-test\n       filewheel --smoke-test\nOutputs are saved beside their originals. Shift: Convert; Ctrl: Tools; Alt: Actions."
        );
        return;
    }
    let testing = args.iter().any(|a| a == "--smoke-test");
    let background = args.iter().any(|a| a == "--background");
    let app = gtk::Application::builder()
        .application_id("local.danny.FileWheel")
        .flags(if testing {
            gio::ApplicationFlags::NON_UNIQUE
        } else {
            gio::ApplicationFlags::HANDLES_OPEN
        })
        .build();
    let current: Rc<RefCell<Option<Rc<Ui>>>> = Rc::new(RefCell::new(None));
    let state = current.clone();
    app.connect_activate(move |app| {
        if state.borrow().is_none() {
            let ui = Ui::new(app, testing);
            *state.borrow_mut() = Some(ui.clone());
            if testing {
                smoke::ui(&ui);
            } else if !background {
                ui.window.present();
            }
        } else if let Some(ui) = state.borrow().as_ref() {
            ui.window.present();
        }
    });
    let state = current.clone();
    app.connect_open(move |app, files, _| {
        app.activate();
        if let Some(ui) = state.borrow().as_ref() {
            ui.set_paths(files.iter().filter_map(|f| f.path()).collect(), true);
            ui.window.present();
        }
    });
    let state = current.clone();
    app.connect_shutdown(move |_| {
        if let Some(ui) = state.borrow().as_ref() {
            ui.hover_sound.stop();
        }
    });
    let mut argv = vec![args[0].clone()];
    argv.extend(
        args.iter()
            .skip(1)
            .filter(|a| !matches!(a.as_str(), "--background" | "--smoke-test"))
            .cloned(),
    );
    app.run_with_args(&argv);
}
