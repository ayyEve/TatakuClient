
#[derive(Clone)]
#[allow(unused)]
pub enum NotificationOnClick {
    None,
    Url(String),
    Menu(String),

    File(String),
    Folder(String),
}
