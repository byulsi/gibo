use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const WORKSPACE_DIRECTORIES: &[&str] = &["wiki", "lens", "method", "evidence", "decision"];
pub const DEFAULT_GURU_MODE: &str = "auto";
pub const GURU_MODES: &[&str] = &["auto", "off"];

#[derive(Debug, Serialize)]
pub struct WorkspaceInit {
    pub root: String,
    pub created: Vec<String>,
    pub directories: Vec<String>,
    pub pack_directory: String,
    pub guru_mode: String,
}

#[derive(Debug, Serialize)]
pub struct GuruModeStatus {
    pub root: String,
    pub mode: String,
    pub mode_file: String,
    pub defaulted: bool,
}

pub fn initialize_workspace(root: &Path) -> Result<WorkspaceInit, String> {
    let root = absolute_path(root)?;
    let data_root = root.join("guru-maker");
    let mut created = Vec::new();
    let mut directories = Vec::new();

    for name in WORKSPACE_DIRECTORIES {
        let path = data_root.join(name);
        if !path.exists() {
            fs::create_dir_all(&path).map_err(io_error)?;
            created.push(relative_display(&root, &path));
        }
        directories.push(path.display().to_string());
    }

    let packs = root.join(".guru-maker").join("packs");
    if !packs.exists() {
        fs::create_dir_all(&packs).map_err(io_error)?;
        created.push(relative_display(&root, &packs));
    }
    let ignore = root.join(".guru-maker").join(".gitignore");
    let existing = if ignore.exists() {
        fs::read_to_string(&ignore).map_err(io_error)?
    } else {
        String::new()
    };
    if !existing.lines().any(|line| line == "packs/") {
        let separator = if existing.is_empty() || existing.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        fs::write(&ignore, format!("{existing}{separator}packs/\n")).map_err(io_error)?;
    }
    let mode = guru_mode_file(&root);
    if !mode.exists() {
        fs::write(&mode, format!("{DEFAULT_GURU_MODE}\n")).map_err(io_error)?;
        created.push(relative_display(&root, &mode));
    }
    let guru_mode = read_guru_mode(&root)?;

    Ok(WorkspaceInit {
        root: root.display().to_string(),
        created,
        directories,
        pack_directory: packs.display().to_string(),
        guru_mode: guru_mode.mode,
    })
}

pub fn pack_directory(root: &Path) -> PathBuf {
    root.join(".guru-maker").join("packs")
}

pub fn guru_mode_file(root: &Path) -> PathBuf {
    root.join(".guru-maker").join("mode")
}

pub fn read_guru_mode(root: &Path) -> Result<GuruModeStatus, String> {
    require_workspace(root)?;
    let root = absolute_path(root)?;
    let mode_file = guru_mode_file(&root);
    let (mode, defaulted) = if mode_file.exists() {
        let value = fs::read_to_string(&mode_file).map_err(io_error)?;
        (parse_guru_mode(&value)?.to_owned(), false)
    } else {
        (DEFAULT_GURU_MODE.to_owned(), true)
    };
    Ok(GuruModeStatus {
        root: root.display().to_string(),
        mode,
        mode_file: mode_file.display().to_string(),
        defaulted,
    })
}

pub fn set_guru_mode(root: &Path, mode: &str) -> Result<GuruModeStatus, String> {
    require_workspace(root)?;
    let root = absolute_path(root)?;
    let mode = parse_guru_mode(mode)?;
    let mode_file = guru_mode_file(&root);
    fs::write(&mode_file, format!("{mode}\n")).map_err(io_error)?;
    read_guru_mode(&root)
}

pub fn is_workspace(root: &Path) -> bool {
    root.join("guru-maker").is_dir() && root.join(".guru-maker/.gitignore").is_file()
}

pub fn find_workspace(start: &Path) -> Option<PathBuf> {
    let start = absolute_path(start).ok()?;
    start
        .ancestors()
        .find(|candidate| is_workspace(candidate))
        .map(Path::to_path_buf)
}

pub fn require_workspace(root: &Path) -> Result<(), String> {
    if is_workspace(root) {
        Ok(())
    } else {
        Err("workspace is not initialized; run `gurumaker init <workspace>` first".into())
    }
}

pub fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(io_error)
            .map(|current| current.join(path))
    }
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn parse_guru_mode(value: &str) -> Result<&str, String> {
    let value = value.trim();
    if GURU_MODES.contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "invalid Guru Mode: {value:?}; expected {}",
            GURU_MODES.join(" or ")
        ))
    }
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}
