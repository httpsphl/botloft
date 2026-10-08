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
    /// A header that holds the client's address, such as `CF-Connecting-IP`
    /// behind Cloudflare. Trust it only when nothing but that proxy can reach
    /// the server, because anyone else could write it. Wins over
    /// `X-Forwarded-For`.
    #[serde(default)]
    pub client_ip_header: Option<String>,
    /// The built files of the phone page (`app/dist-phone`), which the
    /// server serves at `/m` (spec 28.7). Without it there is no page.
    #[serde(default)]
    pub phone_dir: Option<PathBuf>,
    /// Notices to phones through their browser's push service (spec 28.8).
    /// Without it, a phone shows only what is there when it is opened.
    #[serde(default)]
    pub push: Option<Push>,
    #[serde(default)]
    pub storage: Storage,
    /// Where the copies go when `storage = "bucket"`.
    pub bucket: Option<Bucket>,
    pub smtp: Smtp,
}

/// Web Push (spec 28.8): the server signs its calls to the push services
/// with a key of its own (VAPID, RFC 8292) and sends no content.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Push {
    /// Who runs this server, for the push services: a `mailto:` or `https:`
    /// address.
    pub subject: String,
    /// A file with the private key alone (`botloft-cloud vapid-key` makes
    /// one). The variable `BOTLOFT_CLOUD_VAPID_KEY` does the same, and wins.
    #[serde(default)]
    pub vapid_key_file: Option<PathBuf>,
    /// The push services a phone may name, by host (a host also covers its
    /// subdomains). Empty means the ones of the browsers: Chrome, Firefox,
    /// Edge and Safari. The server calls only these, so a phone cannot make
    /// it call another address.
    #[serde(default)]
    pub allow_hosts: Vec<String>,
}

/// Where the bytes of the copies are kept (spec 27.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Storage {
    /// In `data_dir`.
    #[default]
    Disk,
    /// An S3-compatible bucket, such as Cloudflare R2.
    Bucket,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bucket {
    pub endpoint: String,
    pub name: String,
    #[serde(default = "default_region")]
    pub region: String,
    pub access_key_id: String,
    /// A file with the secret alone, so it never sits in `cloud.toml`. The
    /// variable `BOTLOFT_CLOUD_BUCKET_SECRET` does the same, and wins.
    #[serde(default)]
    pub secret_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Smtp {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub user: String,
    /// A file with the password alone, so it never sits in this one. The
    /// variable `BOTLOFT_CLOUD_SMTP_PASSWORD` does the same, and wins.
    #[serde(default)]
    pub password_file: Option<PathBuf>,
    pub from: String,
}

/// The whole configuration as text, for a platform that keeps settings
/// (Coolify, for one) rather than files.
pub const CONFIG_VAR: &str = "BOTLOFT_CLOUD_CONFIG";

/// The variables that can carry a secret, for the same platforms.
pub const SMTP_PASSWORD_VAR: &str = "BOTLOFT_CLOUD_SMTP_PASSWORD";
pub const BUCKET_SECRET_VAR: &str = "BOTLOFT_CLOUD_BUCKET_SECRET";
pub const VAPID_KEY_VAR: &str = "BOTLOFT_CLOUD_VAPID_KEY";

/// A secret from the variable `var` if it is set, else from `file`. `what`
/// names it in the error, which never holds the value.
pub fn secret(file: Option<&Path>, var: &str, what: &str) -> anyhow::Result<String> {
    secret_from(file, std::env::var(var).ok(), var, what)
}

fn secret_from(
    file: Option<&Path>,
    value: Option<String>,
    var: &str,
    what: &str,
) -> anyhow::Result<String> {
    if let Some(value) = value
        .map(|value| value.trim().to_owned())
        .filter(|v| !v.is_empty())
    {
        return Ok(value);
    }
    match file {
        Some(file) => std::fs::read_to_string(file)
            .map(|text| text.trim().to_owned())
            .map_err(|err| anyhow::anyhow!("cannot read the {what} file: {}", err.kind())),
        None => anyhow::bail!("the {what} is missing: set {var}, or a file in cloud.toml"),
    }
}

fn default_listen() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8787))
}
fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}
fn default_quota() -> u64 {
    200 << 20
}
fn default_max_copy() -> u64 {
    50 << 20
}
fn default_keep() -> u32 {
    5
}
fn default_true() -> bool {
    true
}
fn default_region() -> String {
    "auto".to_owned()
}
fn default_port() -> u16 {
    587
}

impl Config {
    /// The configuration: the text in `BOTLOFT_CLOUD_CONFIG` if that is set
    /// (for a platform that keeps settings, not files), else the file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::env::var(CONFIG_VAR)
            .ok()
            .filter(|text| !text.trim().is_empty())
        {
            Some(text) => Self::parse(&text, CONFIG_VAR),
            None => {
                let text = std::fs::read_to_string(path)
                    .map_err(|err| anyhow::anyhow!("cannot read {}: {err}", path.display()))?;
                Self::parse(&text, &path.display().to_string())
            }
        }
    }

    /// `text` as a configuration; `from` names where it came from.
    pub fn parse(text: &str, from: &str) -> anyhow::Result<Self> {
        let config: Self =
            toml::from_str(text).map_err(|err| anyhow::anyhow!("{from} is not valid: {err}"))?;
        if !config.public_url.starts_with("https://")
            && !config.public_url.starts_with("http://127.0.0.1")
        {
            anyhow::bail!("public_url must be https (or http://127.0.0.1 for tests)");
        }
        if let Some(name) = &config.client_ip_header
            && axum::http::HeaderName::from_bytes(name.trim().as_bytes()).is_err()
        {
            anyhow::bail!("client_ip_header is not a header name");
        }
        if let Some(push) = &config.push
            && !push.subject.starts_with("mailto:")
            && !push.subject.starts_with("https://")
        {
            anyhow::bail!("[push] subject must be a mailto: or https: address");
        }
        if config.storage == Storage::Bucket && config.bucket.is_none() {
            anyhow::bail!("storage = \"bucket\" needs a [bucket] section");
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
            client_ip_header: None,
            phone_dir: None,
            push: None,
            storage: Storage::Disk,
            bucket: None,
            smtp: Smtp {
                host: "localhost".to_owned(),
                port: 587,
                user: String::new(),
                password_file: None,
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
    fn a_secret_comes_from_the_variable_first_then_the_file() {
        let (_dir, path) = write("  from-the-file \n");
        let var = Some("  from-the-variable ".to_owned());
        let said = |file: Option<&Path>, value: Option<String>| secret_from(file, value, "V", "x");
        assert_eq!(
            said(Some(&path), var.clone()).expect("var"),
            "from-the-variable"
        );
        assert_eq!(said(Some(&path), None).expect("file"), "from-the-file");
        // An empty variable counts as not set.
        assert_eq!(
            said(Some(&path), Some("  ".to_owned())).expect("file"),
            "from-the-file"
        );
        assert_eq!(said(None, var).expect("var alone"), "from-the-variable");
        let nothing = said(None, None).expect_err("nothing");
        assert!(nothing.to_string().contains("set V"), "{nothing}");
        let gone = said(Some(&path.with_extension("none")), None).expect_err("missing file");
        assert!(
            gone.to_string().contains("cannot read the x file"),
            "{gone}"
        );
    }

    #[test]
    fn push_needs_a_subject_that_says_who_runs_the_server() {
        let with = |subject: &str| {
            let (_dir, path) = write(&format!(
                "public_url = \"https://cloud.example.org\"
                 [smtp]
host = \"mail\"
user = \"u\"
from = \"a <a@b.c>\"
                 [push]
subject = \"{subject}\"
vapid_key_file = \"k\"
"
            ));
            Config::load(&path)
        };
        for good in ["mailto:ops@example.org", "https://example.org/contact"] {
            let config = with(good).expect(good);
            let push = config.push.expect("push");
            assert!(
                push.allow_hosts.is_empty(),
                "the browsers' services by default"
            );
        }
        for bad in ["ops@example.org", "http://example.org", ""] {
            assert!(with(bad).is_err(), "{bad}");
        }
        let (_dir, path) = write(
            "public_url = \"https://cloud.example.org\"
[smtp]
host = \"mail\"
user = \"u\"
             from = \"a <a@b.c>\"
",
        );
        assert!(Config::load(&path).expect("load").push.is_none());
    }

    #[test]
    fn a_small_file_gets_the_defaults() {
        let (_dir, path) = write(
            "public_url = \"https://cloud.example.org/\"\n[smtp]\nhost = \"mail\"\nuser = \"u\"\n\
             password_file = \"p\"\nfrom = \"Botloft <a@b.c>\"\n",
        );
        let config = Config::load(&path).expect("load");
        assert_eq!(config.keep, 5);
        assert_eq!(config.quota_bytes, 200 << 20);
        assert_eq!(config.max_copy_bytes, 50 << 20);
        assert_eq!(config.smtp.port, 587);
        assert_eq!(config.base_url(), "https://cloud.example.org");
    }

    #[test]
    fn a_bucket_needs_its_section() {
        let (_dir, path) = write(
            "public_url = \"https://cloud.example.org\"
storage = \"bucket\"
[smtp]
host = \"m\"
             user = \"u\"
password_file = \"p\"
from = \"a@b.c\"
",
        );
        assert!(Config::load(&path).is_err());
        let (_dir, path) = write(
            "public_url = \"https://cloud.example.org\"
storage = \"bucket\"
[bucket]
             endpoint = \"https://x.r2.cloudflarestorage.com\"
name = \"b\"
access_key_id = \"k\"
             secret_file = \"s\"
[smtp]
host = \"m\"
user = \"u\"
password_file = \"p\"
             from = \"a@b.c\"
",
        );
        let config = Config::load(&path).expect("load");
        assert_eq!(config.bucket.expect("bucket").region, "auto");
    }

    #[test]
    fn the_whole_config_can_be_text() {
        let text = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/cloud/cloud.example.toml"),
        )
        .expect("the example");
        let config = Config::parse(&text, "text").expect("parses");
        assert_eq!(config.keep, 5);
        let broken = Config::parse("public_url = 3", "the variable").expect_err("not valid");
        assert!(
            broken.to_string().starts_with("the variable is not valid"),
            "{broken}"
        );
    }

    /// The file the deploy folder ships is a config the server accepts.
    #[test]
    fn the_example_config_is_valid() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../deploy/cloud/cloud.example.toml");
        let config = Config::load(&path).expect("the example loads");
        assert_eq!(config.storage, Storage::Disk);
        assert_eq!(config.quota_bytes, 200 << 20);
        assert_eq!(config.max_copy_bytes, 50 << 20);
        assert_eq!(config.keep, 5);
        assert!(config.behind_proxy);
    }

    #[test]
    fn the_client_ip_header_must_be_a_header_name() {
        let with = |name: &str| {
            let (_dir, path) = write(&format!(
                "public_url = \"https://cloud.example.org\"\nclient_ip_header = \"{name}\"\n\
                 [smtp]\nhost = \"m\"\nuser = \"u\"\nfrom = \"a@b.c\"\n"
            ));
            Config::load(&path)
        };
        assert_eq!(
            with("CF-Connecting-IP")
                .expect("a header")
                .client_ip_header
                .as_deref(),
            Some("CF-Connecting-IP")
        );
        assert!(with("not a header").is_err());
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
