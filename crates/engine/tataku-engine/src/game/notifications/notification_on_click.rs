use crate::prelude::*;

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub enum NotificationOnClick {
    #[default] None,
    Url(String),
    Menu(String),

    File(String),
    Folder(String),
    MultiplayerLobby(u32)
}
impl NotificationOnClick {
    pub fn do_action(&self, actions: &mut ActionQueue) {
        match self {
            NotificationOnClick::None => {}
            NotificationOnClick::Url(url) => {
                debug!("open url {url}");
                open_link(url.clone());
            }
            NotificationOnClick::Menu(menu_name) => {
                debug!("goto menu {menu_name}");
            }

            NotificationOnClick::MultiplayerLobby(lobby_id) => {
                debug!("join lobby {lobby_id}");
                actions.push(MultiplayerAction::JoinLobby {
                    lobby_id: *lobby_id,
                    password: String::new(),
                });
            }

            NotificationOnClick::File(file_path) => {
                let path = Path::new(file_path);
                let folder = path.parent().unwrap().to_string_lossy().to_string();
                let file = path.file_name().unwrap().to_string_lossy().to_string();

                open_folder(folder, Some(file));
            }
            NotificationOnClick::Folder(folder) => {
                open_folder(folder.clone(), None);
            }
        }
    }
}
