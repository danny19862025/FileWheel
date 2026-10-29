use gtk::{glib, prelude::*};

pub fn window(
    parent: &gtk::ApplicationWindow,
    name: &str,
    logo: &gtk::gdk::Texture,
    comments: &str,
) -> gtk::Window {
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .hide_on_close(true)
        .title(format!("About {name}"))
        .default_width(540)
        .default_height(620)
        .build();
    window.set_widget_name("filewheel-about-window");
    window.set_application(parent.application().as_ref());
    // An explicit X works regardless of the desktop's title-button layout.
    let header = gtk::HeaderBar::new();
    header.set_show_title_buttons(false);
    header.set_title_widget(Some(&gtk::Label::new(Some(&format!("About {name}")))));
    let close_x = gtk::Button::with_label("×");
    close_x.set_widget_name("about-close-x");
    close_x.set_tooltip_text(Some("Close About"));
    close_x.add_css_class("about-close");
    let weak = window.downgrade();
    close_x.connect_clicked(move |_| {
        if let Some(window) = weak.upgrade() {
            window.close();
        }
    });
    header.pack_end(&close_x);
    window.set_titlebar(Some(&header));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 16);
    body.set_margin_top(24);
    body.set_margin_bottom(24);
    body.set_margin_start(24);
    body.set_margin_end(24);
    let image = gtk::Image::from_paintable(Some(logo));
    image.set_pixel_size(56);
    body.append(&image);
    let title = gtk::Label::new(Some(name));
    title.add_css_class("about-title");
    body.append(&title);
    let version = gtk::Label::new(Some(&format!("Version {}", env!("CARGO_PKG_VERSION"))));
    version.add_css_class("muted");
    body.append(&version);
    if name == "FileWheel" {
        let author = gtk::Label::new(Some("Created by Danny Rudd"));
        author.set_widget_name("about-author");
        author.add_css_class("about-author");
        body.append(&author);

        let profile = gtk::LinkButton::new("https://github.com/danny19862025");
        profile.set_widget_name("about-github");
        profile.set_halign(gtk::Align::Center);
        profile.set_tooltip_text(Some("Open Danny Rudd’s GitHub profile"));
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        if let Ok(loader) = gtk::gdk_pixbuf::PixbufLoader::with_type("svg") {
            loader.set_size(48, 48);
            if loader.write(include_bytes!("../data/github.svg")).is_ok()
                && loader.close().is_ok()
                && let Some(pixbuf) = loader.pixbuf()
            {
                let logo = gtk::Image::from_pixbuf(Some(&pixbuf));
                logo.set_pixel_size(24);
                content.append(&logo);
            }
        }
        content.append(&gtk::Label::new(Some("github.com/danny19862025")));
        profile.set_child(Some(&content));
        body.append(&profile);
        crate::updates::add_controls(&body);
    }
    let description = gtk::Label::new(Some(comments));
    description.set_wrap(true);
    description.set_max_width_chars(60);
    description.set_xalign(0.0);
    description.set_selectable(true);
    body.append(&description);
    let close = gtk::Button::with_label("Close");
    close.set_widget_name("about-close");
    close.set_halign(gtk::Align::End);
    let weak = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(window) = weak.upgrade() {
            window.close();
        }
    });
    body.append(&close);
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&body)
        .build();
    window.set_child(Some(&scroll));
    let keys = gtk::EventControllerKey::new();
    let weak = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            if let Some(window) = weak.upgrade() {
                window.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(keys);
    window
}
