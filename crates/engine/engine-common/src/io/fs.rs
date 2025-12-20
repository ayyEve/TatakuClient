use crate::prelude::*;
use std::{ fs::File, path::Path };
use std::io::{ BufRead, BufReader, Lines };

/// read a file into bytes
pub fn read_file(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let time = Instant::now();
    let f = std::fs::read(&path);

    let duration = time.as_millis();
    if duration > 1000.0 { warn!("took {duration:.2}ms to load file bytes {}", path.as_ref().display()); } 
    // else { info!("took {duration:.2}ms to load file {}", path.as_ref().display()); }
    
    f
}

/// helper for the read_lines functions
fn open_file(path: impl AsRef<Path>) -> std::io::Result<File>{
    let time = Instant::now();
    let f = File::open(&path);

    let duration = time.as_millis();
    if duration > 1000.0 { 
        warn!("took {duration:.2}ms to load file {}", path.as_ref().display()); 
    }

    f
}


/// get a file's hash
pub fn get_file_hash(file_path: impl AsRef<Path>) -> tataku::Result<common::Md5Hash> {
    Ok(Cryptography::md5(read_file(file_path)?))
}

// check if file or folder exists
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
/// check if folder exists, creating it if it doesnt
pub fn check_folder(dir: &str) -> std::io::Result<()> {
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
        
        let bytes = ureq::get(download_url)
            .call().expect("error with request")
            .into_body()
            .read_to_vec().expect("error converting to bytes");

        std::fs::write(path, bytes)
            .expect("Error saving file");
    }
}

/// read a file to the end
pub fn read_lines(filename: impl AsRef<Path>) -> std::io::Result<Lines<BufReader<File>>> {
    let file = open_file(filename)?;
    Ok(BufReader::new(file).lines())
}

#[allow(clippy::lines_filter_map_ok)]
pub fn read_lines_resolved(filename: impl AsRef<Path>) -> std::io::Result<impl Iterator<Item = String>> {
    let file = open_file(filename)?;
    let lines = BufReader::new(file)
        .lines()
        .filter_map(core::result::Result::ok);
    Ok(lines)
}


/// opens a folder in the os' file explorer
#[allow(unused)]
#[cfg_attr(not(target_os="windows"), allow(clippy::needless_pass_by_value))]
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
