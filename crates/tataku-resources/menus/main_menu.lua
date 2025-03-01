-- helper for the song controls
local function fa_button(char, action)
    return {
        id = "styled_content",
        debug_name = "fa_button",
        color = color(1.0, 1.0, 1.0, 0.1),
        shape = { round = 5.0 },
        width = "auto",
        height = "auto",
        margin = 5.0,

        element = button({ width = "fill", height = "fill", padding = 15.0 }, 
            text(
                char, -- FA char
                30.0, -- font size
                WHITE, -- color
                "font_awesome", -- font
                "center" -- alignment
            ), -- text
            action -- action
        )
    }
end

local song_controls = row({ width = "percent(30.0)", height = "fill", debug_name = "song_controls" }, {
    -- the actual controls
    col({ width = "fill", height = "fill", debug_name = "song_controls_col" }, {
        -- buttons
        row({ width = "fill", height = "fill", margin = 5.0, padding = 5.0, justify_content = "space-between", debug_name = "song_controls_row" }, {
            fa_button(0xF04A, map_action("previous")), -- previous song
            fa_button(0xF048, song_action({ seek = -500.0 })), -- seek backwards
            -- space("fill", "auto"), -- some padding
            cond(
                "song.playing", -- condition
                fa_button(0xF04C, song_action("pause")), -- pause
                fa_button(0xF04B, song_action("play")) -- play
            ),
            -- space("fill", "auto"), -- some padding
            fa_button(0xF051, song_action({ seek = 500.0 })), -- seek forwards
            fa_button(0xF04E, map_action("next")), -- next song
        }),

        -- progress bar (TODO)
    }),

    -- -- empty space so the controls arent the whole width
    -- space("fill", "fill")
});

local song_display = {
    id = "styled_content",
    color = color(1.0, 1.0, 1.0, 0.1),
    shape = { round = 5.0 },
    padding = 8.0,
    width = "auto",

    element = text(
        text_list({ variable("beatmaps.current.map.artist"), " - ", variable("beatmaps.current.map.title") }),
        30,
        WHITE
    )
}


local start_singleplayer = menu_action("beatmap_select")
local start_multiplayer = menu_action("lobby_select")
local exit_game = game_action("quit")


-- notification example
local notify_now_playing = {
    event = "song_start",
    action = game_action("show_notification", {
        text = text_list({ "Now playing: ", variable("beatmaps.current.map.title"), " by ", variable("beatmaps.current.map.artist")}),
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

        notify_now_playing,

        --[[previous map]] key_event("Left", map_action("previous")),
        --[[next map]] key_event("Right", map_action("next"))
    },

    -- the current main menu is broken up into rows
    element = col({ width = "fill", height = "fill", debug_name = "main_menu" }, {
        -- the first row contains the song display
        row({ width = "fill", height = "auto", align_content = "end", debug_name = "main_menu_row1" }, {
            song_display
        }),

        -- the next row is the preview and menu buttons
        row({ width = "fill", height = "fill", debug_name = "main_menu_row2" }, {
            -- preview
            {
                id = "gameplay_preview",
                visualization = "menu_visualization",
                width = "percent(80.0)",
                height = "fill"
            },

            -- buttons, but inside an animatable element (to hide/unhide)
            {
                id = "animatable", 
                debug_name = "animatable",
                width = "percent(20.0)",
                height = "fill",

                -- TODO: fix the actions lmao
                triggers = {
                    -- -- on any input, it will unhide
                    -- { trigger = "input", action = "unhide" },
                    -- -- on lack of input for 10s, hide
                    -- { trigger = "no_input", action = "hide", duration = 1000.0 } -- 10000.0
                },
                actions = {
                    hide = {
                        -- end is a keyword, so we use start/stop instead (annoying but whatever)
                        -- duration is in ms
                        -- "current" refers to its existing position
                        { action = "scale_x", start = 1.0, stop = 0.0, duration = 1000.0 }
                    },
                    unhide = {
                        -- layout_pos is the object's expected position in the layout
                        { action = "scale_x", start = 0.0, stop = 1.0, duration = 1000.0 }
                    }
                },
                element = col({ width = "fill", height = "fill", debug_name = "main_menu_buttons_list" }, {
                    --[[ Singleplayer ]] button({ margin = 5.0 }, text("Play"), start_singleplayer),
                    --[[ Multiplayer ]] button({ margin = 5.0 }, text("Multiplayer"), start_multiplayer),
                    --[[ Settings ]] button({ margin = 5.0 }, text("Settings"), { id = "action", dialog = "settings" } ),
                    --[[ Quit ]] button({ margin = 5.0 }, text("Quit"), exit_game),
                })
            }
        }),

        -- the next row is the media controls
        row({ width = "fill", height = "auto", debug_name = "media_controls" }, {
            song_controls
        }),

    })

}

add_menu(menu)