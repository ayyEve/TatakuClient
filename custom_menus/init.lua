-- colors
function color(r,g,b,a) 
    if not a then a = 1.0 end
    return {
        r = r,
        g = g,
        b = b,
        a = a
    }
end

WHITE = color(1.0, 1.0, 1.0, 1.0);



-- element helper fns
function row(config, elements) 
    if not elements then
        return {
            id = "row",
            elements = config,
        }
    else 
        config.id = "row"
        config.elements = elements
        return config
    end
end

function col(config, elements) 
    if not elements then
        return {
            id = "col",
            elements = config,
        }
    else 
        config.id = "col"
        config.elements = elements
        return config
    end
end

function space(width, height) 
    return {
        id = "space",
        width = width,
        height = height,
    }
end

-- helper for making a text object
function text(txt, font_size, color, font)
    return {
        id = "text",
        text = txt,
        font_size = font_size,
        color = color,
        font = font
    }
end

-- helper for making a button
function button(ele, action, width, height, padding)
    return {
        id = "button",
        element = ele,
        action = action,
        width = width,
        height = height,
        padding = padding
    }
end

function key_handler(events)
    return {
        id = "key_handler",
        events = events
    }
end


-- song_display = { id = "song_display" }
-- music_player = { id = "music_player" }

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
menu_count = 0

function add_menu(menu) 
    menu_count = menu_count + 1
    menus[menu_count] = menu
end
