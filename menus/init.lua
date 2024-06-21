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

function key_event(key, mods, action) 
    if not action then 
        action = mods
        mods = nil
    end

    return { 
        event = { key_press = { key = key, mods = mods } },
        action = action
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
function map_action(action, data)
    if not data then 
        return {
            id = "action",
            map = action,
        }
    else
        data.id = action
        return {
            id = "action",
            map = data
        }
    end
end
function song_action(action)
    return {
        id = "action",
        song = action
    }
end
function game_action(action, data)
    if not data then 
        return {
            id = "action",
            game = action
        }
    else
        data.id = action
        return {
            id = "action",
            game = data
        }
    end
end
function multiplayer_action(action, slot)
    if slot then 
        return {
            id = "action",
            multiplayer = {
                id = action,
                slot = slot
            }
        }
    else
        return {
            id = "action",
            multiplayer = action,
        }
    end

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
function no_action()
    return {
        id = "none",
    }
end


function variable(var) 
    return {
        variable = var
    }
end

function calc(var) 
    return {
        calc = var
    }
end
function passed_in() 
    return {
        passed_in = true
    }
end

-- menu stuff
-- menus = {}
-- menu_count = 0
new_menu = nil

function add_menu(menu) 
    -- menu_count = menu_count + 1
    -- menus[menu_count] = menu
    new_menu = menu
    print("added menu")
end
