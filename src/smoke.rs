use crate::{
    Ui,
    engine::{self, Kind, Mode, Tools, WorkDir},
};
use gtk::{gdk, glib, prelude::*};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicBool,
    time::Duration,
};

fn command(program: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{program}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
fn image(dir: &Path) -> Result<PathBuf, String> {
    let path = dir.join("sample ' $ (x).png");
    command(
        "magick",
        &[
            "-size".as_ref(),
            "96x64".as_ref(),
            "gradient:#f89b62-#3d4360".as_ref(),
            path.as_os_str(),
        ],
    )?;
    Ok(path)
}

pub fn backend() -> Result<(), String> {
    let tools = Tools::discover();
    for name in [
        "magick",
        "ffmpeg",
        "pdftoppm",
        "libreoffice",
        "bsdtar",
        "sha256sum",
    ] {
        tools.get(name)?;
    }
    let work = WorkDir::new(&std::env::temp_dir())?;
    let source = image(&work.path)?;
    let original = std::fs::read(&source).map_err(|e| e.to_string())?;
    let cancel = AtomicBool::new(false);
    let mut count = 0;
    let mut apply = |path: &Path, mode, op| -> Result<engine::Completed, String> {
        let done = engine::run(path, mode, op, &tools, &cancel)
            .map_err(|e| format!("{} → {op}: {e}", path.display()))?;
        if let Some(ref output) = done.output {
            if !output.exists() {
                return Err(format!("Missing output for {op}"));
            }
        }
        count += 1;
        Ok(done)
    };
    let mut pdf = None;
    for op in engine::menu(Kind::Image, Mode::Convert) {
        let done = apply(&source, Mode::Convert, op)?;
        let output = done.output.unwrap();
        if op == "PDF" {
            pdf = Some(output);
        } else {
            command("magick", &["identify".as_ref(), output.as_os_str()])?;
        }
    }
    for op in engine::menu(Kind::Image, Mode::Tools) {
        let done = apply(&source, Mode::Tools, op)?;
        command(
            "magick",
            &[
                "identify".as_ref(),
                done.output.as_ref().unwrap().as_os_str(),
            ],
        )?;
    }
    if let Some(pdf) = pdf {
        for op in ["PNG", "JPG"] {
            let out = apply(&pdf, Mode::Convert, op)?.output.unwrap();
            if std::fs::read_dir(out).map_err(|e| e.to_string())?.count() != 1 {
                return Err("PDF page export did not produce one page".into());
            }
        }
    }
    let multi = work.path.join("two pages.pdf");
    command(
        "magick",
        &[source.as_os_str(), source.as_os_str(), multi.as_os_str()],
    )?;
    for op in ["PNG", "JPG"] {
        let out = apply(&multi, Mode::Convert, op)?.output.unwrap();
        if std::fs::read_dir(&out).map_err(|e| e.to_string())?.count() != 2 {
            return Err("Multi-page PDF export lost a page".into());
        }
    }
    if tools.has("qpdf") {
        apply(&multi, Mode::Tools, "Strip metadata")?;
    }
    let broken = work.path.join("broken.png");
    std::fs::write(&broken, b"not an image").map_err(|e| e.to_string())?;
    if engine::run(&broken, Mode::Convert, "JPG", &tools, &cancel).is_ok()
        || work.path.join("broken.jpg").exists()
    {
        return Err("Failed conversion left a result behind".into());
    }
    let archive = apply(&source, Mode::Tools, "ZIP")?.output.unwrap();
    command("bsdtar", &["-tf".as_ref(), archive.as_os_str()])?;
    let hash = apply(&source, Mode::Tools, "SHA-256")?.clipboard.unwrap();
    if hash.len() != 64 {
        return Err("Invalid hash".into());
    }
    let audio = work.path.join("tone.wav");
    command(
        "ffmpeg",
        &[
            "-v".as_ref(),
            "error".as_ref(),
            "-f".as_ref(),
            "lavfi".as_ref(),
            "-i".as_ref(),
            "sine=frequency=440:duration=0.7".as_ref(),
            audio.as_os_str(),
        ],
    )?;
    for op in engine::menu(Kind::Audio, Mode::Convert)
        .into_iter()
        .chain(engine::menu(Kind::Audio, Mode::Tools))
    {
        let mode = if engine::menu(Kind::Audio, Mode::Convert).contains(&op) {
            Mode::Convert
        } else {
            Mode::Tools
        };
        let out = apply(&audio, mode, op)?.output.unwrap();
        command(
            "ffprobe",
            &["-v".as_ref(), "error".as_ref(), out.as_os_str()],
        )?;
    }
    let video = work.path.join("clip.mp4");
    command(
        "ffmpeg",
        &[
            "-v".as_ref(),
            "error".as_ref(),
            "-f".as_ref(),
            "lavfi".as_ref(),
            "-i".as_ref(),
            "testsrc=size=96x64:rate=12:duration=0.7".as_ref(),
            "-f".as_ref(),
            "lavfi".as_ref(),
            "-i".as_ref(),
            "sine=frequency=440:duration=0.7".as_ref(),
            "-c:v".as_ref(),
            "libx264".as_ref(),
            "-pix_fmt".as_ref(),
            "yuv420p".as_ref(),
            "-c:a".as_ref(),
            "aac".as_ref(),
            "-shortest".as_ref(),
            video.as_os_str(),
        ],
    )?;
    for op in engine::menu(Kind::Video, Mode::Convert)
        .into_iter()
        .chain(engine::menu(Kind::Video, Mode::Tools))
    {
        let mode = if engine::menu(Kind::Video, Mode::Convert).contains(&op) {
            Mode::Convert
        } else {
            Mode::Tools
        };
        let out = apply(&video, mode, op)?.output.unwrap();
        command(
            "ffprobe",
            &["-v".as_ref(), "error".as_ref(), out.as_os_str()],
        )?;
    }
    let document = work.path.join("notes.v2.final.txt");
    std::fs::write(&document, "Hello Linux.\nThese are four words.\n")
        .map_err(|e| e.to_string())?;
    for op in ["TXT", "MD", "HTML", "DOCX", "EPUB"] {
        let done = apply(&document, Mode::Convert, op)?;
        if op == "DOCX"
            && !apply(done.output.as_ref().unwrap(), Mode::Tools, "Copy text")?
                .clipboard
                .unwrap()
                .contains("Hello Linux")
        {
            return Err("DOCX round trip lost text".into());
        }
    }
    if apply(&document, Mode::Tools, "Word count")?
        .clipboard
        .as_deref()
        != Some("6")
    {
        return Err("Wrong document word count".into());
    }
    if !apply(&document, Mode::Tools, "Copy text")?
        .clipboard
        .unwrap()
        .contains("Hello Linux")
    {
        return Err("Document text was lost".into());
    }

    // Use real LibreOffice/coreutils on a readable source in a non-writable folder.
    {
        use std::os::unix::fs::PermissionsExt;
        let dir = work.path.join("read-only");
        std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
        let input = dir.join("notes.v2.txt");
        std::fs::copy(&document, &input).map_err(|e| e.to_string())?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555))
            .map_err(|e| e.to_string())?;
        let result: Result<(), String> = (|| {
            let hash = apply(&input, Mode::Tools, "SHA-256")?.clipboard.unwrap();
            let text = apply(&input, Mode::Tools, "Copy text")?.clipboard.unwrap();
            let words = apply(&input, Mode::Tools, "Word count")?.clipboard.unwrap();
            if hash.len() != 64 || !text.contains("Hello Linux") || words != "6" {
                return Err("Clipboard operations failed on read-only input".into());
            }
            if std::fs::read_dir(&dir).map_err(|e| e.to_string())?.count() != 1 {
                return Err("Clipboard operation wrote beside read-only input".into());
            }
            Ok(())
        })();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        result?;
    }

    // Exercise every newly added family against real files, with content/page checks.
    let epub = apply(&document, Mode::Convert, "EPUB")?.output.unwrap();
    if !apply(&epub, Mode::Tools, "Copy text")?
        .clipboard
        .unwrap()
        .contains("Hello Linux")
    {
        return Err("EPUB reading-order import lost text".into());
    }
    let rtfd = work.path.join("rich.rtfd");
    std::fs::create_dir(&rtfd).map_err(|e| e.to_string())?;
    std::fs::write(rtfd.join("TXT.rtf"), r"{\rtf1\ansi Hello RTFD bundle.}")
        .map_err(|e| e.to_string())?;
    if !apply(&rtfd, Mode::Tools, "Copy text")?
        .clipboard
        .unwrap()
        .contains("Hello RTFD")
    {
        return Err("RTFD import lost text".into());
    }
    let clipping = work.path.join("clip.textClipping");
    std::fs::write(&clipping, "Text clipping sample.").map_err(|e| e.to_string())?;
    apply(&clipping, Mode::Convert, "DOCX")?;
    let csv = work.path.join("cells.csv");
    std::fs::write(&csv, "Name,Value\nCafé,42\n\"comma, cell\",7\n").map_err(|e| e.to_string())?;
    let xlsx = apply(&csv, Mode::Convert, "XLSX")?.output.unwrap();
    for op in engine::menu(Kind::Spreadsheet, Mode::Convert) {
        let out = apply(&xlsx, Mode::Convert, op)?.output.unwrap();
        if op == "TSV" {
            let content = std::fs::read_to_string(&out).map_err(|e| e.to_string())?;
            if !content.contains("Café\t42") || !content.contains("comma, cell") {
                return Err(format!(
                    "Spreadsheet export lost Unicode/cell delimiters: {content}"
                ));
            }
            apply(&out, Mode::Convert, "XLSX")?;
        }
    }
    let srt = work.path.join("captions.srt");
    std::fs::write(&srt, "1\n00:00:00,000 --> 00:00:01,000\nHello subtitles.\n\n2\n00:00:01,000 --> 00:00:02,000\nSecond cue.\n").map_err(|e| e.to_string())?;
    for op in engine::menu(Kind::Subtitle, Mode::Convert) {
        let out = apply(&srt, Mode::Convert, op)?.output.unwrap();
        if op == "TXT"
            && std::fs::read_to_string(&out).map_err(|e| e.to_string())?
                != "Hello subtitles.\nSecond cue.\n"
        {
            return Err("Subtitle TXT contains timing data or lost cues".into());
        }
        if op == "VTT" {
            apply(&out, Mode::Convert, "SRT")?;
            apply(&out, Mode::Convert, "TXT")?;
        }
    }
    let flat_slides = work.path.join("slides.fodp");
    std::fs::write(&flat_slides, r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0" office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.presentation"><office:body><office:presentation>
<draw:page draw:name="First"><draw:frame svg:x="1cm" svg:y="1cm" svg:width="20cm" svg:height="3cm"><draw:text-box><text:p>First slide</text:p></draw:text-box></draw:frame></draw:page>
<draw:page draw:name="Second"><draw:frame svg:x="1cm" svg:y="1cm" svg:width="20cm" svg:height="3cm"><draw:text-box><text:p>Second slide</text:p></draw:text-box></draw:frame></draw:page>
</office:presentation></office:body></office:document>"#).map_err(|e| e.to_string())?;
    let profile = format!(
        "-env:UserInstallation={}",
        gtk::gio::File::for_path(work.path.join("fixture-profile")).uri()
    );
    command(
        "libreoffice",
        &[
            profile.as_ref(),
            "--headless".as_ref(),
            "--convert-to".as_ref(),
            "pptx".as_ref(),
            "--outdir".as_ref(),
            work.path.as_os_str(),
            flat_slides.as_os_str(),
        ],
    )?;
    let slides = work.path.join("slides.pptx");
    let mut slides_pdf = None;
    for op in engine::menu(Kind::Slides, Mode::Convert) {
        let out = apply(&slides, Mode::Convert, op)?.output.unwrap();
        if op == "PDF" {
            slides_pdf = Some(out);
        } else if std::fs::read_dir(out).map_err(|e| e.to_string())?.count() != 2 {
            return Err("Slide image export lost a slide".into());
        }
    }
    let slides_pdf = slides_pdf.unwrap();
    for op in engine::menu(Kind::Pdf, Mode::Convert) {
        if tools.unavailable(Kind::Pdf, Mode::Convert, op).is_none() {
            apply(&slides_pdf, Mode::Convert, op)?;
        }
    }
    let zip = apply(&document, Mode::Tools, "ZIP")?.output.unwrap();
    for op in engine::menu(Kind::Archive, Mode::Convert) {
        let out = apply(&zip, Mode::Convert, op)?.output.unwrap();
        command("bsdtar", &["-tf".as_ref(), out.as_os_str()])?;
        if op == "GZIP" {
            apply(&out, Mode::Convert, "ZIP")?;
        }
    }
    let raw_gzip = work.path.join("raw.txt.gz");
    command(
        "python3",
        &[
            "-c".as_ref(),
            "import gzip,sys; gzip.open(sys.argv[1],'wb').write(b'Raw gzip contents')".as_ref(),
            raw_gzip.as_os_str(),
        ],
    )?;
    let raw_zip = apply(&raw_gzip, Mode::Convert, "ZIP")?.output.unwrap();
    command("bsdtar", &["-tf".as_ref(), raw_zip.as_os_str()])?;

    let folder = work.path.join("a folder");
    std::fs::create_dir(&folder).map_err(|e| e.to_string())?;
    std::fs::write(folder.join("inside.txt"), "preserved").map_err(|e| e.to_string())?;
    apply(&folder, Mode::Convert, "ZIP")?;
    if std::fs::read(&source).map_err(|e| e.to_string())? != original {
        return Err("Original image was changed".into());
    }
    if std::fs::read_dir(&work.path)
        .map_err(|e| e.to_string())?
        .any(|e| {
            e.ok()
                .map(|e| e.file_name().to_string_lossy().starts_with(".filewheel-"))
                .unwrap_or(false)
        })
    {
        return Err("Temporary job folders leaked".into());
    }
    println!(
        "PASS: {count} real conversion/tool jobs; outputs decoded, original preserved, quoted filename handled, temporary files cleaned."
    );
    Ok(())
}

pub fn ui(ui: &Rc<Ui>) {
    let result = (|| {
        let work = WorkDir::new(&std::env::temp_dir())?;
        let source = image(&work.path)?;
        Ok::<_, String>((work, source))
    })();
    let (work, source) = match result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("UI smoke fixture: {e}");
            ui.app.quit();
            return;
        }
    };
    ui.set_paths(vec![source], false);
    ui.window.present();
    let font = ui
        .window
        .pango_context()
        .load_font(&gtk::pango::FontDescription::from_string("Ribeye 24"))
        .expect("Bundled font");
    assert_eq!(
        font.describe().family().as_deref(),
        Some("Ribeye"),
        "Bundled font must render without fallback"
    );
    let u = ui.clone();
    glib::timeout_add_local_once(Duration::from_millis(850), move || {
        if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
            let snapshot = gtk::Snapshot::new();
            gtk::WidgetPaintable::new(Some(&u.window)).snapshot(
                &snapshot,
                u.window.width() as f64,
                u.window.height() as f64,
            );
            if let (Some(node), Some(renderer)) = (snapshot.to_node(), u.window.renderer()) {
                renderer
                    .render_texture(&node, None)
                    .save_to_png(PathBuf::from(path).with_file_name("filewheel-opening.png"))
                    .expect("Opening screenshot");
            }
        }
    });
    let u = ui.clone();
    glib::timeout_add_local_once(Duration::from_millis(4000), move || {
        let overlay = u.window.child().and_downcast::<gtk::Overlay>().unwrap();
        assert!(
            overlay.child().unwrap().next_sibling().is_none(),
            "Opening animation must remove itself"
        );
        let mut widgets = vec![u.window.titlebar().unwrap()];
        let title = loop {
            let widget = widgets.pop().expect("FileWheel wordmark");
            if widget.widget_name() == "filewheel-wordmark" {
                break widget.downcast::<gtk::Label>().unwrap();
            }
            let mut child = widget.first_child();
            while let Some(widget) = child {
                child = widget.next_sibling();
                widgets.push(widget);
            }
        };
        assert_eq!(
            title.text(),
            "FileWheel",
            "Typewriter must finish without a caret"
        );
        // Exercise the same GTK signal handler used for compositor-delivered drops.
        // This does not claim to synthesize a physical Wayland drag.
        u.mode_buttons[1].emit_clicked();
        assert_eq!(u.mode.get(), Mode::Tools);
        assert!(u.choices().contains(&"Resize 1280"));
        u.mode_buttons[0].emit_clicked();
        let files = gdk::FileList::from_array(&[gtk::gio::File::for_path(&u.paths.borrow()[0])]);
        let controllers = u.ring.observe_controllers();
        let target = (0..controllers.n_items())
            .filter_map(|i| controllers.item(i).and_downcast::<gtk::DropTarget>())
            .next()
            .expect("Wheel drop target");
        let size = u.ring.width().min(u.ring.height()) as f64;
        let angle = -std::f64::consts::PI / 2. + std::f64::consts::TAU / u.choices().len() as f64;
        let x = size / 2. + angle.cos() * size * 0.34;
        let y = size / 2. + angle.sin() * size * 0.34;
        assert!(
            target.emit_by_name::<bool>("drop", &[&glib::BoxedValue(files.to_value()), &x, &y])
        );
    });
    let u = ui.clone();
    let mut ticks = 0;
    let mut owner = Some(work);
    glib::timeout_add_local(Duration::from_millis(200), move || {
        ticks += 1;
        if ticks < 22 {
            return glib::ControlFlow::Continue;
        }
        if u.busy.get() && ticks < 100 {
            return glib::ControlFlow::Continue;
        }
        if u.results.borrow().len() != 1 {
            eprintln!("UI smoke: conversion did not finish: {}", u.detail.text());
            std::process::exit(1);
        }
        let value = glib::Value::from(&gdk::FileList::from_array(&[gtk::gio::File::for_path(
            &u.paths.borrow()[0],
        )]));
        if crate::drop_paths(Some(&value)).map(|p| p.len()) != Some(1) {
            eprintln!("UI smoke: file-list decoding failed");
            std::process::exit(1);
        }
        if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
            let snapshot = gtk::Snapshot::new();
            gtk::WidgetPaintable::new(Some(&u.window)).snapshot(
                &snapshot,
                u.window.width() as f64,
                u.window.height() as f64,
            );
            if let Some(node) = snapshot.to_node() {
                if let Some(renderer) = u.window.renderer() {
                    if let Err(e) = renderer.render_texture(&node, None).save_to_png(path) {
                        eprintln!("Screenshot: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        u.pad.set_visible(true);
        u.floating.set_visible(true);
        println!(
            "PASS: native GTK window, mode switching, drop signal routing, asynchronous conversion and result display; layer shell: {}.",
            u.layer_supported
        );
        let u = u.clone();
        let fixture = owner.take();
        glib::timeout_add_local_once(Duration::from_millis(500), move || {
            if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
                let path = PathBuf::from(path);
                for (widget, name) in [
                    (&u.floating, "filewheel-floating.png"),
                    (&u.pad, "filewheel-target.png"),
                ] {
                    let snapshot = gtk::Snapshot::new();
                    gtk::WidgetPaintable::new(Some(widget)).snapshot(
                        &snapshot,
                        widget.width() as f64,
                        widget.height() as f64,
                    );
                    if let (Some(node), Some(renderer)) = (snapshot.to_node(), widget.renderer()) {
                        renderer
                            .render_texture(&node, None)
                            .save_to_png(path.with_file_name(name))
                            .expect("Surface screenshot");
                    }
                }
            }
            assert!(u.about.child().is_some(), "About must have its content");
            u.about.present();
            glib::timeout_add_local_once(Duration::from_millis(400), move || {
                if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
                    let snapshot = gtk::Snapshot::new();
                    gtk::WidgetPaintable::new(Some(&u.about)).snapshot(
                        &snapshot,
                        u.about.width() as f64,
                        u.about.height() as f64,
                    );
                    if let (Some(node), Some(renderer)) = (snapshot.to_node(), u.about.renderer()) {
                        renderer
                            .render_texture(&node, None)
                            .save_to_png(PathBuf::from(path).with_file_name("filewheel-about.png"))
                            .expect("About screenshot");
                    }
                }
                let mut widgets = vec![u.about.titlebar().expect("About header")];
                let close = loop {
                    let widget = widgets.pop().expect("About × close button");
                    if let Some(button) = widget.downcast_ref::<gtk::Button>() {
                        if button.label().as_deref() == Some("×") {
                            break button.clone();
                        }
                    }
                    let mut child = widget.first_child();
                    while let Some(widget) = child {
                        child = widget.next_sibling();
                        widgets.push(widget);
                    }
                };
                close.emit_clicked();
                assert!(!u.about.is_visible(), "About × must hide the dialog");
                u.about.present();
                assert!(u.about.is_visible(), "About must reopen");
                close.emit_clicked();
                assert!(!u.about.is_visible(), "About × must work after reopening");
                println!("PASS: About embedded logo, × close button, and reopening.");
                let pdf = u.paths.borrow()[0].with_extension("pdf");
                std::fs::write(&pdf, b"UI-only PDF menu fixture").unwrap();
                u.set_paths(vec![pdf], false);
                assert_eq!(u.choices().len(), 9);
                assert!(
                    u.disabled("PAGES")
                        .unwrap()
                        .contains("unavailable on Linux")
                );
                u.show_formats();
                glib::timeout_add_local_once(Duration::from_millis(400), move || {
                    let formats = u
                        .app
                        .windows()
                        .into_iter()
                        .find(|w| w.title().as_deref() == Some("Supported formats"))
                        .expect("Formats window");
                    if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
                        for (widget, name) in [
                            (&formats, "filewheel-formats.png"),
                            (
                                u.window.upcast_ref::<gtk::Window>(),
                                "filewheel-pdf-menu.png",
                            ),
                        ] {
                            let snapshot = gtk::Snapshot::new();
                            gtk::WidgetPaintable::new(Some(widget)).snapshot(
                                &snapshot,
                                widget.width() as f64,
                                widget.height() as f64,
                            );
                            if let (Some(node), Some(renderer)) =
                                (snapshot.to_node(), widget.renderer())
                            {
                                renderer
                                    .render_texture(&node, None)
                                    .save_to_png(PathBuf::from(&path).with_file_name(name))
                                    .expect("Formats screenshot");
                            }
                        }
                    }
                    formats.close();
                    println!(
                        "PASS: nine-format PDF menu, unavailable Pages export, and Formats reference window."
                    );
                    appearance_ui(u, fixture);
                });
            });
        });
        glib::ControlFlow::Break
    });
}

fn appearance_snapshot(widget: &impl IsA<gtk::Widget>, name: &str) {
    if let Some(path) = std::env::var_os("FILEWHEEL_SCREENSHOT") {
        let widget = widget.as_ref();
        let snapshot = gtk::Snapshot::new();
        gtk::WidgetPaintable::new(Some(widget)).snapshot(
            &snapshot,
            widget.width() as f64,
            widget.height() as f64,
        );
        let window = widget
            .downcast_ref::<gtk::Window>()
            .expect("Screenshot window");
        let node = snapshot.to_node().expect("Appearance render node");
        window
            .renderer()
            .unwrap()
            .render_texture(&node, None)
            .save_to_png(PathBuf::from(path).with_file_name(name))
            .expect("Appearance screenshot");
    }
}

fn appearance_ui(ui: Rc<Ui>, fixture: Option<WorkDir>) {
    use crate::appearance::{ACCENTS, Color, Preferences, Theme};
    let file = fixture.as_ref().unwrap().path.join("appearance.conf");
    *ui.appearance_file.borrow_mut() = Some(file.clone());
    let image = ui.paths.borrow()[0].with_extension("png");
    ui.set_paths(vec![image], false);
    ui.show_appearance();
    let (theme, blue, hex, custom, sound, glow, reset, window, preview, preview_label) = {
        let dialog = ui.appearance_dialog.borrow();
        let d = dialog.as_ref().unwrap();
        (
            d.theme.clone(),
            d.swatches[5].1.clone(),
            d.hex.clone(),
            d.custom.clone(),
            d.sound.clone(),
            d.glow.clone(),
            d.reset.clone(),
            d.window.clone(),
            d.preview.clone(),
            d.preview_label.clone(),
        )
    };
    theme.set_selected(1);
    blue.emit_clicked();
    assert_eq!(ui.appearance.get().theme, Theme::Light);
    assert_eq!(ui.appearance.get().accent, ACCENTS[5].1);
    assert_eq!(Preferences::load(&file), ui.appearance.get());
    hex.set_text("#oops");
    hex.emit_activate();
    assert!(hex.has_css_class("invalid"));
    assert_eq!(ui.appearance.get().accent, ACCENTS[5].1);
    hex.set_text("#123456");
    hex.emit_activate();
    assert_eq!(ui.appearance.get().accent, Color(18, 52, 86));
    assert!(!hex.has_css_class("invalid"));
    custom.set_rgba(&gdk::RGBA::new(87. / 255., 157. / 255., 246. / 255., 1.));
    assert_eq!(ui.appearance.get().accent, ACCENTS[5].1);
    sound.set_active(false);
    glow.set_active(false);
    assert!(!ui.appearance.get().hover_sound);
    assert!(!ui.appearance.get().hover_glow);
    assert!(ui.window.has_css_class("no-hover-glow"));
    assert_eq!(Preferences::load(&file), ui.appearance.get());
    preview.emit_clicked();
    assert_eq!(preview_label.text(), "Turn on an effect above.");
    assert!(!preview.has_css_class("effect-preview-active"));
    sound.set_active(true);
    preview.emit_clicked();
    assert_eq!(preview_label.text(), "Sound preview");
    assert!(!preview.has_css_class("effect-preview-active"));
    sound.set_active(false);
    glow.set_active(true);
    preview.emit_clicked();
    assert_eq!(preview_label.text(), "Glow preview");
    assert!(preview.has_css_class("effect-preview-active"));
    sound.set_active(true);
    let saved = std::fs::read_to_string(&file).unwrap();
    preview.emit_clicked();
    assert_eq!(preview_label.text(), "Sound + glow");
    assert!(preview.has_css_class("effect-preview-active"));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), saved);
    // Dispatch a real controller enter signal, not just a direct audio call.
    let button = ui.mode_buttons[0].clone();
    let controllers = button.observe_controllers();
    let motion = (0..controllers.n_items())
        .filter_map(|i| {
            controllers
                .item(i)
                .and_downcast::<gtk::EventControllerMotion>()
        })
        .next()
        .expect("Button hover controller");
    motion.emit_by_name::<()>("enter", &[&0f64, &0f64]);
    ui.hover.set(Some(0));
    ui.redraw();
    glib::timeout_add_local_once(Duration::from_millis(450), move || {
        appearance_snapshot(&ui.window, "filewheel-light.png");
        appearance_snapshot(&window, "filewheel-appearance-light.png");
        appearance_snapshot(&ui.floating, "filewheel-floating-light.png");
        appearance_snapshot(&ui.pad, "filewheel-target-light.png");
        theme.set_selected(0);
        ui.set_appearance(Preferences {
            accent: ACCENTS[6].1,
            ..ui.appearance.get()
        });
        glib::timeout_add_local_once(Duration::from_millis(450), move || {
            appearance_snapshot(&ui.window, "filewheel-dark-violet.png");
            appearance_snapshot(&window, "filewheel-appearance-dark.png");
            // Repeated preview clicks restart one temporary pulse, then return
            // to the ordinary appearance without changing saved preferences.
            assert!(!preview.has_css_class("effect-preview-active"));
            assert_eq!(preview_label.text(), "Effect preview");
            theme.set_selected(2);
            ui.set_system_scheme(2);
            assert_eq!(
                ui.palette().bg,
                crate::appearance::Palette::new(
                    Preferences {
                        theme: Theme::Light,
                        ..ui.appearance.get()
                    },
                    false
                )
                .bg
            );
            ui.set_system_scheme(1);
            assert_eq!(
                ui.palette().bg,
                crate::appearance::Palette::new(
                    Preferences {
                        theme: Theme::Dark,
                        ..ui.appearance.get()
                    },
                    true
                )
                .bg
            );
            reset.emit_clicked();
            assert_eq!(ui.appearance.get(), Preferences::default());
            assert_eq!(Preferences::load(&file), Preferences::default());
            window.close();
            assert!(!window.is_visible());
            ui.show_appearance();
            assert!(window.is_visible());
            window.close();
            *ui.appearance_file.borrow_mut() = None;
            println!(
                "PASS: dark/light/system appearance, presets, custom picker/hex validation, persisted hover toggles, reset, button hover routing, effect-preview clicks/toggle combinations/timeout and reopening."
            );
            // Show the rounded action buttons with the reference's light/orange
            // palette, without changing the user's saved appearance.
            ui.mode_buttons[2].emit_clicked();
            ui.set_appearance(Preferences {
                theme: Theme::Light,
                ..Preferences::default()
            });
            ui.hover.set(Some(1));
            ui.redraw();
            glib::timeout_add_local_once(Duration::from_millis(450), move || {
                appearance_snapshot(&ui.window, "filewheel-rounded-buttons.png");
                appearance_snapshot(&ui.floating, "filewheel-rounded-floating.png");
                ui.set_paths(vec![], false);
                ui.start("Open");
                assert_eq!(ui.status.text(), "Not available");
                assert_eq!(ui.detail.text(), "Add a file first");
                glib::timeout_add_local_once(Duration::from_millis(250), move || {
                    appearance_snapshot(&ui.window, "filewheel-light-unavailable.png");
                    ui.set_appearance(Preferences::default());
                    glib::timeout_add_local_once(Duration::from_millis(250), move || {
                        appearance_snapshot(&ui.window, "filewheel-dark-unavailable.png");
                        println!("PASS: unavailable-operation status in light and dark themes.");
                        drop(fixture);
                        ui.app.quit();
                    });
                });
            });
        });
    });
}
