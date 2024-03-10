-- colors
WHITE = { r = 1.0, g = 1.0, b = 1.0, a = 1.0 }



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

function space(width, height) 
    return {
        id = "space",
        width = width,
        height = height,
    }
end

function key_handler(events)
    return {
        id = "key_handler",
        events = events
    }
end

-- song_display = { id = "song_display" }
music_player = { id = "music_player" }

song_display = {
    id = "row",
    width = "fill",
    height = "shrink",
    padding = 5.0,
    elements = {
        --[[ align-right ]] space("fill", "shrink"),

        -- {artist} - {title}
        {
            id = "styled_content",
            color = { r = 1.0, g = 1.0, b = 1.0, a = 0.1 },
            shape = { round = 5.0 },
            padding = 8.0,

            element = {
                id = "text",
                text = { list = {{ value = "map.artist" }, " - ", { value = "map.title" }} },
                color = WHITE, 
                size = 30
            }
        }
    }
}

-- premade button actions
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

