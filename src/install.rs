#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::{Result, bail};
use std::path::Path;
#[cfg(target_os = "linux")]
use std::{fs, process::Command};

pub fn install_systemd_user_service(config_path: &Path) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = config_path;
        bail!("`eproxy install` is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        ensure_systemctl_available()?;

        let home = dirs::home_dir().context("unable to resolve user home directory")?;
        let systemd_dir = home.join(".config/systemd/user");
        fs::create_dir_all(&systemd_dir)
            .with_context(|| format!("failed to create {}", systemd_dir.display()))?;

        let exe = std::env::current_exe().context("failed to resolve current executable path")?;
        let service_path = systemd_dir.join("eproxy.service");

        let unit = format!(
            "[Unit]\nDescription=eproxy local SNI forwarder\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nType=simple\nExecStart={} run --config {}\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=default.target\n",
            shell_escape(exe.as_os_str().to_string_lossy().as_ref()),
            shell_escape(config_path.display().to_string().as_str()),
        );

        fs::write(&service_path, unit)
            .with_context(|| format!("failed to write {}", service_path.display()))?;

        run_systemctl_user(["daemon-reload"])?;
        run_systemctl_user(["enable", "--now", "eproxy.service"])?;

        println!("Installed user service: {}", service_path.display());
        println!("Service status: systemctl --user status eproxy.service");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn ensure_systemctl_available() -> Result<()> {
    let status = Command::new("systemctl")
        .arg("--version")
        .status()
        .context("failed to execute `systemctl --version`")?;

    if !status.success() {
        bail!("systemctl is not available on this machine");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_systemctl_user<const N: usize>(args: [&str; N]) -> Result<()> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .with_context(|| format!("failed to run systemctl --user {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "systemctl --user {} failed: {}",
            args.join(" "),
            stderr.trim()
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn shell_escape(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./:".contains(c))
    {
        return value.to_string();
    }

    let escaped = value.replace('"', "\\\"");
    format!("\"{escaped}\"")
}
