#[macro_use] extern crate log;

mod cli;
mod game;
mod tasks;
#[cfg(feature="graphics")]
mod menus;
mod helpers;
pub mod prelude; 
mod managers;
mod integrations;


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