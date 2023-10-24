
#[derive(Clone, Debug)]
pub enum DownloadError {
    BadStatusCode(u16),
    /// contains the redirect url
    Redirected(String),
}
