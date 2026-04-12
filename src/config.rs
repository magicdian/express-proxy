use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use url::Url;

pub const DEFAULT_LISTEN: &str = "127.0.0.1:9718";

#[derive(Debug, Clone)]
pub struct Layout {
    pub config_path: PathBuf,
    pub logs_dir: PathBuf,
    pub pid_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub log_level: String,
    pub listeners: Vec<ResolvedListener>,
}

#[derive(Debug, Clone)]
pub struct ResolvedListener {
    pub name: String,
    pub listen_addr: SocketAddr,
    pub upstream_base: Url,
    pub connect_addr: SocketAddr,
    pub host_header: String,
    pub server_name: String,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    log: RawLog,
    #[serde(default)]
    listeners: Vec<RawListener>,
}

#[derive(Debug, Deserialize)]
struct RawLog {
    #[serde(default = "default_log_level")]
    level: String,
}

#[derive(Debug, Deserialize)]
struct RawListener {
    name: Option<String>,
    listen: String,
    upstream: String,
    connect_ip: String,
    connect_port: u16,
    sni: Option<String>,
    host_header: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for RawLog {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

pub fn ensure_layout() -> Result<Layout> {
    let home = dirs::home_dir().context("unable to resolve user home directory")?;
    let eproxy_home = home.join(".eproxy");
    let config_path = eproxy_home.join("config.toml");
    let logs_dir = eproxy_home.join("logs");
    let pid_file = eproxy_home.join("eproxy.pid");

    fs::create_dir_all(&eproxy_home)
        .with_context(|| format!("failed to create {}", eproxy_home.display()))?;
    fs::create_dir_all(&logs_dir)
        .with_context(|| format!("failed to create {}", logs_dir.display()))?;

    Ok(Layout {
        config_path,
        logs_dir,
        pid_file,
    })
}

pub fn load_or_bootstrap(config_path: &Path) -> Result<ResolvedConfig> {
    if !config_path.exists() {
        write_default_config(config_path)?;
        bail!(
            "config file not found. A template has been created at {}",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config file {}", config_path.display()))?;
    let parsed: RawConfig = toml::from_str(&raw)
        .with_context(|| format!("invalid TOML in {}", config_path.display()))?;
    resolve_config(parsed)
}

fn resolve_config(parsed: RawConfig) -> Result<ResolvedConfig> {
    if parsed.listeners.is_empty() {
        bail!("config requires at least one [[listeners]] entry")
    }

    let mut listeners = Vec::with_capacity(parsed.listeners.len());

    for (idx, item) in parsed.listeners.into_iter().enumerate() {
        let name = item.name.unwrap_or_else(|| format!("listener-{}", idx + 1));

        let listen_addr: SocketAddr = item
            .listen
            .parse()
            .with_context(|| format!("listener '{name}' has invalid listen address"))?;

        let connect_ip: IpAddr = item
            .connect_ip
            .parse()
            .with_context(|| format!("listener '{name}' has invalid connect_ip"))?;

        if item.connect_port == 0 {
            bail!("listener '{name}' has invalid connect_port: 0");
        }

        let mut upstream_base = Url::parse(&item.upstream)
            .with_context(|| format!("listener '{name}' has invalid upstream URL"))?;

        if upstream_base.scheme() != "https" {
            bail!("listener '{name}' upstream must use https://");
        }

        let upstream_host = upstream_base
            .host_str()
            .context("upstream URL requires host")?;

        let server_name = item.sni.unwrap_or_else(|| upstream_host.to_string());

        upstream_base
            .set_host(Some(&server_name))
            .with_context(|| format!("listener '{name}' has invalid sni/server_name"))?;

        let connect_addr = SocketAddr::new(connect_ip, item.connect_port);
        let host_header = item.host_header.unwrap_or_else(|| server_name.clone());

        listeners.push(ResolvedListener {
            name,
            listen_addr,
            upstream_base,
            connect_addr,
            host_header,
            server_name,
        });
    }

    Ok(ResolvedConfig {
        log_level: parsed.log.level,
        listeners,
    })
}

pub fn write_default_config(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    let content = format!(
        r#"# eproxy configuration
#
# Local model SDK -> eproxy (HTTP loopback) -> upstream HTTPS over IX route

[log]
level = "info"

[[listeners]]
name = "default"
listen = "{default_listen}"
upstream = "https://xxx.domain.com:8443"
connect_ip = "a.b.c.d"
connect_port = 8443
# Optional: override TLS SNI. If omitted, host from `upstream` is used.
# sni = "xxx.domain.com"
# Optional: override HTTP Host header. If omitted, SNI/domain is used.
# host_header = "xxx.domain.com"
"#,
        default_listen = DEFAULT_LISTEN
    );

    fs::write(path, content)
        .with_context(|| format!("failed to write default config at {}", path.display()))
}
