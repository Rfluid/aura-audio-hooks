//! Native folder/file picker dialogs, used by panel button actions so
//! paths can be edited entirely from the aura UI. Shells out to zenity
//! or kdialog — no toolkit dependency. The aura host gives `action`
//! invocations a generous timeout precisely so these can block.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Result};

enum Tool {
    Zenity,
    Kdialog,
}

fn tool() -> Result<Tool> {
    let on_path = |bin: &str| {
        std::env::var_os("PATH")
            .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file()))
    };
    if on_path("zenity") {
        Ok(Tool::Zenity)
    } else if on_path("kdialog") {
        Ok(Tool::Kdialog)
    } else {
        bail!("no dialog tool found — install zenity or kdialog to pick paths from the UI")
    }
}

fn run_dialog(mut cmd: Command) -> Result<Option<PathBuf>> {
    let output = cmd.output()?;
    if !output.status.success() {
        // Cancelled (both tools exit 1 on cancel) — not an error.
        return Ok(None);
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(path)))
}

/// Pick a directory. `Ok(None)` means the user cancelled.
pub fn pick_dir(title: &str) -> Result<Option<PathBuf>> {
    match tool()? {
        Tool::Zenity => {
            let mut c = Command::new("zenity");
            c.args(["--file-selection", "--directory", "--title", title]);
            run_dialog(c)
        }
        Tool::Kdialog => {
            let mut c = Command::new("kdialog");
            c.args(["--getexistingdirectory", ".", "--title", title]);
            run_dialog(c)
        }
    }
}

/// Pick a single file. `Ok(None)` means the user cancelled.
pub fn pick_file(title: &str) -> Result<Option<PathBuf>> {
    match tool()? {
        Tool::Zenity => {
            let mut c = Command::new("zenity");
            c.args(["--file-selection", "--title", title]);
            run_dialog(c)
        }
        Tool::Kdialog => {
            let mut c = Command::new("kdialog");
            c.args(["--getopenfilename", ".", "--title", title]);
            run_dialog(c)
        }
    }
}
