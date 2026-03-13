#[cfg(target_os = "windows")]
#[allow(dead_code)]
#[path = "src/app_icon.rs"]
mod app_icon;

#[cfg(target_os = "windows")]
fn main() {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::PathBuf;

    println!("cargo:rerun-if-changed=src/app_icon.rs");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
    let icon_path = out_dir.join("rust-clock.ico");

    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for size in [16, 24, 32, 48, 64, 128, 256] {
        let rgba = app_icon::clock_face_icon_rgba(size);
        let image = IconImage::from_rgba_data(size, size, rgba);
        let entry = IconDirEntry::encode(&image).expect("failed to encode icon image");
        icon_dir.add_entry(entry);
    }

    let icon_file = File::create(&icon_path).expect("failed to create generated icon file");
    let mut writer = BufWriter::new(icon_file);
    icon_dir
        .write(&mut writer)
        .expect("failed to write generated icon file");

    let mut resource = winres::WindowsResource::new();
    resource.set_icon(icon_path.to_string_lossy().as_ref());
    resource
        .compile()
        .expect("failed to compile Windows resources");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
