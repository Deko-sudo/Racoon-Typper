//! OS delivery of a frontend-rendered PNG. No score or database policy lives here.
use std::{fs, io::Write, path::Path};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::error::AppError;

const MAX_PNG_BYTES: usize = 4 * 1024 * 1024;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CardExportOutcome {
    saved_path: Option<String>,
    copied: bool,
}

#[tauri::command]
pub(crate) async fn save_result_card<R: Runtime>(
    app: AppHandle<R>,
    png: Vec<u8>,
) -> Result<CardExportOutcome, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let pictures = app.path().picture_dir().ok();
        deliver_card(
            &png,
            pictures.as_deref(),
            &chrono::Utc::now().format("%Y%m%d-%H%M%S-%3f").to_string(),
            |image| app.clipboard().write_image(&image).is_ok(),
        )
    })
    .await
    .map_err(|_| AppError::Internal("result card worker failed".into()))?
}

fn deliver_card(
    png: &[u8],
    pictures: Option<&Path>,
    timestamp: &str,
    copy: impl FnOnce(tauri::image::Image<'static>) -> bool,
) -> Result<CardExportOutcome, AppError> {
    // Reject malformed/oversized input before touching disk or the clipboard.
    if png.len() > MAX_PNG_BYTES || !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(AppError::InvalidConfig("invalid result PNG".into()));
    }
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(png), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(900);
    limits.max_image_height = Some(900);
    limits.max_alloc = Some(16 * 1024 * 1024);
    reader.limits(limits);
    let rgba = reader
        .decode()
        .map_err(|_| AppError::InvalidConfig("invalid result PNG".into()))?
        .into_rgba8();
    if rgba.width() != 900 || rgba.height() < 470 {
        return Err(AppError::InvalidConfig(
            "invalid result card dimensions".into(),
        ));
    }
    let (width, height) = rgba.dimensions();
    // Neither delivery failure suppresses the other operation. No private error
    // details cross IPC; only an acknowledged saved path is returned to the user.
    let saved_path = pictures.and_then(|dir| save_png(dir, timestamp, png).ok());
    let copied = copy(tauri::image::Image::new_owned(
        rgba.into_raw(),
        width,
        height,
    ));
    Ok(CardExportOutcome { saved_path, copied })
}

fn save_png(pictures: &Path, timestamp: &str, png: &[u8]) -> std::io::Result<String> {
    let directory = pictures.join("Raccoon Typer");
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&directory)?;
    if !fs::symlink_metadata(&directory)?.file_type().is_dir() {
        return Err(std::io::Error::other("invalid gallery directory"));
    }
    for suffix in 0..100 {
        let path = directory.join(format!("racoon-typper-{timestamp}-{suffix}.png"));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = match options.open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let written = file.write_all(png).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = written {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        return Ok(path.to_string_lossy().into_owned());
    }
    Err(std::io::Error::other("gallery filename collision limit"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Gallery(PathBuf);
    impl Gallery {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "racoon-gallery-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Gallery {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::RgbaImage::new(width, height)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn save_acknowledges_exact_png_and_copies_image_pixels() {
        let gallery = Gallery::new();
        let bytes = png(900, 470);
        let result = deliver_card(&bytes, Some(&gallery.0), "test", |image| {
            assert_eq!((image.width(), image.height()), (900, 470));
            assert_eq!(image.rgba().len(), 900 * 470 * 4);
            true
        })
        .unwrap();
        let path = PathBuf::from(result.saved_path.unwrap());
        assert_eq!(path.parent().unwrap(), gallery.0.join("Raccoon Typer"));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(result.copied);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn delivery_reports_partial_and_total_failures_independently() {
        let gallery = Gallery::new();
        let bytes = png(900, 900);
        let saved = deliver_card(&bytes, Some(&gallery.0), "test", |_| false).unwrap();
        assert!(saved.saved_path.is_some());
        assert!(!saved.copied);
        let copied = deliver_card(&bytes, None, "test", |_| true).unwrap();
        assert!(copied.saved_path.is_none());
        assert!(copied.copied);
        fs::write(gallery.0.join("blocked"), "file").unwrap();
        let failed =
            deliver_card(&bytes, Some(&gallery.0.join("blocked")), "test", |_| false).unwrap();
        assert!(failed.saved_path.is_none());
        assert!(!failed.copied);
    }

    #[test]
    fn rejects_invalid_and_oversized_png_before_delivery() {
        let gallery = Gallery::new();
        for bytes in [
            vec![],
            b"secret-not-png".to_vec(),
            b"\x89PNG\r\n\x1a\n".to_vec(),
            vec![0; MAX_PNG_BYTES + 1],
            png(901, 470),
            png(900, 901),
            png(1, 1),
        ] {
            assert!(deliver_card(&bytes, Some(&gallery.0), "test", |_| panic!(
                "must not copy invalid input"
            ))
            .is_err());
            assert!(!gallery.0.join("Raccoon Typer").exists());
        }
    }

    #[test]
    fn filename_collisions_never_overwrite_existing_files() {
        let gallery = Gallery::new();
        let first = save_png(&gallery.0, "same", b"first").unwrap();
        let second = save_png(&gallery.0, "same", b"second").unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(first).unwrap(), b"first");
        assert_eq!(fs::read(second).unwrap(), b"second");
        for _ in 2..100 {
            save_png(&gallery.0, "same", b"next").unwrap();
        }
        assert!(save_png(&gallery.0, "same", b"overflow").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_gallery_symlink_but_still_copies() {
        let gallery = Gallery::new();
        let target = Gallery::new();
        std::os::unix::fs::symlink(&target.0, gallery.0.join("Raccoon Typer")).unwrap();
        let result = deliver_card(&png(900, 470), Some(&gallery.0), "test", |_| true).unwrap();
        assert!(result.saved_path.is_none());
        assert!(result.copied);
        assert_eq!(fs::read_dir(&target.0).unwrap().count(), 0);
    }
}
