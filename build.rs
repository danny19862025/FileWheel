fn main() {
    println!("cargo:rerun-if-changed=data/filewheel.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("data/filewheel.ico")
        .set("ProductName", "FileWheel")
        .set("FileDescription", "FileWheel file converter");
    if let Err(error) = resource.compile() {
        // A missing resource compiler only costs the .exe icon; the tray
        // icon falls back to the embedded PNG.
        println!("cargo:warning=Windows resources not embedded: {error}");
    }
}
