use crate::errors::AppError;
use libloading::Library;
use std::{path::Path, sync::OnceLock};

struct DylibEntry {
    filename: &'static str,
    required: bool,
}

static DYLIB_LIST: [DylibEntry; 15] = [
    DylibEntry {
        filename: "vcruntime140.dll",
        required: false,
    },
    DylibEntry {
        filename: "vcruntime140_1.dll",
        required: false,
    },
    DylibEntry {
        filename: "msvcp140.dll",
        required: false,
    },
    DylibEntry {
        filename: "msvcp140_1.dll",
        required: false,
    },
    DylibEntry {
        filename: "cublas64_13.dll",
        required: false,
    },
    DylibEntry {
        filename: "cublasLt64_13.dll",
        required: false,
    },
    DylibEntry {
        filename: "cudnn64_9.dll",
        required: false,
    },
    DylibEntry {
        filename: "cudnn_graph64_9.dll",
        required: false,
    },
    DylibEntry {
        filename: "cudnn_engines_precompiled64_9.dll",
        required: false,
    },
    DylibEntry {
        filename: "cudnn_engines_runtime_compiled64_9.dll",
        required: false,
    },
    DylibEntry {
        filename: "cudnn_ops64_9.dll",
        required: false,
    },
    DylibEntry {
        filename: "cufft64_12.dll",
        required: false,
    },
    DylibEntry {
        filename: "nvrtc-builtins64_130.dll",
        required: false,
    },
    DylibEntry {
        filename: "nvrtc64_130_0.dll",
        required: false,
    },
    DylibEntry {
        filename: "onnxruntime.dll",
        required: true,
    },
];

static PRELOADED_LIBS: OnceLock<Vec<Library>> = OnceLock::new();

pub fn preload_libs(libs_dir: &Path) -> Result<(), AppError> {
    let mut libs: Vec<Library> = vec![];
    for entry in &DYLIB_LIST {
        let path = libs_dir.join(entry.filename);
        if entry.required && !path.exists() {
            return Err(AppError::LibraryLoadFailed {
                detail: "not_found".to_string(),
                name: path.display().to_string(),
            });
        }
        unsafe {
            match Library::new(&path) {
                Ok(lib) => {
                    libs.push(lib);
                }
                Err(e) => {
                    if entry.required {
                        return Err(AppError::LibraryLoadFailed {
                            detail: e.to_string(),
                            name: path.display().to_string(),
                        });
                    }
                }
            }
        }
    }

    PRELOADED_LIBS.get_or_init(|| libs);
    Ok(())
}
