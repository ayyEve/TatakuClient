local scores_list = {
    id = "list",
    debug_name = "scores list",
    width = "fill",
    height = "fill",

    list = "score_list.scores",
    variable = "_score",
    scroll = true,

    element = button(
        col({ width = "fill", height = "shrink" }, {
            -- username and score
            row({ width = "fill" }, { text(text_list({variable("_score.username"), ": ", calc("display(_score.score)")}), 16.0, WHITE) }),
            -- combo, acc, mods
            row({ width = "fill" }, { text(text_list({calc("display(_score.max_combo)"), "x, ", calc("display(_score.accuracy)"), "% ", calc("display(_score.mods)")}), 16.0, WHITE) }),
        }),
        game_action("view_score", { score_id = variable("_score.id") }),
        "fill",
        "shrink"
    )
}

local beatmap_list = {
    -- group items
    id = "list",
    debug_name = "groups list",
    width = "fill",
    height = "shrink",

    list = "beatmap_list.groups",
    variable = "_group",
    scroll = true,

    element = col({ width = "fill", height = "shrink", spacing = 5.0, debug_name = "col" }, {
        -- set info
        button(
            text(variable("_group.name"), 20.0, WHITE),
            -- set this as the selected set
            map_action("select_group", { group_id = variable("_group.id") }),
            "fill",
            "shrink",
            5.0
        ),
        
        -- map items
        cond(
            "_group.selected", -- if this group is selected
            -- show a list of maps
            {
                id = "list",
                debug_name = "song list",
                width = "fill",
                height = "shrink",

                list = "_group.maps",
                variable = "_map",
                element = row({ width = "fill", height = "shrink", spacing = 2.0 }, {
                    -- add some space to indent the list
                    space("fill_portion(1)", "shrink"),

                    -- and the rest of the list
                    col({ width = "fill_portion(10)", height = "shrink", spacing = 5.0, debug_name = "song text" }, {
                        button(
                            text(
                                text_list({ calc("display(_map.playmode)"), " - ", variable("_map.version") }),
                                20, 
                                WHITE
                            ),
                            cond(
                                "map.hash == _map.hash", -- if the map is selected
                                -- confirm it
                                map_action("confirm"),
                                -- otherwise, set it as the selected map
                                map_action("select_map", { map_hash = variable("_map.hash")})
                            ),
                            "fill",
                            "shrink",
                            5.0
                        )
                        -- diff info here
                    })

                })
                
            }
        )
    })
}

local menu = {
    id = "beatmap_select",

    events = {
        -- on entering menu, make sure song is playing, and is also at a rate of 1.0
        {
            event = "menu_enter",
            actions = {
                song_action({ rate = 1.0 }),
                cond("!song.playing", song_action("play")),
            }
        },
        
        -- on song end, restart map
        { 
            event = "song_end", 
            actions = { 
                song_action({ position = variable("map.preview_time")}),
                song_action("play") 
            }
        },

        -- automatically select new maps
        { event = "map_added", action = map_action("select_map", { map_hash = passed_in() }) },


        --[[previous set]] key_event("Left", map_action("previous_set")),
        --[[next set]] key_event("Right", map_action("next_set")),
        --[[previous map]] key_event("Up", map_action("previous_map")),
        --[[next map]] key_event("Down", map_action("next_map")),
        --[[next map]] key_event("Escape", { id = "action", menu = "main_menu" }),

        --[[mods dialog]] key_event("M", {"ctrl"}, dialog_action("mods")),

        -- mods
        --[[nofail]] key_event("N", {"ctrl"}, mod_action({ toggle = "no_fail" })),
        --[[autoplay]] key_event("A", {"ctrl"}, mod_action({ toggle = "autoplay" })),

        --[[add speed]] key_event("Equals", {"ctrl"}, mod_action({ add_speed = 0.1 })),
        --[[remove speed]] key_event("Minus", {"ctrl"}, mod_action({ add_speed = -0.1 })),
    },

    -- the beatmap select menu is broken up into rows
    element = col({ width = "fill", height = "fill", spacing = 10.0 }, {
        -- the first row contains the dropdowns and search
        row({ width = "fill", height = "shrink", spacing = 10.0, debug_name="dropdowns" }, {
            -- score get method dropdown
            {
                id = "dropdown",
                debug_name = "score method dropdown",
                width = "fill",
                font_size = 25.0,

                options_key = "enums.score_methods",
                selected_key = "settings.score_method",
            },
            -- mode dropdown
            {
                id = "dropdown",
                debug_name = "playmode dropdown",
                width = "fill",
                font_size = 25.0,

                placeholder = "Mode",
                on_select = map_action("set_playmode", { playmode = passed_in() }),

                options_key = "enums.playmodes",
                selected_key = "global.playmode",
            },
            -- sort_by dropdown
            {
                id = "dropdown",
                debug_name = "sort_by dropdown",
                width = "fill",
                font_size = 25.0,

                placeholder = "Sort",

                options_key = "enums.sort_by",
                selected_key = "settings.sort_by",
            },

            -- filter text input
            {
                id = "text_input",
                width = "fill",

                on_input = map_action("refresh_list"),
                placeholder = "search",
                variable = "beatmap_list.search_text",
            },
        }),

        -- the next row has the score list, gameplay preview, and beatmap list
        row({ width = "fill", height = "fill", debug_name = "score list" }, {
            -- score list
            {
                id = "styled_content",
                debug_name = "scores list styled",
                color = color(1.0, 1.0, 1.0, 0.1),
                shape = { round = 5.0 },
                width = "fill",
                height = "fill",
        
                element = cond(
                    "!score_list.loaded", -- if not loaded...
                    text("Loading..."), -- show loading text
                    cond( -- otherwise,
                        "score_list.empty", -- if empty
                        text("No scores"), -- show no scores text
                        scores_list -- otherwise, show score list
                    )
                )
            },

            -- preview
            {
                id = "gameplay_preview",
                debug_name = "gameplay_preview",
                width = "fill_portion(4)",
                height = "fill"
            },

            -- beatmap list
            {
                id = "styled_content",
                debug_name = "beatmap list styled",
                color = color(1.0, 1.0, 1.0, 0.1),
                shape = { round = 5.0 },
                width = "fill_portion(4)",
                height = "fill",
                element = beatmap_list
            },
        }),
        
    })
    
}

add_menu(menu)