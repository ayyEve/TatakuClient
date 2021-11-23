use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};

use crate::{Vector2, render::Image};
use crate::game::{Settings, managers::NotificationManager};


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
        println!("Check failed for '{:?}', downloading from '{}'", path, download_url);
        
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


/// read a file to the end
pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

/// get a file's hash
pub fn get_file_hash<P:AsRef<Path>>(file_path:P) -> std::io::Result<String> {
    let body = std::fs::read(file_path)?;
    Ok(format!("{:x}", md5::compute(body).to_owned()))
}

// check if file or folder exists
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}


/// load an image file to an image struct
pub fn load_image<T:AsRef<str>>(path: T) -> Option<Image> {
    let settings = opengl_graphics::TextureSettings::new();
    // helper.log("settings made", true);

    let buf: Vec<u8> = match std::fs::read(path.as_ref()) {
        Ok(buf) => buf,
        Err(_) => return None,
    };

    match image::load_from_memory(&buf) {
        Ok(img) => {
            let img = img.into_rgba8();
            let tex = opengl_graphics::Texture::from_image(&img, &settings);
            Some(Image::new(Vector2::zero(), f64::MAX, tex, Settings::window_size()))
        }
        Err(e) => {
            NotificationManager::add_error_notification(&format!("Error loading wallpaper: {}", path.as_ref()), e);
            // println!("Error loading image {}: {}", path.as_ref(), e);
            None
        }
    }
}
