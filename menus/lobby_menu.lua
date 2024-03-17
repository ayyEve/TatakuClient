local font_size = 40.0
local padding = 5.0;

-- fixed size to keep smaller icon glyphs inline
local size = "fixed(" .. (font_size + padding * 2.0) .. ")"

local ready_button = {
    id = "button",
    debug_name = "Lobby Ready Button",

    action = cond(
        "lobby.has_beatmap",
        cond(
            "lobby.our_player.ready",
            multiplayer_action("unready"), -- if we're ready, we want the action to be unready
            multiplayer_action("ready") -- otherwise, we want 
        )
        -- if we dont have the map, we dont want to have an action here
    ),

    element = cond(
        "lobby.has_beatmap && lobby.our_player.ready",
        text("Un-Ready"), -- if we have the map, and we're already read, show "unready"
        text("Ready") -- otherwise, show "ready"
    ),
}

local function beatmap_info_text(has_map, map_exists, line) 
    local text = {}

    -- determine which lines to use
    if has_map then
        text = { 
            variable("map.artist"), " - ", variable("map.title"), "\n", 
            variable("map.version"), " // ", variable("map.creator"), "\n",
            variable("map.diff_info")
        }
    elseif map_exists then
        text = { variable("lobby.map.title"), " (", variable("lobby.map.game"), ")" }
    else
        text = { "No beatmap" }
    end

    -- append newline and line if provided
    if line then 
        table.insert(text, "\n")
        table.insert(text, line)
    end

    return {
        id = "text",
        width = "fill",
        height = "fill_portion(2)",

        text = text_list(text)
    }
end

local beatmap_info_button = {
    id = "button",
    debug_name = "beatmap_info",
    width = "fill",
    height = "fill",

    action = cond(
        "lobby.is_host",
        menu_action("beatmap_select"), -- if we're the host, always override the button with opening the beatmap select menu
        cond( -- if we're not the host..
            "!lobby.has_beatmap", -- and we don't have the beatmap
            multiplayer_action("open_map_link") -- open a link to the beatmap
            
            -- otherwise, there is no action to perform
        )
    ),
    element = cond(
        "lobby.is_host",
        cond( -- if host
            "lobby.has_beatmap",
            beatmap_info_text(true, true, "Click here to change the beatmap"),
            cond( -- if we dont have the beatmap
                "lobby.map.exists",
                beatmap_info_text(false, true, "Click here to change the beatmap"),
                beatmap_info_text(false, false, "Click here to change the beatmap")
            )
        ),
        cond( -- if not host
            "lobby.has_beatmap",
            beatmap_info_text(true, true),
            cond( -- if we dont have the beatmap
                "lobby.map.exists",
                beatmap_info_text(false, true, "Click here to open beatmap download page"),
                beatmap_info_text(false, false)
            )
        )
    )
}

-- slot icon helper
local function icon_button(icon)
    return {
        id = "text",
        text = icon,
        font_size = font_size, 
        color = WHITE, 
        font = "fa",
        padding = padding,

        width = size,
        height = size
    }
end


local menu = {
    id = "lobby_menu",

    element = row({ width = "fill", height = "fill", spacing = 5.0 }, { 

        -- slot list
        col({ width = "fill", height = "fill", spacing = 5.0 }, {
            {
                id = "list",
                debug_name = "slot list",
                width = "fill",
                height = "shrink",
                
                list = "lobby.slots",
                variable = "_slot",
                scroll = true;

                element = row({ width = "fill", height = "shrink", spacing = 5.0 }, {
                    -- icon
                    button(
                        -- element
                        cond(
                            "_slot.is_host",
                            icon_button(0xF521), -- crown
                            cond(
                                "_slot.empty || _slot.filled",
                                icon_button(0xF13E), -- lock
                                icon_button(0xF023) -- unlock
                            )
                        ),

                        -- action
                        cond(
                            "lobby.is_host", -- if we're host...
                            cond(
                                "_slot.filled", -- and the slot is filled...
                                cond(
                                    "!_slot.is_host", -- and the slot isnt us...
                                    multiplayer_action("kick_slot", variable("_slot.id")) -- kick
                                ),
                                -- if the slot isnt filled...
                                cond(
                                    "_slot.locked", -- and the slot is locked...
                                    multiplayer_action("unlock_slot", variable("_slot.id")), -- unlock it
                                    multiplayer_action("lock_slot", variable("_slot.id")) -- otherwise, lock it
                                )
                            ),
                            -- if we're not the host, don't perform any action
                            no_action()
                        ),
                        
                        "shrink",
                        "shrink",
                        padding
                    ),

                    -- slot state
                    button(
                        cond(
                            "_slot.filled", -- if the slot has someone, show their username and status
                            {
                                id = "text",
                                text = text_list({ variable("_slot.player.username"), " - ", variable("_slot.player.status") }),
                                font_size = font_size,
                                color = WHITE,
                                width = "fill",
                                height = size
                            },
                            space("fill", size)
                        ),
                        cond(
                            "_slot.filled",
                            -- if the slot is filled, show the user profile of the user in the slot
                            multiplayer_action("show_slot_profile", variable("_slot.id")), 
                            -- otherwise, if its empty, try to move to it
                            cond(
                                "_slot.empty",
                                multiplayer_action("move_to_slot", variable("_slot.id")),
                                no_action()
                            )
                        ),
                        "fill",
                        "shrink",
                        padding
                    )
                })
            }
        }),
        
        
        -- beatmap info button and gameplay preview
        col({ width = "fill", height = "fill", spacing = 10.0 }, {
            -- beatmap info
            beatmap_info_button,

            -- leave lobby and ready/unready button
            row({ width = "fill", height = "fill" }, {
                -- leave lobby button
                button(text("Leave lobby"), multiplayer_action("leave")),

                -- gap
                space("fill", "fill"),

                -- ready/unready button
                ready_button
            }),

            -- gameplay preview
            {
                id = "gameplay_preview",
                debug_name = "gameplay_preview",
                width = "fill",
                height = "fill_portion(4)"
            }


        })

    })
}

add_menu(menu)