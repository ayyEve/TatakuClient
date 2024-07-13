-- helper for the song controls
local function fa_button(char, action) 
    return {
        id = "styled_content",
        debug_name = "fa_button",
        color = color(1.0, 1.0, 1.0, 0.1),
        shape = { round = 5.0 },
        width = "shrink",
        height = "shrink",

        element = button(
            text(
                char, -- FA char
                30.0, -- font size
                WHITE, -- color
                "font_awesome" -- font
            ), -- text
            action,-- action
            "shrink", -- width
            "shrink", -- height
            15.0 -- padding
        )
    }
end

local song_controls = row({ width = "shrink", height = "shrink", debug_name = "song_controls" }, {
    -- the actual controls
    col({ width = "fill", height = "fill", spacing = 2.0, debug_name = "song_controls_col" }, {
        -- buttons
        row({ width = "fill", height = "fill_portion(10)", spacing = 5.0, debug_name = "song_controls_row" }, {
            fa_button(0xF04A, map_action("previous")), -- previous song
            fa_button(0xF048, song_action({ seek = -500.0 })), -- seek backwards
            space("fill", "shrink"), -- some padding
            cond(
                "song.playing", -- condition
                fa_button(0xF04C, song_action("pause")), -- pause
                fa_button(0xF04B, song_action("play")) -- play
            ),
            space("fill", "shrink"), -- some padding
            fa_button(0xF051, song_action({ seek = 500.0 })), -- seek forwards
            fa_button(0xF04E, map_action("next")), -- next song
        }),

        -- progress bar (TODO)
    }),

    -- empty space so the controls arent the whole width
    space("fill_portion(4)", "fill")
});

local song_display = {
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


local start_singleplayer = menu_action("beatmap_select")
local start_multiplayer = menu_action("lobby_select")
local exit_game = game_action("quit")


-- notification example
local notify_now_playing = { 
    event = "song_start", 
    action = game_action("show_notification", { 
        text = text_list({ "Now playing: ", variable("map.title"), " by ", variable("map.artist")}),
        duration = 10000, -- ms
        color = TEAL,
    })
}


local menu = {
    id = "main_menu",

    events = { 
        -- on song end, play next song
        { event = "song_end", action = map_action("next") },

        { event = "map_added", action = map_action("select_map", { map_hash = passed_in() }) },

        -- notify_now_playing,

        --[[previous map]] key_event("Left", map_action("previous")),
        --[[next map]] key_event("Right", map_action("next"))
    },

    -- the current main menu is broken up into rows
    element = col({ width = "fill", height = "fill", debug_name = "main_menu" }, {
        -- the first row contains the song display
        row({ width = "fill", height = "fill", debug_name = "main_menu_row1" }, { 
            song_display 
        }),

        -- the next row is the preview and menu buttons
        row({ width = "fill", height = "fill_portion(10)", debug_name = "main_menu_row2" }, {
            -- preview
            {
                id = "gameplay_preview",
                visualization = "menu_visualization",
                width = "fill_portion(4)",
                height = "fill"
            },

            -- buttons, but inside an animatable element (to hide/unhide)
            {
                id = "animatable", -- not implemented yet because animating iced elements is pain
                debug_name = "animatable",
                width = "shrink", 
                height = "shrink",

                triggers = {
                    -- on any input, it will unhide
                    { trigger = "input", action = "unhide" },
                    -- on lack of input for 10s, hide
                    { trigger = "no_input", action = "hide", duration = 10000.0 }
                },
                actions = {
                    hide = {
                        -- end is a keyword, so we use start/stop instead (annoying but whatever)
                        -- duration is in ms
                        -- "current" refers to its existing position
                        { action = "translate_x", start = "current", stop = "parent_width", duration = 1000.0 },
                        { action = "opacity", start = "current", stop = 0.0, duration = 1000.0 }
                    },
                    unhide = {
                        -- layout_pos is the object's expected position in the layout
                        { action = "translate_x", start = "current", stop = "layout_pos_x", duration = 1000.0 },
                        { action = "opacity", start = "current", stop = 1.0, duration = 1000.0 }
                    }
                },
                element = col({ width = "fill", height = "fill", spacing = 5.0, debug_name = "main_menu_buttons_list" }, {
                    --[[ Singleplayer ]] button(text("Play"), start_singleplayer), 
                    --[[ Multiplayer ]] button(text("Multiplayer"), start_multiplayer),
                    --[[ Settings ]] button(text("Settings"), { id = "action", dialog = "settings" } ),
                    --[[ Quit ]] button(text("Quit"), exit_game),
                })
            }
        }),
        
        -- the next row is the media controls
        row({ width = "fill", height = "fill" }, {
            song_controls
        }),

    })
    
}

add_menu(menu)