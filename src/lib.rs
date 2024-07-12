#![cfg_attr(not(feature="graphics"), allow(unused))]
#![deny(unused_must_use)] // ensure all futures are awaited

#[macro_use] extern crate log;
pub mod engine;
pub mod tataku;
pub mod prelude;
pub mod interface;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKINS_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";



use prelude::*;
/// perform a download on another thread
pub(crate) fn perform_download(url:String, path:String, progress: Arc<RwLock<DownloadProgress>>) {
    debug!("Downloading '{url}' to '{path}'");

    tokio::spawn(async move {
        Downloader::download_existing_progress(DownloadOptions::new(url, 5), progress.clone());

        loop {
            tokio::task::yield_now().await;

            let progress = progress.read();
            if progress.complete() {
                debug!("direct download completed");
                let bytes = progress.data.as_ref().unwrap();
                std::fs::write(path, bytes).unwrap();
                break;
            }

            if progress.failed() {
                if let Some(e) = &progress.error {
                    error!("failed: {e}");
                }
                break;
            }
        }
    });
}