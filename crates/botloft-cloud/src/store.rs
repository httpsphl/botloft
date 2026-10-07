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

use crate::config::Bucket;
use crate::error::ApiError;

/// Up to this size a copy goes in one request; bigger ones in parts.
const ONE_REQUEST: u64 = 8 << 20;
const PART: usize = 1 << 20;

#[derive(Clone)]
pub struct CopyStore {
    inner: Arc<dyn ObjectStore>,
}

fn failed(what: &str, err: &object_store::Error) -> ApiError {
    // The kind of failure, never the key: it names an account.
    let kind = match err {
        object_store::Error::NotFound { .. } => "not found",
        object_store::Error::Precondition { .. } => "precondition",
        _ => "other",
    };
    tracing::error!("storage {what} failed ({kind})");
    ApiError::internal()
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
        let secret = std::fs::read_to_string(&bucket.secret_file)
            .map_err(|err| anyhow::anyhow!("cannot read the bucket secret file: {err}"))?;
        let built = AmazonS3Builder::new()
            .with_endpoint(&bucket.endpoint)
            .with_bucket_name(&bucket.name)
            .with_region(&bucket.region)
            .with_access_key_id(&bucket.access_key_id)
            .with_secret_access_key(secret.trim())
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
}
