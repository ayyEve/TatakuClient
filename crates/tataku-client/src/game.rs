use clap::Parser;
use tataku_game::prelude::*;

pub async fn run_game(
    game_event_receiver: tokio::sync::mpsc::Receiver<WindowEvent>,
    proxy: winit::event_loop::EventLoopProxy<WindowAction>,
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
        ]
    }

    // start the game
    trace!("creating game");
    let mut game = Game::new(
        game_event_receiver,
        proxy,
        vec![
            #[cfg(feature="kira_audio")] tataku_kira::KiraAudioInit, 
            #[cfg(feature="bass_audio")] tataku_bass::BassAudioInit,
        ],
        gamemodes,
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
