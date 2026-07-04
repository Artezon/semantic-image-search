use crate::errors::AppError;
use libloading::Library;
use std::{path::Path, sync::OnceLock};

struct DylibEntry {
    filename: &'static str,
    required: bool,
}

static DYLIB_LIST: [DylibEntry; 5] = [
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
