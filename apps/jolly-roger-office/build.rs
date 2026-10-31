//! Windows only: embed the suite icon and version info into `jolly-roger-office.exe`, so it
//! shows in Explorer, the taskbar and Alt-Tab. A missing resource compiler is only a warning.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/app-icon/jolly-roger-office.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/jolly-roger-office.ico")
        .set("ProductName", "Jolly Roger Office")
        .set("FileDescription", "Jolly Roger Office")
        .set("CompanyName", "Hillyard Tech")
        .set("LegalCopyright", "Copyright (c) Hillyard Tech and the Craft Launcher authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "jolly-roger-office.exe")
        .set("InternalName", "jolly-roger-office");
    if let Err(e) = res.compile() {
        println!("cargo:warning=jolly-roger-office.exe built without icon/version resources: {e}");
    }
}
