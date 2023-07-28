use crate::prelude::*;

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum NotificationOnClick {
    None,
    Url(String),
    Menu(String),

    File(String),
    Folder(String),
    MultiplayerLobby(u32)
}
impl NotificationOnClick {
    pub async fn do_action(&self, game: &mut Game) {
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
                tokio::spawn(OnlineManager::join_lobby(*lobby_id, String::new()));
                let menu = LobbySelect::new().await;
                game.queue_state_change(GameState::InMenu(Box::new(menu)));
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