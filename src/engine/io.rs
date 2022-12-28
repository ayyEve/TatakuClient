use crate::prelude::*;
use std::fs::DirEntry;
use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};


/// check if folder exists, creating it if it doesnt
pub fn check_folder(dir:&str) {
    if !Path::new(dir).exists() {
        std::fs::create_dir(dir).expect("error creating folder: ");
    }
}

/// check if a file exists, downloading it if it doesnt
pub async fn check_file<P:AsRef<Path>>(path:P, download_url:&str) {
    let path = path.as_ref();
    if !path.exists() {
        info!("Check failed for '{:?}', downloading from '{}'", path, download_url);
        
        let bytes = reqwest::get(download_url)
            .await
            .expect("error with request")
            .bytes()
            .await
            .expect("error converting to bytes");

        std::fs::write(path, bytes)
            .expect("Error saving file");
    }
}


pub fn sanitize_filename(filename: impl AsRef<str>) -> String {
    filename.as_ref()
        .replace("\\", "") 
        .replace("/", "") 
        .replace(":", "")  
        .replace("*", "") 
        .replace("?", "") 
        .replace("\"", "") 
        .replace("'", "") 
        .replace("<", "") 
        .replace(">", "") 
        .replace("|", "") 
}


/// read a file to the end
pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}
#[allow(unused)]
pub fn read_lines_resolved<P: AsRef<Path>>(filename: P) -> io::Result<impl Iterator<Item = String>> {
    let file = File::open(filename)?;
    let lines = BufReader::new(file).lines().filter_map(|f|f.ok());
    Ok(lines)
}

/// get a file's hash
pub fn get_file_hash<P:AsRef<Path>>(file_path:P) -> std::io::Result<String> {
    Ok(md5(std::fs::read(file_path)?))
}

// check if file or folder exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}


/// load an image file to an image struct
pub async fn load_image<T:AsRef<str>>(path: T, use_grayscale: bool) -> Option<Image> {
    // helper.log("settings made", true);

    let buf: Vec<u8> = match std::fs::read(path.as_ref()) {
        Ok(buf) => buf,
        Err(_) => return None,
    };

    match image::load_from_memory(&buf) {
        Ok(img) => {
            let mut img = img.into_rgba8();

            if use_grayscale {
                for i in img.pixels_mut() {
                    let [r, g, b, _a] = &mut i.0;

                    let rf = *r as f32 / 255.0;
                    let gf = *g as f32 / 255.0;
                    let bf = *b as f32 / 255.0;

                    let gray = 0.299 * rf + 0.587 * gf + 0.114 * bf;

                    *r = (gray * 255.0) as u8;
                    *g = (gray * 255.0) as u8;
                    *b = (gray * 255.0) as u8;
                }
            }

            let tex = load_texture_data(img).await.ok()?;
            let img = Some(Image::new(Vector2::ZERO, f64::MAX, tex, Vector2::ONE));
            img
        }
        Err(e) => {
            NotificationManager::add_error_notification(&format!("Error loading wallpaper: {}", path.as_ref()), e).await;
            // error!("Error loading image {}: {}", path.as_ref(), e);
            None
        }
    }
}

/// download a file from `url` to `download_path`
pub async fn _download_file(url: impl reqwest::IntoUrl, download_path: impl AsRef<Path>) -> TatakuResult<()> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    
    // check if the received data 
    if bytes.len() == 0 {
        return Err(TatakuError::String("Downloaded file was empty".to_owned()));
    }

    std::fs::write(download_path, bytes)?;


    Ok(())
}


pub async fn extract_all() {

    // check for new maps
    if let Ok(files) = std::fs::read_dir(crate::DOWNLOADS_DIR) {
        // let completed = Arc::new(Mutex::new(0));

        let files:Vec<std::io::Result<DirEntry>> = files.collect();
        // let len = files.len();
        trace!("Files: {:?}", files);

        for file in files {
            trace!("Looping file {:?}", file);
            // let completed = completed.clone();

            match file {
                Ok(filename) => {
                    trace!("File ok");
                    // tokio::spawn(async move {
                        trace!("Reading file {:?}", filename);

                        let mut error_counter = 0;
                        // unzip file into ./Songs
                        while let Err(e) = std::fs::File::open(filename.path().to_str().unwrap()) {
                            error!("Error opening osz file: {}", e);
                            error_counter += 1;

                            // if we've waited 5 seconds and its still broken
                            if error_counter > 5 {
                                error!("5 errors opening osz file: {}", e);
                                return;
                            }

                            // tokio::time::sleep(Duration::from_millis(1000)).await;
                        }

                        let file = std::fs::File::open(filename.path().to_str().unwrap()).unwrap();
                        let mut archive = match zip::ZipArchive::new(file) {
                            Ok(a) => a,
                            Err(e) => {
                                error!("Error extracting zip archive: {}", e);
                                NotificationManager::add_text_notification("Error extracting file\nSee console for details", 3000.0, Color::RED).await;
                                continue;
                            }
                        };
                        
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).unwrap();
                            let mut outpath = match file.enclosed_name() {
                                Some(path) => path,
                                None => continue,
                            };

                            let x = outpath.to_str().unwrap();
                            let y = format!("{}/{}/", SONGS_DIR, filename.file_name().to_str().unwrap().trim_end_matches(".osz"));
                            let z = &(y + x);
                            outpath = Path::new(z);

                            if (&*file.name()).ends_with('/') {
                                debug!("File {} extracted to \"{}\"", i, outpath.display());
                                std::fs::create_dir_all(&outpath).unwrap();
                            } else {
                                debug!("File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
                                if let Some(p) = outpath.parent() {
                                    if !p.exists() {std::fs::create_dir_all(&p).unwrap()}
                                }
                                let mut outfile = std::fs::File::create(&outpath).unwrap();
                                std::io::copy(&mut file, &mut outfile).unwrap();
                            }

                            // Get and Set permissions
                            // #[cfg(unix)] {
                            //     use std::os::unix::fs::PermissionsExt;
                            //     if let Some(mode) = file.unix_mode() {
                            //         fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                            //     }
                            // }
                        }
                    
                        match std::fs::remove_file(filename.path().to_str().unwrap()) {
                            Ok(_) => {},
                            Err(e) => error!("Error deleting file: {}", e),
                        }
                        
                        trace!("Done");
                        // *completed.lock() += 1;
                    // });
                }
                Err(e) => {
                    error!("Error with file: {}", e);
                }
            }
        }
    
        
        // while *completed.lock() < len {
        //     debug!("waiting for downloads {} of {}", *completed.lock(), len);
        //     std::thread::sleep(Duration::from_millis(500));
        // }
    }
}



pub async fn read_other_game_replay(path: impl AsRef<Path>) -> TatakuResult<Replay> {
    let path = path.as_ref();

    match path.extension().and_then(|s|s.to_str()) {

        // tataku replay
        Some("ttkr") => Ok(open_database(path.to_str().unwrap())?.read::<Replay>()?),

        // osu replay
        Some("osr") => Ok(convert_osu_replay(path)?),

        _ => Err(TatakuError::String("Unknown replay file".to_owned()))
    }
}



/// opens a folder in the os' file explorer
pub fn open_folder(path: String, selected_file: Option<String>) {
    #[cfg(windows)] {
        let mut cmd = &mut std::process::Command::new("explorer.exe");
        let path = path.replace("/", "\\");
        
        if let Some(selected_file) = selected_file {
            let arg = format!("/select,{path}\\{selected_file}");
            trace!("open folder: {arg}");
            cmd = cmd.arg(arg)
        } else {
            cmd = cmd.arg(path)
        }

        if let Err(e) = cmd.spawn() {
            error!("error running cmd: {e}")
        }

        // explorer.exe /select,"C:\Folder\subfolder\file.txt"
    }
}

pub fn open_link(url: String) {
    #[cfg(windows)] {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(url);

        if let Err(e) = cmd.spawn() {
            error!("error running cmd: {e}")
        }
    }
}





#[derive(Clone)]
pub struct AsyncLoader<T> {
    value: Arc<RwLock<Option<T>>>
}
impl<T:Send + Sync + 'static> AsyncLoader<T> {
    pub fn new<F: std::future::IntoFuture<Output = T> + Send + 'static>(f: F) -> Self where <F as std::future::IntoFuture>::IntoFuture: Send {
        let val = Arc::new(RwLock::new(None));
        let value = val.clone();
        
        tokio::spawn(async move {
            let v = f.into_future().await;
            *val.write().await = Some(v);
        });

        Self {
            value
        }
    }

    pub async fn check(&self) -> Option<T> {
        if self.value.read().await.is_some() {
            Some(std::mem::take(&mut *self.value.write().await).unwrap())
        } else {
            None
        }
    }
}