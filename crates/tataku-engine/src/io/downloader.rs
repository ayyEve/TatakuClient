use crate::prelude::*;

use tokio::{ net::TcpStream, io::{AsyncReadExt, AsyncWriteExt} };

pub struct Downloader;
impl Downloader {
    pub fn download(options: DownloadOptions) -> Arc<RwLock<DownloadProgress>> {
        let progress = Arc::new(RwLock::new(DownloadProgress::default()));
        Self::download_existing_progress(options, progress.clone());

        progress
    }

    pub fn download_existing_progress(mut options: DownloadOptions, progress: Arc<RwLock<DownloadProgress>>) {
        tokio::spawn(async move {
            for i in 0..=options.retry_count {
                match Self::perform_download(&options, &progress).await {
                    Ok(_) => break,

                    // redirected
                    Err(TatakuError::DownloadError(DownloadError::Redirected(new_url))) => {
                        // download using the new url
                        warn!("Redirected: {new_url}");
                        options.url = new_url;
                        return Self::download_existing_progress(options, progress);
                    }

                    // other error
                    Err(e) => {
                        let mut p = progress.write();
                        p.error = Some(e);
                        p.retrying = i != options.retry_count;

                        p.downloaded = 0;
                        p.size = 0;
                    }
                }
            }
        });
    }


    async fn perform_download(options: &DownloadOptions, progress: &Arc<RwLock<DownloadProgress>>) -> TatakuResult {
        let params = UrlParams::parse(&options.url).unwrap();
        debug!("got params: {params:?}");

        let conn = TcpStream::connect(format!("{}:{}", params.host, params.port)).await?;
        let mut conn = if params.is_https {
            // tls wrapper
            let cx = native_tls::TlsConnector::builder().build().unwrap();
            let cx = tokio_native_tls::TlsConnector::from(cx);
            let conn = cx.connect(&params.host, conn).await.unwrap();
            TcpConnection::Ssl(conn)
        } else {
            TcpConnection::NonSsl(conn)
        };

        // send request
        let request = format!("GET /{} HTTP/1.0\r\nHost: {}\r\n\r\n", params.path, params.host);
        conn.write(request.as_bytes()).await?;
        
        let mut bytes = Vec::new();
        let mut got_headers = false;

        let mut buf = [0; 1024];
        loop {
            let read = conn.read(&mut buf).await?;
            if read == 0 { 
                let mut progress = progress.write();
                if progress.downloaded == progress.size {
                    progress.data = Some(bytes);
                    progress.error = None;
                    progress.retrying = false;
                }

                return Ok(())
            }
    
            bytes.extend(&buf[..read]);
            
            if !got_headers {
                let str = String::from_utf8_lossy(&bytes).to_string();
                if str.contains("\r\n\r\n") {
                    got_headers = true;

                    let mut split = str.split("\r\n\r\n");
                    let headers = split.next().unwrap();
                    let len = headers.len() + 4;

                    // remove the headers stuff from the bytes
                    bytes = bytes[len..].to_vec();

                    // set how much we've downloaded
                    progress.write().downloaded = bytes.len();

                    // parse headers
                    let mut headers_split = headers.split("\r\n");

                    // first header is response code
                    let response_code = headers_split.next().unwrap();
                    let mut response_code_split = response_code.split(" ");
                    let _ = response_code_split.next(); // HTTP/1.1
                    let code = response_code_split.next().and_then(|c|c.parse::<u16>().ok()).unwrap();
                
                    match code {
                        // success
                        200 => {}

                        // redirect
                        302 => {}

                        code => {
                            let code_text = response_code_split.collect::<Vec<_>>().join(" ");
                            println!("Bad status code: {code} ({code_text:?})");
                            return Err(TatakuError::DownloadError(DownloadError::BadStatusCode(code)));
                        }
                    }

                    for header in headers_split {
                        let mut split = header.split(":").map(|s|s.trim());
                        let key = split.next().unwrap().to_lowercase();
                        let value = split.next().unwrap_or("").to_string() + &split.collect::<Vec<_>>().join(":");

                        if key == "content-length" {
                            let length = value.parse::<usize>().ok().unwrap_or_default();
                            progress.write().size = length;
                        }
                        if code == 302 && key == "location" {
                            // location is the redirect url, try downloading from there.
                            let location = value.to_owned();
                            return Err(TatakuError::DownloadError(DownloadError::Redirected(location)));
                        }
                    }
                }
            } else {
                progress.write().downloaded += read;
            }

        }

    }
}

pub struct DownloadOptions {
    pub url: String,
    pub retry_count: usize,
}
impl DownloadOptions {
    pub fn new(url: String, retry_count: usize) -> Self {
        Self {
            url,
            retry_count
        }
    }
}

#[derive(Default)]
pub struct DownloadProgress {
    /// size of the download in bytes
    pub size: usize,
    /// how many bytes have been downloaded
    pub downloaded: usize,

    /// was there an error?
    pub error: Option<TatakuError>,
    /// if there was an error, are we retrying?
    /// if so, the watcher should not give up
    pub retrying: bool,

    /// the data if the download was successful
    pub data: Option<Vec<u8>>
}
impl DownloadProgress {
    pub fn failed(&self) -> bool { self.error.is_some() && !self.retrying }
    pub fn complete(&self) -> bool { self.data.is_some() }

    pub fn progress(&self) -> f32 {
        if self.size == 0 { return 0.0 }

        self.downloaded as f32 / self.size as f32
    }
}


#[derive(Default, Debug)]
struct UrlParams {
    port: u16,
    host: String,
    path: String,
    is_https: bool,
}
impl UrlParams {
    pub fn parse(url: impl ToString) -> Option<Self> {
        // TODO: use a regex?
        let url = url.to_string();

        let mut s = Self {
            port: 80,
            ..Default::default()
        };
        

        let url = url.replace("//", "/");
        let mut split = url.split("/");

        // protocol (http/https)
        let protocol = split.next()?;
        if protocol.to_lowercase() == "https:" {
            s.port = 443; 
            s.is_https = true;
        }

        // host
        let host = split.next()?;
        let mut host_split = host.split(":");
        let host = host_split.next()?;
        s.host = host.to_string();

        // port
        if let Some(port) = host_split.next() {
            s.port = port.parse().ok()?;
        }

        // path
        let path = split.collect::<Vec<_>>().join("/");
        s.path = path;

        // return
        Some(s)
    }

}


enum TcpConnection {
    Ssl(tokio_native_tls::TlsStream<tokio::net::TcpStream>),
    NonSsl(tokio::net::TcpStream)
}
impl TcpConnection {
    async fn read(&mut self, buf: &mut [u8]) -> TatakuResult<usize> {
        match self {
            Self::Ssl(stream) => Ok(stream.read(buf).await?),
            Self::NonSsl(stream) => Ok(stream.read(buf).await?),
        }
    }
    async fn write(&mut self, buf: &[u8]) -> TatakuResult {
        match self {
            Self::Ssl(stream) => stream.write_all(buf).await?,
            Self::NonSsl(stream) => stream.write_all(buf).await?,
        }
        Ok(())
    }
}


#[tokio::test]
async fn test() -> TatakuResult {
    let file = "eveflatshading.png1";
    let url = format!("https://cdn.ayyeve.xyz/{file}");
    println!("downloading {file} from url {url}");

    let options = DownloadOptions {
        url,
        retry_count: 5
    };

    let progress = Downloader::download(options);

    let mut last_progress = 0.0;
    loop {
        tokio::task::yield_now().await;
        let progress = progress.read();
        // progress printing
        {
            let p = progress.progress() * 100.0;
            if p - last_progress > 1.0 {
                last_progress = p;
                println!("{p}% ({}/{})", progress.downloaded, progress.size);
            }
        }

        if progress.complete() {
            let bytes = progress.data.as_ref().unwrap();
            std::fs::write(format!("debug/{file}"), bytes).unwrap();
            break;
        }

        if progress.failed() {
            if let Some(e) = &progress.error {
                println!("failed: {e}");
            }
            break;
        }
    }
    
    Ok(())
}
