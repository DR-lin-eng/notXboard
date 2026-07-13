use crate::*;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

pub(crate) const THEME_ARCHIVE_MAX_BYTES: usize = 10 * 1024 * 1024;
pub(crate) const PLUGIN_ARCHIVE_MAX_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct ArchiveExtractionLimits {
    pub(crate) max_entries: usize,
    pub(crate) max_total_uncompressed_bytes: u64,
    pub(crate) max_file_uncompressed_bytes: u64,
}

pub(crate) const THEME_ARCHIVE_LIMITS: ArchiveExtractionLimits = ArchiveExtractionLimits {
    max_entries: 512,
    max_total_uncompressed_bytes: 50 * 1024 * 1024,
    max_file_uncompressed_bytes: 10 * 1024 * 1024,
};

pub(crate) const PLUGIN_ARCHIVE_LIMITS: ArchiveExtractionLimits = ArchiveExtractionLimits {
    max_entries: 512,
    max_total_uncompressed_bytes: 50 * 1024 * 1024,
    max_file_uncompressed_bytes: 10 * 1024 * 1024,
};

pub(crate) struct PrivateTempDirectory {
    path: Option<PathBuf>,
}

impl PrivateTempDirectory {
    pub(crate) fn create(path: PathBuf) -> std::io::Result<Self> {
        create_private_directory(&path)?;
        Ok(Self { path: Some(path) })
    }

    pub(crate) fn path(&self) -> &Path {
        self.path.as_deref().expect("temporary directory is active")
    }

    pub(crate) fn into_path(mut self) -> PathBuf {
        self.path.take().expect("temporary directory is active")
    }
}

impl Drop for PrivateTempDirectory {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

pub(crate) fn create_private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

pub(crate) fn validate_zip_metadata<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    limits: ArchiveExtractionLimits,
    message: &'static str,
) -> Result<(), Response<Body>> {
    if archive.len() > limits.max_entries {
        return Err(upload_limit_response(message));
    }

    let mut total = 0_u64;
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|_| upload_limit_response(message))?;
        if file.is_dir() {
            continue;
        }

        let file_size = file.size();
        if file_size > limits.max_file_uncompressed_bytes {
            return Err(upload_limit_response(message));
        }
        total = total
            .checked_add(file_size)
            .ok_or_else(|| upload_limit_response(message))?;
        if total > limits.max_total_uncompressed_bytes {
            return Err(upload_limit_response(message));
        }
    }

    Ok(())
}

pub(crate) fn copy_zip_entry_limited<R: Read, W: Write>(
    source: &mut R,
    destination: &mut W,
    total_written: &mut u64,
    limits: ArchiveExtractionLimits,
    message: &'static str,
) -> Result<(), Response<Body>> {
    let written = std::io::copy(
        &mut source
            .by_ref()
            .take(limits.max_file_uncompressed_bytes + 1),
        destination,
    )
    .map_err(|_| upload_limit_response(message))?;

    if written > limits.max_file_uncompressed_bytes {
        return Err(upload_limit_response(message));
    }

    *total_written = total_written
        .checked_add(written)
        .ok_or_else(|| upload_limit_response(message))?;
    if *total_written > limits.max_total_uncompressed_bytes {
        return Err(upload_limit_response(message));
    }

    Ok(())
}

fn upload_limit_response(message: &'static str) -> Response<Body> {
    json_status_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "message": message }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_temp_directory_is_removed_on_drop() {
        let path = std::env::temp_dir().join(format!(
            "notxboard-private-temp-test-{}",
            uuid::Uuid::new_v4()
        ));
        {
            let directory =
                PrivateTempDirectory::create(path.clone()).expect("create private temp directory");
            assert_eq!(directory.path(), path.as_path());
            assert!(path.is_dir());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(&path)
                    .expect("read private temp metadata")
                    .permissions()
                    .mode()
                    & 0o777;
                assert_eq!(mode, 0o700);
            }
        }
        assert!(!path.exists());
    }
}
