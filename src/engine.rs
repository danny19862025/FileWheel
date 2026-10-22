use gtk::gio::prelude::FileExt;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, String>;
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Video,
    Audio,
    Document,
    Pdf,
    Slides,
    Spreadsheet,
    Subtitle,
    Archive,
    Other,
    Folder,
}
impl Kind {
    pub fn of(path: &Path) -> Self {
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        if path.is_dir() && !matches!(ext.as_str(), "pages" | "rtfd") {
            return Self::Folder;
        }
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "heic" | "heif" | "tif" | "tiff" | "avif" | "bmp"
            | "svg" => Self::Image,
            "mp4" | "mov" | "mkv" | "avi" | "wmv" | "webm" | "m4v" | "gif" => Self::Video,
            "mp3" | "m4a" | "wav" | "flac" | "ogg" | "opus" | "aif" | "aiff" | "wma" | "aac" => {
                Self::Audio
            }
            "txt" | "md" | "doc" | "docx" | "rtf" | "html" | "htm" | "odt" | "pages"
            | "markdown" | "epub" | "rtfd" | "textclipping" => Self::Document,
            "pdf" => Self::Pdf,
            "ppt" | "pptx" | "odp" => Self::Slides,
            "xls" | "xlsx" | "ods" | "csv" | "tsv" => Self::Spreadsheet,
            "srt" | "vtt" => Self::Subtitle,
            "zip" | "tar" | "gz" | "gzip" | "tgz" | "rar" => Self::Archive,
            _ => Self::Other,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Document => "Document",
            Self::Pdf => "PDF",
            Self::Slides => "Slides",
            Self::Spreadsheet => "Spreadsheet",
            Self::Subtitle => "Subtitle",
            Self::Archive => "Archive",
            Self::Folder => "Folder",
            Self::Other => "File",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Convert,
    Tools,
    Actions,
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Convert => "Convert",
            Self::Tools => "Tools",
            Self::Actions => "Actions",
        }
    }
}
pub fn menu(kind: Kind, mode: Mode) -> Vec<&'static str> {
    if mode == Mode::Actions {
        return vec!["Copy path", "Reveal", "Open", "Preview", "Trash"];
    }
    match (mode, kind) {
        (Mode::Convert, Kind::Image) => {
            vec!["JPG", "PNG", "WEBP", "HEIC", "TIFF", "AVIF", "BMP", "PDF"]
        }
        (Mode::Convert, Kind::Video) => {
            vec!["MP4", "MOV", "MKV", "WEBM", "AVI", "WMV", "GIF", "MP3"]
        }
        (Mode::Convert, Kind::Audio) => {
            vec!["MP3", "M4A", "WAV", "FLAC", "OGG", "OPUS", "AIFF", "WMA"]
        }
        (Mode::Convert, Kind::Document) => vec!["DOCX", "PAGES", "MD", "TXT", "EPUB", "HTML"],
        (Mode::Convert, Kind::Pdf) => vec![
            "DOCX", "PAGES", "PPTX", "XLSX", "JPG", "PNG", "EPUB", "TXT", "MD",
        ],
        (Mode::Convert, Kind::Slides) => vec!["PDF", "JPG", "PNG"],
        (Mode::Convert, Kind::Spreadsheet) => vec!["XLSX", "ODS", "CSV", "TSV", "PDF"],
        (Mode::Convert, Kind::Subtitle) => vec!["SRT", "VTT", "TXT"],
        (Mode::Convert, Kind::Archive) => vec!["ZIP", "TAR", "GZIP"],
        (Mode::Tools, Kind::Image) => vec![
            "Resize 1280",
            "Compress",
            "Strip metadata",
            "Rotate 90",
            "Grayscale",
        ],
        (Mode::Tools, Kind::Video) => vec![
            "Compress",
            "Strip metadata",
            "Mute",
            "First 10s",
            "Grab frame",
        ],
        (Mode::Tools, Kind::Audio) => vec![
            "Strip metadata",
            "Normalize",
            "First 30s",
            "Mono",
            "Compress",
        ],
        (Mode::Tools, Kind::Document) => vec!["Copy text", "Word count", "ZIP"],
        (Mode::Tools, Kind::Pdf) => vec!["ZIP", "Strip metadata"],
        (_, Kind::Folder) => vec!["ZIP", "TAR", "GZIP"],
        _ => vec!["ZIP", "SHA-256"],
    }
}

#[derive(Clone, Debug)]
pub struct Tools(pub BTreeMap<String, PathBuf>);
impl Tools {
    pub fn discover() -> Self {
        let mut found = BTreeMap::new();
        for name in [
            "magick",
            "ffmpeg",
            "libreoffice",
            "pdftoppm",
            "pdftotext",
            "qpdf",
            "bsdtar",
            "sha256sum",
            "python3",
        ] {
            if let Some(path) = find(name) {
                found.insert(name.into(), path);
            }
        }
        Self(found)
    }
    pub fn has(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
    pub fn get(&self, name: &str) -> Result<&Path> {
        self.0.get(name).map(PathBuf::as_path).ok_or_else(|| {
            format!("{name} is needed for this operation. Install it and reopen FileWheel.")
        })
    }
    pub fn unavailable_for(&self, input: &Path, mode: Mode, op: &str) -> Option<String> {
        if let Some(reason) = self.unavailable(Kind::of(input), mode, op) {
            return Some(reason);
        }
        let ext = input
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        if mode != Mode::Actions && op == "EPUB" && !self.has("python3") {
            return Some("Requires python3 for EPUB export".into());
        }
        if mode != Mode::Actions
            && matches!(ext.as_str(), "gz" | "gzip" | "tgz")
            && matches!(op, "ZIP" | "TAR" | "GZIP")
            && !self.has("python3")
        {
            return Some("Requires python3 to read gzip input".into());
        }
        if mode == Mode::Actions || matches!(op, "ZIP" | "TAR" | "GZIP" | "SHA-256") {
            return None;
        }
        if matches!(ext.as_str(), "epub" | "textclipping") && !self.has("python3") {
            return Some("Requires python3 to import EPUB/text clippings".into());
        }
        if ext == "pages" && input.is_dir() && !self.has("bsdtar") {
            return Some("Requires bsdtar to import a Pages bundle".into());
        }
        None
    }
    pub fn unavailable(&self, kind: Kind, mode: Mode, op: &str) -> Option<String> {
        if mode == Mode::Actions {
            return None;
        }
        if op == "PAGES" {
            return Some("Pages export is unavailable on Linux. Import a Pages file and export DOCX instead.".into());
        }
        let needed: &[&str] = if matches!(op, "ZIP" | "TAR" | "GZIP") {
            &["bsdtar"]
        } else if op == "SHA-256" {
            &["sha256sum"]
        } else if kind == Kind::Pdf && op == "Strip metadata" {
            &["qpdf"]
        } else if kind == Kind::Pdf && matches!(op, "PNG" | "JPG") {
            &["pdftoppm"]
        } else if kind == Kind::Pdf && op == "PPTX" {
            &["libreoffice"]
        } else if kind == Kind::Pdf && matches!(op, "TXT" | "MD") {
            &["pdftotext"]
        } else if kind == Kind::Pdf {
            &["pdftotext", "libreoffice"]
        } else if kind == Kind::Image {
            &["magick"]
        } else if matches!(kind, Kind::Video | Kind::Audio)
            || (kind == Kind::Subtitle && op != "TXT")
        {
            &["ffmpeg"]
        } else if kind == Kind::Subtitle {
            &[]
        } else if kind == Kind::Slides && matches!(op, "PNG" | "JPG") {
            &["libreoffice", "pdftoppm"]
        } else if matches!(kind, Kind::Document | Kind::Slides | Kind::Spreadsheet) {
            &["libreoffice"]
        } else {
            return Some("This operation is not available for this file.".into());
        };
        let missing = needed
            .iter()
            .filter(|name| !self.has(name))
            .copied()
            .collect::<Vec<_>>();
        if missing.is_empty() {
            None
        } else {
            Some(format!("Requires {}", missing.join(", ")))
        }
    }
}
fn find(name: &str) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|p| p.join(name))
        .find(|p| {
            p.is_file()
                && p.metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
        })
}

pub struct WorkDir {
    pub path: PathBuf,
}
impl WorkDir {
    pub fn new(parent: &Path) -> Result<Self> {
        for _ in 0..100 {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".filewheel-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
                        .map_err(|e| e.to_string())?;
                    return Ok(Self { path });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Cannot write in {}: {e}", parent.display())),
            }
        }
        Err("Cannot allocate a temporary folder".into())
    }
}
impl Drop for WorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct Output {
    path: PathBuf,
    directory: bool,
    committed: bool,
}
impl Output {
    fn reserve(input: &Path, suffix: &str, extension: &str, directory: bool) -> Result<Self> {
        let parent = input.parent().ok_or("No parent folder")?;
        let stem = if input.is_dir() {
            input.file_name()
        } else {
            input.file_stem()
        }
        .unwrap_or(OsStr::new("file"));
        for number in 1..100_000 {
            let mut name = stem.to_os_string();
            name.push(suffix);
            if number > 1 {
                name.push(format!(" {number}"));
            }
            if !extension.is_empty() {
                name.push(".");
                name.push(extension);
            }
            let path = parent.join(name);
            let result = if directory {
                fs::create_dir(&path)
            } else {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map(|_| ())
            };
            match result {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        directory,
                        committed: false,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Cannot save beside {}: {e}", input.display())),
            }
        }
        Err("Too many files with this name".into())
    }
    fn commit(mut self, staged: &Path) -> Result<PathBuf> {
        if self.directory {
            let entries = fs::read_dir(staged)
                .map_err(|e| e.to_string())?
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?;
            if entries.is_empty() {
                return Err("The converter produced no pages.".into());
            }
            for entry in entries {
                fs::rename(entry.path(), self.path.join(entry.file_name()))
                    .map_err(|e| e.to_string())?;
            }
        } else {
            if fs::metadata(staged).map(|m| m.len()).unwrap_or(0) == 0 {
                return Err("The converter produced an empty output.".into());
            }
            fs::rename(staged, &self.path).map_err(|e| e.to_string())?;
        }
        self.committed = true;
        Ok(self.path.clone())
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        if !self.committed {
            if self.directory {
                let _ = fs::remove_dir_all(&self.path);
            } else {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}

pub fn process(
    program: &Path,
    args: &[OsString],
    work: &Path,
    cancel: &AtomicBool,
) -> Result<String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    let stdout = work.join("stdout.log");
    let stderr = work.join("stderr.log");
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(File::create(&stdout).map_err(|e| e.to_string())?)
        .stderr(File::create(&stderr).map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| format!("Cannot start {}: {e}", program.display()))?;
    let start = Instant::now();
    let status = loop {
        if cancel.load(Ordering::Relaxed) || start.elapsed() > Duration::from_secs(1800) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(if cancel.load(Ordering::Relaxed) {
                "Cancelled"
            } else {
                "Conversion exceeded 30 minutes"
            }
            .into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(75)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(e.to_string());
            }
        }
    };
    if !status.success() {
        let err = fs::read_to_string(&stderr).unwrap_or_default();
        let tail = err
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "{} failed ({}). {}",
            program.file_name().unwrap_or_default().to_string_lossy(),
            status,
            tail
        ));
    }
    let mut text = String::new();
    File::open(stdout)
        .map_err(|e| e.to_string())?
        .take(2 * 1024 * 1024)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Ok(text)
}
fn strings(parts: &[&str]) -> Vec<OsString> {
    parts.iter().map(OsString::from).collect()
}

#[derive(Debug)]
pub struct Completed {
    pub output: Option<PathBuf>,
    pub clipboard: Option<String>,
    pub detail: String,
}
pub fn run(
    input: &Path,
    mode: Mode,
    op: &str,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<Completed> {
    let input = if input.is_absolute() {
        input.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(input)
    };
    if !input.exists() {
        return Err(format!("{} no longer exists", input.display()));
    }
    let kind = Kind::of(&input);
    if !menu(kind, mode).contains(&op) && op != "ZIP" && !(op == "SHA-256" && input.is_file()) {
        return Err(format!("{op} is not available for {}", kind.label()));
    }
    if mode == Mode::Actions {
        return Err("File actions are handled by the desktop.".into());
    }
    if let Some(reason) = tools.unavailable_for(&input, mode, op) {
        return Err(reason);
    }
    let parent = input.parent().ok_or("No output directory")?;
    let work = WorkDir::new(parent)?;
    if op == "SHA-256" {
        let text = process(
            tools.get("sha256sum")?,
            &[input.clone().into_os_string()],
            &work.path,
            cancel,
        )?;
        let hash = text
            .trim_start_matches('\\')
            .get(..64)
            .ok_or("Invalid SHA-256 output")?
            .to_string();
        if !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Invalid SHA-256 output".into());
        }
        return Ok(Completed {
            output: None,
            clipboard: Some(hash),
            detail: "SHA-256 copied".into(),
        });
    }
    if matches!(op, "ZIP" | "TAR" | "GZIP") {
        return archive(&input, op, &work.path, tools, cancel);
    }
    if kind == Kind::Pdf && mode == Mode::Convert {
        if matches!(op, "PNG" | "JPG") {
            return pdf_images(&input, &input, op, &work.path, tools, cancel);
        }
        if op == "PPTX" {
            return office(
                &input,
                &input,
                op,
                &work.path,
                tools,
                cancel,
                Some("impress_pdf_import"),
            );
        }
        let text = work.path.join(if op == "XLSX" {
            "document.csv"
        } else {
            "document.txt"
        });
        let args = vec![
            "-layout".into(),
            input.as_os_str().into(),
            text.as_os_str().into(),
        ];
        process(tools.get("pdftotext")?, &args, &work.path, cancel)?;
        if fs::read_to_string(&text)
            .map_err(|e| e.to_string())?
            .trim()
            .is_empty()
        {
            return Err(
                "This PDF has no extractable text. Export JPG/PNG or use OCR first.".into(),
            );
        }
        if matches!(op, "TXT" | "MD") {
            return saved(&input, op, &text);
        }
        if op == "XLSX" {
            let rows = fs::read_to_string(&text).map_err(|e| e.to_string())?;
            let csv = rows
                .lines()
                .map(|line| format!("\"{}\"", line.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(&text, csv).map_err(|e| e.to_string())?;
        }
        return office(&input, &text, op, &work.path, tools, cancel, None);
    }
    if matches!(kind, Kind::Document | Kind::Spreadsheet | Kind::Slides) {
        let source = prepare_document(&input, &work.path, tools, cancel)?;
        if kind == Kind::Slides && matches!(op, "PNG" | "JPG") {
            let pdf = office_stage(&source, "PDF", &work.path, tools, cancel, None)?;
            return pdf_images(&input, &pdf, op, &work.path, tools, cancel);
        }
        return office(&input, &source, op, &work.path, tools, cancel, None);
    }
    if kind == Kind::Subtitle {
        return subtitle(&input, op, &work.path, tools, cancel);
    }
    let original_ext = input
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let (suffix, ext) = if mode == Mode::Convert {
        ("", op.to_ascii_lowercase())
    } else {
        match op {
            "Resize 1280" => (" (1280)", original_ext.clone()),
            "Compress" if kind == Kind::Image => (" (compressed)", "jpg".into()),
            "Compress" if kind == Kind::Audio => (" (compressed)", "mp3".into()),
            "Compress" => (" (compressed)", "mp4".into()),
            "Strip metadata" => (" (clean)", original_ext.clone()),
            "Rotate 90" => (" (rotated)", original_ext.clone()),
            "Grayscale" => (" (gray)", original_ext.clone()),
            "Mute" => (" (muted)", original_ext.clone()),
            "First 10s" => (" (10s)", original_ext.clone()),
            "Grab frame" => (" (frame)", "png".into()),
            "Normalize" => (" (normalized)", original_ext.clone()),
            "First 30s" => (" (30s)", original_ext.clone()),
            "Mono" => (" (mono)", original_ext.clone()),
            _ => return Err("Unknown operation".into()),
        }
    };
    let output = Output::reserve(&input, suffix, &ext, false)?;
    let staged = work.path.join(format!("result.{ext}"));
    if kind == Kind::Pdf {
        let args = vec![
            OsString::from("--remove-info"),
            OsString::from("--remove-metadata"),
            input.as_os_str().into(),
            staged.clone().into_os_string(),
        ];
        process(tools.get("qpdf")?, &args, &work.path, cancel)?;
    } else if kind == Kind::Image {
        let mut args = vec![input.as_os_str().into()];
        if mode == Mode::Convert {
            args.extend(strings(&["-auto-orient"]));
            if op == "WEBP" {
                args.extend(strings(&["-quality", "85"]));
            }
        } else {
            args.extend(strings(match op {
                "Resize 1280" => &["-resize", "1280x1280>"],
                "Compress" => &[
                    "-background",
                    "white",
                    "-alpha",
                    "remove",
                    "-alpha",
                    "off",
                    "-quality",
                    "60",
                ],
                "Strip metadata" => &["-strip"],
                "Rotate 90" => &["-rotate", "90"],
                "Grayscale" => &["-colorspace", "Gray"],
                _ => return Err("Unknown image tool".into()),
            }));
        }
        if mode == Mode::Convert && op == "JPG" {
            args.extend(strings(&[
                "-background",
                "white",
                "-alpha",
                "remove",
                "-alpha",
                "off",
            ]));
        }
        args.push(staged.clone().into_os_string());
        process(tools.get("magick")?, &args, &work.path, cancel)?;
    } else {
        let mut args = strings(&["-nostdin", "-y", "-v", "error", "-i"]);
        args.push(input.as_os_str().into());
        let preset = if mode == Mode::Convert {
            match op {
                "MP4" => vec![
                    "-c:v",
                    "libx264",
                    "-crf",
                    "23",
                    "-preset",
                    "veryfast",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                    "-movflags",
                    "+faststart",
                ],
                "MOV" => vec![
                    "-c:v", "libx264", "-crf", "23", "-preset", "veryfast", "-c:a", "aac",
                ],
                "GIF" => vec!["-vf", "fps=12,scale=480:-1:flags=lanczos", "-an"],
                "WEBM" => vec![
                    "-c:v",
                    "libvpx-vp9",
                    "-b:v",
                    "0",
                    "-crf",
                    "34",
                    "-row-mt",
                    "1",
                    "-c:a",
                    "libopus",
                ],
                "MP3" => vec!["-vn", "-c:a", "libmp3lame", "-q:a", "2"],
                "M4A" => vec!["-vn", "-c:a", "aac", "-b:a", "192k"],
                "WAV" => vec!["-vn", "-c:a", "pcm_s16le"],
                "FLAC" => vec!["-vn", "-c:a", "flac"],
                "OGG" => vec!["-vn", "-c:a", "libvorbis", "-q:a", "5"],
                "OPUS" => vec!["-vn", "-c:a", "libopus", "-b:a", "128k"],
                "AIFF" => vec!["-vn", "-c:a", "pcm_s16be"],
                "WMA" => vec!["-vn", "-c:a", "wmav2", "-b:a", "192k"],
                "MKV" => vec![
                    "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p", "-c:a", "aac",
                ],
                "AVI" => vec!["-c:v", "mpeg4", "-q:v", "5", "-c:a", "libmp3lame"],
                "WMV" => vec!["-c:v", "wmv2", "-c:a", "wmav2"],
                _ => return Err("Unknown media format".into()),
            }
        } else {
            match op {
                "Compress" if kind == Kind::Video => vec![
                    "-c:v", "libx264", "-crf", "28", "-preset", "veryfast", "-pix_fmt", "yuv420p",
                    "-c:a", "aac", "-b:a", "128k",
                ],
                "Compress" => vec!["-vn", "-c:a", "libmp3lame", "-b:a", "96k"],
                "Strip metadata" => vec!["-map", "0", "-map_metadata", "-1", "-c", "copy"],
                "Mute" => vec!["-an", "-c:v", "copy"],
                "First 10s" => vec!["-t", "10", "-c", "copy"],
                "Grab frame" => vec!["-frames:v", "1"],
                "Normalize" => vec!["-af", "loudnorm"],
                "First 30s" => vec!["-t", "30", "-c", "copy"],
                "Mono" => vec!["-ac", "1"],
                _ => return Err("Unknown media tool".into()),
            }
        };
        args.extend(preset.iter().map(OsString::from));
        args.push(staged.clone().into_os_string());
        process(tools.get("ffmpeg")?, &args, &work.path, cancel)?;
    }
    let path = output.commit(&staged)?;
    Ok(Completed {
        output: Some(path),
        clipboard: None,
        detail: format!("{op} complete"),
    })
}

fn saved(input: &Path, op: &str, staged: &Path) -> Result<Completed> {
    let extension = if op == "GZIP" {
        "tar.gz".into()
    } else {
        op.to_ascii_lowercase()
    };
    let output = Output::reserve(input, "", &extension, false)?;
    Ok(Completed {
        output: Some(output.commit(staged)?),
        clipboard: None,
        detail: format!("{op} complete"),
    })
}

fn pdf_images(
    input: &Path,
    pdf: &Path,
    op: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<Completed> {
    let output = Output::reserve(input, " pages", "", true)?;
    let pages = work.join("pages");
    fs::create_dir(&pages).map_err(|e| e.to_string())?;
    let mut args = strings(&[if op == "PNG" { "-png" } else { "-jpeg" }, "-r", "300"]);
    args.push(pdf.as_os_str().into());
    args.push(pages.join("page").into_os_string());
    process(tools.get("pdftoppm")?, &args, work, cancel)?;
    Ok(Completed {
        output: Some(output.commit(&pages)?),
        clipboard: None,
        detail: "All pages exported at 300 DPI".into(),
    })
}

fn helper(
    input: &Path,
    output: &Path,
    action: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<()> {
    let args = vec![
        "-c".into(),
        include_str!("formats.py").into(),
        action.into(),
        input.as_os_str().into(),
        output.as_os_str().into(),
    ];
    process(tools.get("python3")?, &args, work, cancel).map(|_| ())
}

fn prepare_document(
    input: &Path,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    let ext = input
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    if ext == "rtfd" {
        if !input.is_dir() {
            return Err("RTFD input must be a document bundle containing TXT.rtf.".into());
        }
        return fs::read_dir(input)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("rtf")))
            .ok_or_else(|| "No RTF document found inside this RTFD bundle.".into());
    }
    if ext == "pages" && input.is_dir() {
        let source = work.join("document.pages");
        let args = vec![
            "-a".into(),
            "-cf".into(),
            source.as_os_str().into(),
            "-C".into(),
            input.as_os_str().into(),
            ".".into(),
        ];
        process(tools.get("bsdtar")?, &args, work, cancel)?;
        return Ok(source);
    }
    if matches!(ext.as_str(), "epub" | "textclipping") {
        let source = work.join(if ext == "epub" {
            "document.html"
        } else {
            "document.txt"
        });
        helper(input, &source, &ext, work, tools, cancel)?;
        return Ok(source);
    }
    Ok(input.to_path_buf())
}

fn office_stage(
    source: &Path,
    op: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
    import: Option<&str>,
) -> Result<PathBuf> {
    if op == "EPUB" {
        let text = office_stage(source, "TXT", work, tools, cancel, import)?;
        let epub = work.join("result.epub");
        helper(&text, &epub, "make_epub", work, tools, cancel)?;
        return Ok(epub);
    }
    let extension = match op {
        "Copy text" | "Word count" => "txt".into(),
        _ => op.to_ascii_lowercase(),
    };
    let kind = if import == Some("impress_pdf_import") {
        Kind::Slides
    } else {
        Kind::of(source)
    };
    let filter = match extension.as_str() {
        "txt" => "txt:Text (encoded):UTF8",
        "md" => "md:Markdown",
        "html" => "html:HTML (StarWriter)",
        "docx" => "docx:Office Open XML Text",
        "epub" => "epub:EPUB",
        "pptx" => "pptx:Impress MS PowerPoint 2007 XML",
        "xlsx" => "xlsx:Calc Office Open XML",
        "ods" => "ods:calc8",
        "csv" => "csv:Text - txt - csv (StarCalc):44,34,76,1,,0,false,true,false,false,false,0",
        "tsv" => "tsv:Text - txt - csv (StarCalc):9,34,76,1,,0,false,true,false,false,false,0",
        "pdf" if kind == Kind::Slides => "pdf:impress_pdf_Export",
        "pdf" if kind == Kind::Spreadsheet => "pdf:calc_pdf_Export",
        "pdf" => "pdf:writer_pdf_Export",
        _ => return Err("Unsupported office export format".into()),
    };
    let out_dir = work.join("converted");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let profile = gtk::gio::File::for_path(work.join("office-profile")).uri();
    let mut args = vec![
        format!("-env:UserInstallation={profile}").into(),
        "--headless".into(),
    ];
    if let Some(import) = import {
        args.push(format!("--infilter={import}").into());
    } else if source
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("tsv") || e.eq_ignore_ascii_case("csv"))
    {
        let delimiter = if source
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("tsv"))
        {
            9
        } else {
            44
        };
        // Import text cells literally; do not turn untrusted CSV text into formulas.
        args.push(
            format!("--infilter=Text - txt - csv (StarCalc):{delimiter},34,76,1,,0,false,false")
                .into(),
        );
    } else if source
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("markdown") || e.eq_ignore_ascii_case("md"))
    {
        args.push("--infilter=Markdown".into());
    }
    args.extend(strings(&["--convert-to", filter, "--outdir"]));
    args.push(out_dir.as_os_str().into());
    args.push(source.as_os_str().into());
    process(tools.get("libreoffice")?, &args, work, cancel)?;
    let converted = out_dir
        .join(source.file_stem().ok_or("No document name")?)
        .with_extension(extension);
    if !converted.is_file() {
        return Err("LibreOffice could not read this file or write the chosen format. Pages/RTFD support depends on the source version; EPUB input is imported as reflowed text.".into());
    }
    Ok(converted)
}

fn office(
    input: &Path,
    source: &Path,
    op: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
    import: Option<&str>,
) -> Result<Completed> {
    let converted = office_stage(source, op, work, tools, cancel, import)?;
    if matches!(op, "Copy text" | "Word count") {
        let text = fs::read_to_string(converted).map_err(|e| e.to_string())?;
        let value = if op == "Word count" {
            text.split_whitespace().count().to_string()
        } else {
            text
        };
        return Ok(Completed {
            output: None,
            clipboard: Some(value),
            detail: format!("{op} copied"),
        });
    }
    saved(input, op, &converted)
}

fn archive(
    input: &Path,
    op: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<Completed> {
    let staged = work.join(match op {
        "ZIP" => "result.zip",
        "TAR" => "result.tar",
        _ => "result.tar.gz",
    });
    let mut args = strings(&["-a", "-cf"]);
    args.push(staged.as_os_str().into());
    if Kind::of(input) == Kind::Archive {
        let ext = input
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        let raw = work.join("gzip-content");
        fs::create_dir(&raw).map_err(|e| e.to_string())?;
        let name = input.file_stem().ok_or("No gzip name")?;
        let decompressed = raw.join(name);
        if matches!(ext.as_str(), "gz" | "gzip" | "tgz") {
            helper(input, &decompressed, "gzip", work, tools, cancel)?;
        }
        if decompressed.is_file() {
            args.push("-C".into());
            args.push(raw.as_os_str().into());
            args.push("--".into());
            args.push(name.into());
        } else {
            let mut archive = OsString::from("@");
            archive.push(input);
            args.push(archive);
        }
    } else {
        args.extend(strings(&["-C"]));
        args.push(input.parent().ok_or("No parent folder")?.as_os_str().into());
        args.push("--".into());
        args.push(
            input
                .file_name()
                .ok_or("Cannot archive the filesystem root")?
                .into(),
        );
    }
    process(tools.get("bsdtar")?, &args, work, cancel)?;
    saved(input, op, &staged)
}

fn subtitle(
    input: &Path,
    op: &str,
    work: &Path,
    tools: &Tools,
    cancel: &AtomicBool,
) -> Result<Completed> {
    let staged = work.join(format!("result.{}", op.to_ascii_lowercase()));
    if op == "TXT" {
        let text = fs::read_to_string(input).map_err(|e| e.to_string())?;
        let mut lines = Vec::new();
        let mut cue = false;
        let mut block = false;
        for line in text.trim_start_matches('\u{feff}').lines() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() {
                cue = false;
                block = false;
                continue;
            }
            if !cue
                && (line.starts_with("WEBVTT")
                    || line == "NOTE"
                    || line.starts_with("NOTE ")
                    || line == "STYLE"
                    || line == "REGION")
            {
                block = true;
            }
            if block {
                continue;
            }
            if line.contains("-->") {
                cue = true;
                continue;
            }
            if cue {
                lines.push(line);
            }
        }
        if lines.is_empty() {
            return Err("No subtitle cues found.".into());
        }
        fs::write(&staged, format!("{}\n", lines.join("\n"))).map_err(|e| e.to_string())?;
    } else {
        let args = vec![
            "-nostdin".into(),
            "-y".into(),
            "-v".into(),
            "error".into(),
            "-i".into(),
            input.as_os_str().into(),
            staged.as_os_str().into(),
        ];
        process(tools.get("ffmpeg")?, &args, work, cancel)?;
    }
    saved(input, op, &staged)
}

pub fn format_note(kind: Kind) -> &'static str {
    match kind {
        Kind::Image => "JPG · PNG · WebP · HEIC · TIFF · AVIF · BMP · PDF export · SVG input only",
        Kind::Audio => "MP3 · M4A · WAV · FLAC · OGG · Opus · AIFF · WMA",
        Kind::Video => "MP4 · MOV · MKV · WebM · AVI · WMV · GIF · MP3 export",
        Kind::Pdf => {
            "All pages · JPG/PNG at 300 DPI · DOCX/EPUB/TXT/MD/XLSX extract text · PPTX imports page objects · Pages export unavailable"
        }
        Kind::Document => {
            "DOCX · Markdown · TXT · EPUB · HTML export · Pages, RTF, RTFD and text clippings input · Pages export unavailable"
        }
        Kind::Slides => {
            "PPT/PPTX → PDF, JPG, PNG · All slides · Images at 300 DPI · PPT input only"
        }
        Kind::Spreadsheet => {
            "XLSX · ODS · CSV · TSV · PDF export · XLS input only · CSV/TSV export the first sheet"
        }
        Kind::Subtitle => "SRT · VTT · TXT export (cue text without timestamps)",
        Kind::Archive | Kind::Folder => "ZIP · TAR · GZIP (tar.gz) · RAR input only",
        Kind::Other => "ZIP · SHA-256",
    }
}

pub fn log_job(message: &str) {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"));
    let dir = state.join("filewheel");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("jobs.log");
    if fs::metadata(&path)
        .map(|m| m.len() > 1_000_000)
        .unwrap_or(false)
    {
        let _ = fs::rename(&path, dir.join("jobs.previous.log"));
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", message.replace('\n', " | "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundle_and_media_inputs_use_the_correct_family() {
        let work = WorkDir::new(&std::env::temp_dir()).unwrap();
        for (name, kind) in [
            ("book.pages", Kind::Document),
            ("rich.rtfd", Kind::Document),
            ("folder", Kind::Folder),
        ] {
            let path = work.path.join(name);
            fs::create_dir(&path).unwrap();
            assert_eq!(Kind::of(&path), kind);
        }
        for (name, kind) in [
            ("clip.GIF", Kind::Video),
            ("picture.SVG", Kind::Image),
            ("deck.PPT", Kind::Slides),
            ("data.XLS", Kind::Spreadsheet),
            ("captions.VTT", Kind::Subtitle),
            ("bundle.RAR", Kind::Archive),
        ] {
            assert_eq!(Kind::of(Path::new(name)), kind);
        }
    }
    #[test]
    fn pages_export_and_missing_import_engines_are_disabled() {
        let tools = Tools(BTreeMap::new());
        assert!(
            tools
                .unavailable(Kind::Document, Mode::Convert, "PAGES")
                .unwrap()
                .contains("unavailable on Linux")
        );
        assert_eq!(
            tools.unavailable(Kind::Pdf, Mode::Convert, "PNG"),
            Some("Requires pdftoppm".into())
        );
        assert!(!menu(Kind::Image, Mode::Convert).contains(&"SVG"));
        assert!(!menu(Kind::Slides, Mode::Convert).contains(&"PPT"));
        assert!(!menu(Kind::Spreadsheet, Mode::Convert).contains(&"XLS"));
        assert!(!menu(Kind::Archive, Mode::Convert).contains(&"RAR"));
    }
    #[test]
    fn subtitle_text_keeps_dialogue_that_looks_like_a_header() {
        let work = WorkDir::new(&std::env::temp_dir()).unwrap();
        let input = work.path.join("caption.vtt");
        fs::write(&input, "WEBVTT\n\nNOTE metadata\nignore this\n\ncue-id\n00:00.000 --> 00:01.000\nNOTE this is dialogue\n123\n").unwrap();
        let result = run(
            &input,
            Mode::Convert,
            "TXT",
            &Tools(BTreeMap::new()),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(result.output.unwrap()).unwrap(),
            "NOTE this is dialogue\n123\n"
        );
    }
    #[test]
    fn reserved_outputs_never_replace_input_or_collide() {
        let work = WorkDir::new(&std::env::temp_dir()).unwrap();
        let source = work.path.join("photo.png");
        fs::write(&source, b"original").unwrap();
        let one = Output::reserve(&source, "", "png", false).unwrap();
        let two = Output::reserve(&source, "", "png", false).unwrap();
        assert_eq!(one.path.file_name().unwrap(), "photo 2.png");
        assert_eq!(two.path.file_name().unwrap(), "photo 3.png");
        let staged = work.path.join("staged");
        fs::write(&staged, b"converted").unwrap();
        let output = one.commit(&staged).unwrap();
        drop(two);
        assert_eq!(fs::read(source).unwrap(), b"original");
        assert_eq!(fs::read(output).unwrap(), b"converted");
        assert!(!work.path.join("photo 3.png").exists());
    }
    #[test]
    fn cancelled_process_cannot_create_a_result() {
        let work = WorkDir::new(&std::env::temp_dir()).unwrap();
        let cancel = AtomicBool::new(true);
        assert!(
            process(Path::new("/usr/bin/false"), &[], &work.path, &cancel)
                .unwrap_err()
                .contains("Cancelled")
        );
    }
    #[test]
    fn active_process_is_killed_on_cancel() {
        let work = WorkDir::new(&std::env::temp_dir()).unwrap();
        let flag = std::sync::Arc::new(AtomicBool::new(false));
        let other = flag.clone();
        let trigger = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            other.store(true, Ordering::Relaxed);
        });
        let start = Instant::now();
        let result = process(
            Path::new("/usr/bin/sleep"),
            &strings(&["5"]),
            &work.path,
            &flag,
        );
        trigger.join().unwrap();
        assert_eq!(result.unwrap_err(), "Cancelled");
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
