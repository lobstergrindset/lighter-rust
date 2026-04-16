use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use libloading::Library;

use crate::error::{Result, SdkError};

static SIGNER_LIBRARY: OnceLock<SignerLibrary> = OnceLock::new();
static INIT_LOCK: Mutex<()> = Mutex::new(());

pub struct SignerLibrary {
    #[allow(dead_code)]
    library: Library,
}

unsafe impl Send for SignerLibrary {}
unsafe impl Sync for SignerLibrary {}

impl SignerLibrary {
    pub fn lib(&self) -> &Library {
        &self.library
    }
}

fn lib_filename() -> &'static str {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "lighter-signer-darwin-arm64.dylib"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "lighter-signer-linux-amd64.so"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "lighter-signer-linux-arm64.so"
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "lighter-signer-windows-amd64.dll"
    } else {
        panic!(
            "Unsupported platform/architecture: {}/{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }
}

fn push_path(paths: &mut Vec<PathBuf>, configured: &Path, filename: &str) {
    if configured.is_dir() {
        paths.push(configured.join(filename));
    } else {
        paths.push(configured.to_path_buf());
    }
}

fn search_paths(configured_path: Option<&str>) -> Vec<PathBuf> {
    let filename = lib_filename();
    let mut paths = Vec::new();

    // 1. Configured path
    if let Some(p) = configured_path {
        push_path(&mut paths, Path::new(p), filename);
    }

    // 2. LIGHTER_SIGNER_LIB_PATH env var
    if let Ok(env_path) = std::env::var("LIGHTER_SIGNER_LIB_PATH") {
        push_path(&mut paths, Path::new(&env_path), filename);
    }

    // 3. Next to the current executable
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        paths.push(dir.join(filename));
        paths.push(dir.join("signers").join(filename));
    }

    // 4. Current directory
    paths.push(PathBuf::from(filename));
    paths.push(PathBuf::from("signers").join(filename));

    paths
}

pub fn load_signer(configured_path: Option<&str>) -> Result<&'static SignerLibrary> {
    if let Some(lib) = SIGNER_LIBRARY.get() {
        return Ok(lib);
    }

    let _guard = INIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(lib) = SIGNER_LIBRARY.get() {
        return Ok(lib);
    }

    let paths = search_paths(configured_path);
    let mut errors = Vec::new();

    for path in &paths {
        match unsafe { Library::new(path) } {
            Ok(library) => {
                tracing::info!("Loaded signer library from {}", path.display());
                let _ = SIGNER_LIBRARY.set(SignerLibrary { library });
                return SIGNER_LIBRARY.get().ok_or(SdkError::SignerNotLoaded);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
                tracing::debug!("Failed to load signer from {}: {}", path.display(), e);
            }
        }
    }

    Err(SdkError::SignerLoadFailed(errors.join(" | ")))
}

pub fn get_signer() -> Result<&'static SignerLibrary> {
    SIGNER_LIBRARY.get().ok_or(SdkError::SignerNotLoaded)
}
