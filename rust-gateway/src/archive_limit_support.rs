use crate::*;
use std::io::{Read, Seek, Write};

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
