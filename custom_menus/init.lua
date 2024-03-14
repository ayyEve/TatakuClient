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
function key_event(key, mods, action) 
    if not action then 
        action = mods
        mods = nil
    end

    return {
        key = key,
        action = action,
        mods = mods,
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


--- action helpers
function map_action(action)
    return {
        id = "action",
        map = action
    }
end
function song_action(action)
    return {
        id = "action",
        song = action
    }
end
function game_action(action)
    return {
        id = "action",
        game = action
    }
end

function custom_action(tag, val)
    if not val then val = { value = "" } end
    val.id = "custom"
    val.tag = tag
    return val
end

function menu_action(menu) 
    return {
        id = "action",
        menu = menu
    }
end
function dialog_action(dialog)
    return {
        id = "action",
        dialog = dialog
    }
end



function variable(var) 
    return {
        variable = var
    }
end

-- a simple element which displays the current map info
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
            color = color(1.0, 1.0, 1.0, 0.1),
            shape = { round = 5.0 },
            padding = 8.0,

            element = text(
                text_list({ variable("map.artist"), " - ", variable("map.title") }),
                30,
                WHITE
            )
        }
    }
}

-- premade button actions
start_singleplayer = menu_action("beatmap_select")
start_multiplayer = menu_action("lobby_select")
exit_game = game_action("quit") -- TODO: this properly


-- menu stuff
menus = {}
menu_count = 0

function add_menu(menu) 
    menu_count = menu_count + 1
    menus[menu_count] = menu
end
