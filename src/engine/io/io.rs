use crate::prelude::*;
use std::{ fs::File, path::Path };
use std::io::{ self, BufRead, BufReader, Lines };


pub struct Io;
impl Io {

    /// read a file into bytes
    pub fn read_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let time = Instant::now();
        let f = std::fs::read(&path);

        let duration = time.as_millis();
        if duration > 1000.0 { warn!("took {duration:.2}ms to load file bytes {}", path.as_ref().display()); } 
        // else { info!("took {duration:.2}ms to load file {}", path.as_ref().display()); }
        
        f
    }
    /// read a file into bytes
    pub async fn read_file_async(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let time = Instant::now();
        let f = tokio::fs::read(&path).await;

        let duration = time.as_millis();
        if duration > 1000.0 { warn!("took {duration:.2}ms to load file bytes {}", path.as_ref().display()); } 
        // else { info!("took {duration:.2}ms to load file bytes {}", path.as_ref().display()); }
        
        f
    }

    /// helper for the read_lines functions
    fn open_file(path: impl AsRef<Path>) -> io::Result<File>{
        let time = Instant::now();
        let f = File::open(&path);

        let duration = time.as_millis();
        if duration > 1000.0 { warn!("took {duration:.2}ms to load file {}", path.as_ref().display()); }

        f
    }

    
    /// get a file's hash
    pub fn get_file_hash<P:AsRef<Path>>(file_path:P) -> TatakuResult<Md5Hash> {
        Ok(md5(Self::read_file(file_path)?))
    }

    // pub fn get_file_with_hash(path: impl AsRef<Path>) -> TatakuResult<(String, Vec<u8>)> {
    //     let bytes = Self::read_file(path)?;
    //     let hash = md5(&bytes);
    //     Ok((hash, bytes))
    // }

    // check if file or folder exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
    /// check if folder exists, creating it if it doesnt
    pub fn check_folder(dir:&str) -> io::Result<()> {
        if !Path::new(dir).exists() {
            std::fs::create_dir(dir)?;
        }
        Ok(())
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
    pub fn read_lines(filename: impl AsRef<Path>) -> io::Result<Lines<BufReader<File>>> {
        let file = Self::open_file(filename)?;
        Ok(BufReader::new(file).lines())
    }

    pub fn read_lines_resolved(filename: impl AsRef<Path>) -> io::Result<impl Iterator<Item = String>> {
        let file = Self::open_file(filename)?;
        let lines = BufReader::new(file).lines().filter_map(|f|f.ok());
        Ok(lines)
    }

}



/// load an image file to an image struct
/// non-main thread safe
pub async fn load_image(path: impl AsRef<str> + Send + Sync, use_grayscale: bool, base_scale: Vector2) -> Option<Image> {
    let path2 = path.as_ref().to_owned();

    let Ok(buf) = Io::read_file_async(&path2).await else { return None };

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

            let tex = GameWindow::load_texture_data(img).await.expect("no atlas");
            Some(Image::new(Vector2::ZERO, tex, base_scale))
        }
        Err(e) => {
            NotificationManager::add_error_notification(format!("Error loading image: {}", path.as_ref()), e).await;
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


pub async fn read_replay_path(path: impl AsRef<Path>) -> TatakuResult<Replay> {
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
#[allow(unused)]
pub fn open_folder(path: String, selected_file: Option<String>) {
    #[cfg(target_os="windows")] {
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

    #[cfg(target_os="linux")] {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(path);
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}") }
    }
}   

pub fn open_link(url: String) {
    info!("Opening link '{url}'");
    #[cfg(target_os="windows")] {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(url);
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}") }
    }

    #[cfg(target_os="linux")] {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(url);
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}") }
    }

    #[cfg(target_os="macos")] {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(url);
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}") }
    }
}





#[derive(Clone)]
pub struct AsyncLoader<T> {
    value: Arc<AsyncMutex<Option<T>>>,
    written: Arc<AtomicBool>,
    abort_handle: Arc<tokio::task::AbortHandle>,
}
impl<T:Send + Sync + 'static> AsyncLoader<T> {
    pub fn new<F: std::future::IntoFuture<Output = T> + Send + 'static>(f: F) -> Self where <F as std::future::IntoFuture>::IntoFuture: Send {
        let value = Arc::new(AsyncMutex::new(None));
        let written = Arc::new(AtomicBool::new(false));
        
        let val = value.clone();
        let wrote = written.clone();
        let task = tokio::spawn(async move {
            let v = f.into_future().await;
            *val.lock().await = Some(v);
            wrote.store(true, Ordering::Release)
        });
        let abort_handle = Arc::new(task.abort_handle());

        Self {
            value,
            written,
            abort_handle,
        }
    }

    pub fn abort(&self) {
        self.abort_handle.abort();
    }

    pub fn is_complete(&self) -> bool {
        self.written.load(Ordering::Acquire)
    }

    pub async fn check(&self) -> Option<T> {
        if self.written.load(Ordering::Acquire) {
            std::mem::take(&mut *self.value.lock().await)
        } else {
            None
        }
    }
}
