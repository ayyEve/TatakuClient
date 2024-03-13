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
TEAL = color(0.0, 0.5, 0.5, 1.0);
RED = color(1.0, 0.0, 0.0, 1.0);
GREEN = color(0.0, 1.0, 0.0, 1.0);
BLUE = color(0.0, 0.0, 1.0, 1.0);



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
function text_list(list)
    return {
        list = list
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

function cond(cond, if_true, if_false) 
    return {
        id = "conditional",
        cond = cond,
        if_true = if_true,
        if_false = if_false
    }
end

function map_action(action)
    return {
        id = "action",
        action = {
            map = action
        }
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
start_singleplayer = { id = "action", action = { menu = "beatmap_select_menu" } }
start_multiplayer = { id = "action", action = { menu = "lobby_select_menu" } }
exit_game = { id = "action", action = { menu = "none" } }


-- menu stuff
menus = {}
menu_count = 0

function add_menu(menu) 
    menu_count = menu_count + 1
    menus[menu_count] = menu
end
