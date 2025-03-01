use crate::prelude::*;
const ATTEMPTS: usize = 5;

pub struct Zip;
impl Zip {
    pub async fn extract_all(
        in_folder: impl AsRef<Path>, 
        out_folder: impl AsRef<Path>, 
        delete_archive: ArchiveDelete,
        
    ) -> Vec<String> {
        let in_folder = in_folder.as_ref();
        let out_folder = out_folder.as_ref();

        let mut paths = Vec::new();

        // get list of files to extract
        let Ok(files) = std::fs::read_dir(in_folder) else { return paths };

        for filename in files.filter_map(|f|f.ok()) {
            trace!("Archive chcking file {:?}", filename);
            
            match Self::extract_single(filename.path(), out_folder, true, delete_archive).await {
                Ok(path) => paths.push(path),
                Err(e) => {
                    error!("Error extracting zip archive: {e}");
                    // NotificationManager::add_text_notification("Error extracting file\nSee console for details", 3000.0, Color::RED).await;
                }
            }

        }
        
        paths
    }

    pub async fn extract_single(zip: impl AsRef<Path>, dir: impl AsRef<Path>, extract_to_folder: bool, delete_file: ArchiveDelete) -> TatakuResult<String> {
        let zip = zip.as_ref();
        let dir = dir.as_ref();

        // try to open the file
        let mut error_counter = 0;
        let file = loop {
            match std::fs::File::open(zip) {
                Ok(f) => break f,
                Err(e) => {
                    // if we've waited 200ms*ATTEMPTS and its still broken, give up
                    if error_counter > ATTEMPTS {
                        error!("5 errors opening archive file: {e}");
                        if let ArchiveDelete::Always = delete_file {
                            if let Err(e) = std::fs::remove_file(zip) {
                                error!("Error deleting failed archive file {e}");
                            }
                        }

                        return Err(e.into());
                    }

                    warn!("Error opening archive file: {e}");
                    error_counter += 1;

                    // wait 200ms before trying again
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }

            }
        };

        let mut dir = dir.to_path_buf();
        if extract_to_folder {
            let ext = ".".to_owned() + &zip.extension().map(|s|s.to_string_lossy().to_string()).unwrap_or(".".to_owned());
            dir = dir.join(zip.file_name().unwrap().to_str().unwrap().trim_end_matches(&ext));// format!("{dir}/{}/", SONGS_DIR, );
        }

        let mut archive = zip::ZipArchive::new(file).map_err(|e|e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let Some(outpath) = file.enclosed_name() else { continue };
            let outpath = dir.join(outpath);

            if file.name().ends_with('/') {
                debug!("File {i} extracted to \"{outpath:?}\"");
                std::fs::create_dir_all(&outpath).unwrap();
            } else {
                debug!("File {i} extracted to \"{outpath:?}\" ({} bytes)", file.size());
                if let Some(p) = outpath.parent() {
                    if !p.exists() { std::fs::create_dir_all(p).unwrap() }
                }
                let mut outfile = std::fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
        
        match delete_file {
            ArchiveDelete::Never => {},
            _ => {
                if let Err(e) = std::fs::remove_file(zip) {
                    error!("Error deleting file: {}", e)
                }
            }
        }

        Ok(dir.to_string_lossy().to_string())
    }
}

#[allow(unused)]
#[derive(Copy, Clone)]
pub enum ArchiveDelete {
    Always,
    OnSuccess,
    Never
}
