//! `cloud.toml` (spec 27.2).

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: SocketAddr,
    /// The public address, which goes in the links of the e-mails.
    pub public_url: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    #[serde(default = "default_quota")]
    pub quota_bytes: u64,
    #[serde(default = "default_max_copy")]
    pub max_copy_bytes: u64,
    #[serde(default = "default_keep")]
    pub keep: u32,
    /// A proxy in front sets `X-Forwarded-For`; its last value is the client.
    #[serde(default = "default_true")]
    pub behind_proxy: bool,
    pub smtp: Smtp,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Smtp {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub user: String,
    /// A file with the password alone, so it never sits in this one.
    pub password_file: PathBuf,
    pub from: String,
}

fn default_listen() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8787))
}
fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}
fn default_quota() -> u64 {
    2 << 30
}
fn default_max_copy() -> u64 {
    1 << 30
}
fn default_keep() -> u32 {
    3
}
fn default_true() -> bool {
    true
}
fn default_port() -> u16 {
    587
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|err| anyhow::anyhow!("cannot read {}: {err}", path.display()))?;
        let config: Self = toml::from_str(&text)
            .map_err(|err| anyhow::anyhow!("{} is not valid: {err}", path.display()))?;
        if !config.public_url.starts_with("https://")
            && !config.public_url.starts_with("http://127.0.0.1")
        {
            anyhow::bail!("public_url must be https (or http://127.0.0.1 for tests)");
        }
        Ok(config)
    }

    /// The link base, without a closing slash.
    pub fn base_url(&self) -> &str {
        self.public_url.trim_end_matches('/')
    }

    /// A config for tests: nothing on disk and a mailer that never connects.
    pub fn for_tests() -> Self {
        Self {
            listen: default_listen(),
            public_url: "http://127.0.0.1:8787".to_owned(),
            data_dir: PathBuf::from("."),
            quota_bytes: default_quota(),
            max_copy_bytes: default_max_copy(),
            keep: default_keep(),
            behind_proxy: true,
            smtp: Smtp {
                host: "localhost".to_owned(),
                port: 587,
                user: String::new(),
                password_file: PathBuf::new(),
                from: "Botloft <noreply@localhost>".to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(text: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("cloud.toml");
        std::fs::write(&path, text).expect("write");
        (dir, path)
    }

    #[test]
    fn a_small_file_gets_the_defaults() {
        let (_dir, path) = write(
            "public_url = \"https://cloud.example.org/\"\n[smtp]\nhost = \"mail\"\nuser = \"u\"\n\
             password_file = \"p\"\nfrom = \"Botloft <a@b.c>\"\n",
        );
        let config = Config::load(&path).expect("load");
        assert_eq!(config.keep, 3);
        assert_eq!(config.quota_bytes, 2 << 30);
        assert_eq!(config.smtp.port, 587);
        assert_eq!(config.base_url(), "https://cloud.example.org");
    }

    #[test]
    fn an_http_address_is_refused() {
        let (_dir, path) = write(
            "public_url = \"http://cloud.example.org\"\n[smtp]\nhost = \"m\"\nuser = \"u\"\n\
             password_file = \"p\"\nfrom = \"a@b.c\"\n",
        );
        assert!(Config::load(&path).is_err());
    }
}
