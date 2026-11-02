//! Where the bytes of the copies live (spec 27.2): a folder, or an
//! S3-compatible bucket such as Cloudflare R2. They are sealed by the owner,
//! so whoever keeps them keeps bytes nobody can open.

use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path as Key;
use object_store::{GetOptions, GetRange, ObjectStore, ObjectStoreExt, PutPayload, WriteMultipart};
use tokio::io::AsyncReadExt;

use crate::config::{self, Bucket};
use crate::error::ApiError;

/// Up to this size a copy goes in one request; bigger ones in parts.
const ONE_REQUEST: u64 = 8 << 20;
const PART: usize = 1 << 20;

#[derive(Clone)]
pub struct CopyStore {
    inner: Arc<dyn ObjectStore>,
}

fn failed(what: &str, err: &object_store::Error) -> ApiError {
    tracing::error!("storage {what} failed ({})", reason(err));
    ApiError::internal()
}

/// Why the store said no: the kind of failure and, when a bucket answered,
/// its status and its own code (`InvalidAccessKeyId` for a wrong key). Never
/// the key or the rest of the answer: they name an account.
fn reason(err: &object_store::Error) -> String {
    let kind = match err {
        object_store::Error::NotFound { .. } => "not found",
        object_store::Error::Precondition { .. } => "precondition",
        object_store::Error::PermissionDenied { .. } => "permission denied",
        object_store::Error::Unauthenticated { .. } => "unauthenticated",
        _ => "other",
    };
    let text = err.to_string();
    let status = between(&text, "status code: ", " ")
        .filter(|status| status.len() == 3 && status.bytes().all(|b| b.is_ascii_digit()));
    let code = between(&text, "<Code>", "</Code>").filter(|code| {
        !code.is_empty() && code.len() <= 64 && code.bytes().all(|b| b.is_ascii_alphanumeric())
    });
    [Some(kind), status, code]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
}

/// The text between the first `start` and the `end` after it.
fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let to = from + text[from..].find(end)?;
    Some(&text[from..to])
}

impl CopyStore {
    /// Copies in a folder.
    pub fn disk(dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir)?;
        Ok(Self {
            inner: Arc::new(LocalFileSystem::new_with_prefix(dir)?),
        })
    }

    /// Copies in memory, for tests.
    pub fn memory() -> Self {
        Self {
            inner: Arc::new(InMemory::new()),
        }
    }

    /// Copies in a bucket.
    pub fn bucket(bucket: &Bucket) -> anyhow::Result<Self> {
        let secret = config::secret(
            bucket.secret_file.as_deref(),
            config::BUCKET_SECRET_VAR,
            "bucket secret",
        )?;
        let built = AmazonS3Builder::new()
            .with_endpoint(&bucket.endpoint)
            .with_bucket_name(&bucket.name)
            .with_region(&bucket.region)
            .with_access_key_id(&bucket.access_key_id)
            .with_secret_access_key(secret)
            .with_virtual_hosted_style_request(false)
            .build()?;
        Ok(Self {
            inner: Arc::new(built),
        })
    }

    /// Stores the file at `path` (`size` bytes) under `key`.
    pub async fn put_file(&self, key: &str, path: &Path, size: u64) -> Result<(), ApiError> {
        let location = Key::from(key);
        let io = |err: std::io::Error| {
            tracing::error!("reading an upload failed: {}", err.kind());
            ApiError::internal()
        };
        if size <= ONE_REQUEST {
            let bytes = tokio::fs::read(path).await.map_err(io)?;
            self.inner
                .put(&location, PutPayload::from(bytes))
                .await
                .map_err(|err| failed("put", &err))?;
            return Ok(());
        }
        let upload = self
            .inner
            .put_multipart(&location)
            .await
            .map_err(|err| failed("put", &err))?;
        let mut writer = WriteMultipart::new(upload);
        let mut file = tokio::fs::File::open(path).await.map_err(io)?;
        let mut buffer = vec![0u8; PART];
        loop {
            let read = file.read(&mut buffer).await.map_err(io)?;
            if read == 0 {
                break;
            }
            writer
                .wait_for_capacity(4)
                .await
                .map_err(|err| failed("put", &err))?;
            writer.write(&buffer[..read]);
        }
        writer.finish().await.map_err(|err| failed("put", &err))?;
        Ok(())
    }

    /// The bytes of `key`, all of them or just `range`.
    pub async fn read(
        &self,
        key: &str,
        range: Option<Range<u64>>,
    ) -> Result<BoxStream<'static, std::io::Result<Bytes>>, ApiError> {
        let options = GetOptions {
            range: range.map(GetRange::Bounded),
            ..GetOptions::default()
        };
        let got = self
            .inner
            .get_opts(&Key::from(key), options)
            .await
            .map_err(|err| match err {
                object_store::Error::NotFound { .. } => ApiError::not_found(),
                err => failed("get", &err),
            })?;
        Ok(got
            .into_stream()
            .map(|chunk| chunk.map_err(std::io::Error::other))
            .boxed())
    }

    /// Removes `key`. A key that is not there is already as wanted.
    pub async fn delete(&self, key: &str) {
        match self.inner.delete(&Key::from(key)).await {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
            Err(err) => {
                let _ = failed("delete", &err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn all(store: &CopyStore, key: &str, range: Option<Range<u64>>) -> Vec<u8> {
        let mut stream = store.read(key, range).await.expect("read");
        let mut out = Vec::new();
        while let Some(chunk) = stream.next().await {
            out.extend_from_slice(&chunk.expect("chunk"));
        }
        out
    }

    /// A small copy goes in one request and a big one in parts; both come back
    /// whole or in a range, and go away when deleted.
    async fn round_trip(store: CopyStore) {
        let dir = tempfile::tempdir().expect("dir");
        for (name, size) in [
            ("small", 5_000usize),
            ("big", (ONE_REQUEST as usize) + 3_000_000),
        ] {
            let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            let path = dir.path().join(name);
            std::fs::write(&path, &data).expect("write");
            store.put_file(name, &path, size as u64).await.expect("put");
            assert_eq!(all(&store, name, None).await, data);
            assert_eq!(all(&store, name, Some(1000..1010)).await, data[1000..1010]);
            store.delete(name).await;
            assert!(store.read(name, None).await.is_err());
            // Deleting what is not there is fine.
            store.delete(name).await;
        }
    }

    #[tokio::test]
    async fn copies_round_trip_in_memory() {
        round_trip(CopyStore::memory()).await;
    }

    #[tokio::test]
    async fn copies_round_trip_on_disk() {
        let dir = tempfile::tempdir().expect("dir");
        round_trip(CopyStore::disk(&dir.path().join("copies")).expect("store")).await;
    }

    /// What a bucket that answers every request with `status` and `body`
    /// says to a put.
    async fn refused(status: axum::http::StatusCode, body: &'static str) -> object_store::Error {
        let app = axum::Router::new().fallback(move || async move { (status, body) });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        let bucket = AmazonS3Builder::new()
            .with_endpoint(format!("http://{addr}"))
            .with_allow_http(true)
            .with_bucket_name("copies")
            .with_region("auto")
            .with_access_key_id("COLE_AQUI_O_ACCESS_KEY_ID")
            .with_secret_access_key("secret")
            .with_virtual_hosted_style_request(false)
            .build()
            .expect("bucket");
        bucket
            .put(
                &Key::from("7/files/fil_a1b2"),
                PutPayload::from_static(b"sealed"),
            )
            .await
            .expect_err("refused")
    }

    /// A bucket that refuses the key says so in the log, with its status
    /// and code, and the log never names the file or its account.
    #[tokio::test]
    async fn a_refusal_says_why_without_the_key() {
        use axum::http::StatusCode;
        let wrong_key = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error>\
            <Code>InvalidAccessKeyId</Code><Message>The key is not valid.</Message></Error>";
        let said = reason(&refused(StatusCode::FORBIDDEN, wrong_key).await);
        assert_eq!(said, "permission denied, 403, InvalidAccessKeyId");

        let unsigned = "<Error><Code>Unauthorized</Code></Error>";
        let said = reason(&refused(StatusCode::UNAUTHORIZED, unsigned).await);
        assert_eq!(said, "unauthenticated, 401, Unauthorized");

        // A code that is not a plain word stays out; the status alone goes.
        let odd = "<Error><Code>7/files/fil_a1b2</Code></Error>";
        let said = reason(&refused(StatusCode::BAD_REQUEST, odd).await);
        assert_eq!(said, "other, 400");
        assert!(!said.contains("fil_a1b2"));
    }
}
