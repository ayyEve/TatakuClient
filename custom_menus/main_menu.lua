menu = {
    id = "main_menu",

    -- the current main menu is broken into rows
    element = col({
        -- the first row contains the song display
        {
            id = "row",
            width = "fill",
            -- height = "fill", 
            elements = { song_display }
        },

        -- the next row is the preview and menu buttons
        row({
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
                element = col({
                    -- context is a table or list of tags to help the game know what was happening before, or to pass arguments into the menu
                    -- TODO: do we even need context? 
                    --[[ Singleplayer ]] { id = "button", text = "Play", action = start_singleplayer },
                    --[[ Multiplayer ]] { id = "button", text = "Multiplayer", action = start_multiplayer },
                    --[[ Settings ]] { id = "button", text = "Settings", action = { dialog = "settings" } },
                    --[[ Quit ]] { id = "button", text = "Quit", action = exit_game }
                })
            }
        }),
        
        -- the next row is the media controls (unhelpfully named "music_player")
        row({
            music_player -- not implemented yet
        })
    })
    
}

add_menu(menu)