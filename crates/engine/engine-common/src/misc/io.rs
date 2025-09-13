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

    /// helper for the read_lines functions
    fn open_file(path: impl AsRef<Path>) -> io::Result<File>{
        let time = Instant::now();
        let f = File::open(&path);

        let duration = time.as_millis();
        if duration > 1000.0 { 
            warn!("took {duration:.2}ms to load file {}", path.as_ref().display()); 
        }

        f
    }

    
    /// get a file's hash
    pub fn get_file_hash<P:AsRef<Path>>(file_path:P) -> TatakuResult<common::Md5Hash> {
        Ok(Cryptography::md5(Self::read_file(file_path)?))
    }

    // check if file or folder exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
    /// check if folder exists, creating it if it doesnt
    pub fn check_folder(dir: &str) -> io::Result<()> {
        if !Path::new(dir).exists() {
            std::fs::create_dir(dir)?;
        }
        Ok(())
    }

    /// check if a file exists, downloading it if it doesnt
    pub fn check_file_sync(path: impl AsRef<Path>, download_url: &str) {
        let path = path.as_ref();
        if !path.exists() {
            info!("Check failed for '{path:?}', downloading from '{download_url}'");
            
            let bytes = reqwest::blocking::get(download_url)
                .expect("error with request")
                .bytes()
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

    #[allow(clippy::lines_filter_map_ok)]
    pub fn read_lines_resolved(filename: impl AsRef<Path>) -> io::Result<impl Iterator<Item = String>> {
        let file = Self::open_file(filename)?;
        let lines = BufReader::new(file)
            .lines()
            .filter_map(core::result::Result::ok);
        Ok(lines)
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
            cmd = cmd.arg(arg);
        } else {
            cmd = cmd.arg(path);
        }

        if let Err(e) = cmd.spawn() {
            error!("error running cmd: {e}");
        }

        // explorer.exe /select,"C:\Folder\subfolder\file.txt"
    }

    #[cfg(target_os="linux")] {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(path);
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}"); }
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
        if let Err(e) = cmd.spawn() { error!("error running cmd: {e}"); }
    }
}
