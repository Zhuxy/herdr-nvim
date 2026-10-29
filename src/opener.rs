//! Hand a file to the desktop's default application.
//!
//! The file picker's `⇧⏎` skips the nvim sidebar and asks the OS to open the
//! chosen file the way a double-click in the file manager would: `open` on
//! macOS, `xdg-open` on Linux/BSD, `cmd /C start` on Windows. Only the argv
//! construction is unit tested; the spawn is exercised live, since it launches
//! a GUI app.

use std::process::{Command, Stdio};

use anyhow::{Context, Result};

/// The launcher program and the args it needs before the path itself.
///
/// Windows' `start` is a `cmd` builtin, and its first quoted argument is taken
/// as the window title -- hence the empty `""` so the path is the target, not
/// a title.
#[cfg(target_os = "macos")]
fn launcher() -> (&'static str, &'static [&'static str]) {
    ("open", &[])
}

#[cfg(target_os = "windows")]
fn launcher() -> (&'static str, &'static [&'static str]) {
    ("cmd", &["/C", "start", ""])
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn launcher() -> (&'static str, &'static [&'static str]) {
    ("xdg-open", &[])
}

/// The program and arguments that open `path` with its default application,
/// as `(program, args)` with the path last. Pure, unit tested.
pub fn opener_argv(path: &str) -> (&'static str, Vec<String>) {
    let (program, leading) = launcher();
    let mut args: Vec<String> = leading.iter().map(|arg| (*arg).to_owned()).collect();
    args.push(path.to_owned());
    (program, args)
}

/// Launch the default application for `path`, detached, without waiting for it.
///
/// `open`/`xdg-open` return as soon as the app is handed the file, so the
/// finisher does not block on the GUI; a failure to *launch* is still reported.
pub fn open_path(path: &str) -> Result<()> {
    let (program, args) = opener_argv(path);
    let mut command = Command::new(program);
    command
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::daemon::detach_command(&mut command);
    command
        .spawn()
        .with_context(|| format!("spawning {program} for {path}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_the_last_argument() {
        let (program, args) = opener_argv("/repo/src/main.rs");
        assert!(!program.is_empty());
        assert_eq!(args.last().map(String::as_str), Some("/repo/src/main.rs"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_uses_open() {
        assert_eq!(
            opener_argv("/repo/a b.rs"),
            ("open", vec!["/repo/a b.rs".to_owned()],)
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    #[test]
    fn unix_fallback_uses_xdg_open() {
        assert_eq!(
            opener_argv("/repo/a.rs"),
            ("xdg-open", vec!["/repo/a.rs".to_owned()])
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_starts_through_cmd_with_an_empty_title() {
        assert_eq!(
            opener_argv(r"C:\repo\a.rs"),
            (
                "cmd",
                vec![
                    "/C".to_owned(),
                    "start".to_owned(),
                    String::new(),
                    r"C:\repo\a.rs".to_owned(),
                ],
            )
        );
    }
}
