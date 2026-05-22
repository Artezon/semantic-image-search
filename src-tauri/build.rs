use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let vcpkg = Path::new("C:\\vcpkg\\installed\\x64-windows");
    copy_dlls(vcpkg);

    tauri_build::build()
}

fn copy_dlls(from: &Path) {
    let dlls = [
        "avcodec-62.dll",
        "avformat-62.dll",
        "avutil-60.dll",
        "swscale-9.dll",
        // "swresample-6.dll",
        // "heif.dll",
        // "libde265.dll",
        // "libx265.dll",
    ];

    let src_dir = from.join("bin");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap().to_path_buf();

    for dll in &dlls {
        let src = src_dir.join(dll);
        if src.exists() {
            let dst = target_dir.join(dll);
            if let Err(e) = fs::copy(&src, &dst) {
                println!("cargo:warning=Could not copy {dll}: {e}");
            }
        } else {
            println!("cargo:warning={dll} not found at {}", src_dir.display());
        }
    }
}
