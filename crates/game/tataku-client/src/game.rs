use clap::Parser;
use tataku_game::prelude::*;
use tracing::*;

pub fn run_game(
    window_data: engine::window::WindowData,
    window_counters: engine::window::WindowCounters,
) {

    // gamemodes
    let gamemodes;
    #[cfg(feature="dynamic_gamemodes")] {
        gamemodes = vec![
            engine::gameplay::GamemodeLibrary::load_gamemode(
                "gamemodes/taiko"
            ).unwrap(),
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


    // database
    let database;
    #[cfg(feature="sqlite")] {
        database = Box::new(tataku_sqlite::Database::new());
    }

    // start the game
    trace!("creating game");
    let mut game = Game::new(
        window_data,
        window_counters,
        BuiltinMenus { 
            menus: tataku_resources::menus::ALL,
            dialogs: tataku_resources::dialogs::ALL,
        },
        vec![
            #[cfg(feature="kira_audio")] tataku_kira::KiraAudioInit, 
            #[cfg(feature="bass_audio")] tataku_bass::BassAudioInit,
        ],
        gamemodes,
        database,
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
