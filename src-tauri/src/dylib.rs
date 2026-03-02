use libloading::Library;
use std::{path::Path, sync::OnceLock};

static PRELOADED_LIBS: OnceLock<Vec<Library>> = OnceLock::new();

static DYLIB_LIST: [&'static str; 11] = [
    "cublas64_13.dll",
    "cublasLt64_13.dll",
    "cudnn64_9.dll",
    "cudnn_graph64_9.dll",
    "cudnn_engines_precompiled64_9.dll",
    "cudnn_engines_runtime_compiled64_9.dll",
    "cudnn_ops64_9.dll",
    "cufft64_12.dll",
    "nvrtc-builtins64_130.dll",
    "nvrtc64_130_0.dll",
    "onnxruntime.dll",
];

pub fn preload_libs(libs_dir: &Path) {
    let mut libs: Vec<Library> = vec![];
    for dylib in DYLIB_LIST {
        let path = libs_dir.join(dylib);
        // print!("Loading {}... ", path.display());
        unsafe {
            match Library::new(path) {
                Ok(lib) => {
                    // println!("OK");
                    libs.push(lib);
                }
                Err(e) => {
                    // println!("FAILED: {e}");
                }
            }
        }
    }

    PRELOADED_LIBS.get_or_init(|| libs);
}
