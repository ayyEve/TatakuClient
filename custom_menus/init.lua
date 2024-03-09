
-- element helper fns
function row(elements) 
    return {
        id = "row",
        elements = elements,
    }
end

function col(elements) 
    return {
        id = "column",
        elements = elements,
    }
end

song_display = { id = "song_display" }
music_player = { id = "music_player" }

-- button actions
start_singleplayer = { menu = "beatmap_select_menu", context = { "main_menu", "singleplayer" } }
start_multiplayer = { menu = "lobby_select_menu", context = { "main_menu" } }
exit_game = { menu = "none", context = { "main_menu", "quit_game" } }


-- menu stuff
menus = {}
menu_count = 0;

function add_menu(menu) 
    menu_count = menu_count + 1
    menus[menu_count] = menu
end