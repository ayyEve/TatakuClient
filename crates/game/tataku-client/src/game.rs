use clap::Parser;
use tataku_game::prelude::*;
use tataku::Vector2;
use tracing::*;

pub fn run_game(
    game_event_receiver: tokio::sync::mpsc::Receiver<engine::window::Event>,
    mouse_position_receiver: engine::triple_buffer::Output<Vector2>,
    proxy: winit::event_loop::EventLoopProxy<engine::actions::window::WindowAction>,
) {
    let gamemodes;
    #[cfg(feature="dynamic_gamemodes")] {
        gamemodes = vec![
            GamemodeLibrary::load_gamemode("/home/ayyeve/Desktop/projects/tataku/tataku-client/target/release/gamemode_taiko").unwrap(),
        ];
    }

    #[cfg(not(feature="dynamic_gamemodes"))] {
        gamemodes = vec![
            gamemode_osu::GAME_INFO,
            gamemode_taiko::GAME_INFO,
            gamemode_mania::GAME_INFO,
            gamemode_utyping::GAME_INFO,
        ];
    }


    // start the game
    trace!("creating game");
    let mut game = Game::new(
        game_event_receiver,
        mouse_position_receiver,
        proxy,
        vec![
            #[cfg(feature="kira_audio")] tataku_kira::KiraAudioInit, 
            #[cfg(feature="bass_audio")] tataku_bass::BassAudioInit,
        ],
        gamemodes,
        BuiltinMenus { 
            menus: tataku_resources::menus::ALL, 
            dialogs: tataku_resources::dialogs::ALL, 
        }
    );


    let args = GameArgs::parse();
    if let Some(test_path) = args.xml_test {
        game.make_xml_helper(test_path);
    }

    
    trace!("running game");
    game.game_loop();
    warn!("game closed");
}


#[derive(clap::Parser)]
struct GameArgs {
    #[arg(long="xml-test")]
    xml_test: Option<String>,
}
