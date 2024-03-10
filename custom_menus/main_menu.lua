-- helper for the song controls
function fa_button(char, action) 
    return {
        id = "styled_content",
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
            action, -- action
            "shrink", -- width
            "shrink", -- height
            15.0 -- padding
        )
    }
end

song_controls = row({ width = "shrink", height = "shrink" }, {
    -- the actual controls
    col({ width = "fill", height = "fill", spacing = 2.0 }, {
        -- buttons
        row({ width = "fill", height = "fill_portion(10)", spacing = 5.0 }, {
            fa_button(0xF04A, { map = "previous" }), -- previous song
            fa_button(0xF048, { song = { seek = -500.0 } }), -- seek backwards
            space("fill", "shrink"),
            fa_button(0xF04C, { song = "toggle" }), -- play/pause
            space("fill", "shrink"),
            fa_button(0xF051, { song = { seek = 500.0 } }), -- seek forwards
            fa_button(0xF04E, { map = "next" }), -- next song
        }),

        -- progress bar (TODO)
    }),

    -- empty space so the controls arent the whole width
    space("fill_portion(4)", "fill")
});




menu = {
    id = "main_menu",

    -- the current main menu is broken up into rows
    element = col({ width = "fill", height = "fill" }, {
        -- the first row contains the song display
        row({ width = "fill", height = "fill" }, { 
            song_display 
        }),

        -- the next row is the preview and menu buttons
        row({ width = "fill", height = "fill_portion(10)" }, {
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
                element = col({ width = "fill", height = "fill", margin = 5.0 }, {
                    --[[ Singleplayer ]] button(text("Play"), start_singleplayer), 
                    --[[ Multiplayer ]] button(text("Multiplayer"), start_multiplayer),
                    --[[ Settings ]] button(text("Settings"), { dialog = "settings" } ),
                    --[[ Quit ]] button(text("Quit"), exit_game),
                })
            }
        }),
        
        -- the next row is the media controls
        row({ width = "fill", height = "fill" }, {
            song_controls
        }),

        -- lastly, we have the key handler
        -- it doesnt actually occupy any space, but is required to handle key input
        key_handler({
            -- previous map
            {
                key = "Left",
                action = { map = "previous" },
            },

            -- next map
            {
                key = "Right",
                action = { map = "next" },
                mods = { "ctrl" }
            }
        })
    })
    
}

add_menu(menu)