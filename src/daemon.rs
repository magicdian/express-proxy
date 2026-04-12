use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn spawn_daemon(config_path: &Path, pid_file: &Path) -> Result<u32> {
    if let Some(existing_pid) = read_pid(pid_file)?
        && process_exists(existing_pid)
    {
        bail!(
            "daemon appears to be already running (pid {}). remove {} if stale",
            existing_pid,
            pid_file.display()
        );
    }

    let current_exe =
        std::env::current_exe().context("failed to resolve current executable path")?;

    let mut cmd = Command::new(&current_exe);
    cmd.arg("run")
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    {
        // SAFETY: Calling setsid before exec is a standard detach operation.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    let child = cmd.spawn().context("failed to spawn daemon process")?;
    let pid = child.id();

    if let Some(parent) = pid_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(pid_file, pid.to_string())
        .with_context(|| format!("failed to write pid file {}", pid_file.display()))?;

    Ok(pid)
}

fn read_pid(path: &Path) -> Result<Option<u32>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read pid file {}", path.display()))?;

    let pid = raw
        .trim()
        .parse::<u32>()
        .with_context(|| format!("invalid pid contents in {}", path.display()))?;

    Ok(Some(pid))
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    // SAFETY: kill with signal 0 only checks process existence.
    let rc = unsafe { libc::kill(pid as i32, 0) };
    if rc == 0 {
        return true;
    }

    matches!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(code) if code == libc::EPERM
    )
}

#[cfg(not(unix))]
fn process_exists(_pid: u32) -> bool {
    false
}
