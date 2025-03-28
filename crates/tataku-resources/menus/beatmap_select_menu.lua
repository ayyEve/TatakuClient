local scores_list = {
    id = "list",
    debug_name = "scores list",
    width = "fill",
    height = "fill",
    flex_direction = "column",

    list = "score_list.scores",
    variable = "_score",
    scroll = true,

    element = button({ width = "fill", height = "auto", margin = 5.0 }, 
        col({ width = "fill", height = "auto" }, {
            -- username and score 
            row({ width = "fill" }, { text(text_list({ variable("_score.score.username"), ": ", calc("display(_score.score.score)")}), 16.0, WHITE) }),
            -- combo, acc, mods
            row({ width = "fill" }, { text(text_list({ calc("display(_score.score.max_combo)"), "x, ", calc("display(_score.score.accuracy * 100.0)"), "% ", text_iter("_score.score.mods", ".display_name", ",")}), 16.0, WHITE) }),
        }),
        game_action("view_score", { score_id = variable("_score.id") })
    )
}

local map_list = {
    id = "list",
    debug_name = "map_list",
    width = "fill",
    height = "auto",
    margin = 2.0,
    
    justify_content = "space-around",
    align_content = "end",
    flex_wrap = "wrap",
    flex_direction = "column",
    flex_shrink = 0.0,
    -- align_content = "space-around",
    -- justify_content = "end",
    -- flex_wrap = "wrap",
    -- flex_direction = "row",

    list = "_group.maps",
    variable = "_map",
    element = button({ width = "auto", height = "auto", flex_shrink = 0.0, padding = 5.0, margin = 2.0, active_cond = "beatmaps.current.map.beatmap_hash == _map.map.beatmap_hash" },
        col({ width = "fill", height = "auto" }, {
            -- title and stuff
            -- cond(
            --     "_map.diff_rating > 0.0",
                text(
                    text_list({ 
                        variable("_map.map.mode"), 
                        " - ", 
                        variable("_map.map.version"), 
                        " (diff: ", calc("display(_map.diff_rating)"), ")" 
                    }), -- display(_map.map.playmode)
                    20,
                    WHITE
                ),
            --     text(
            --         text_list({ variable("_map.map.mode"), " - ", variable("_map.map.version") }), -- display(_map.map.playmode)
            --         20,
            --         WHITE
            --     )
            -- ),
            
            -- map diff info
            text(
                variable("_map.diff_info"),
                20,
                WHITE
            )
        }),
        cond(
            "beatmaps.current.map.beatmap_hash == _map.map.beatmap_hash", -- if the map is selected
            -- confirm it
            map_action("confirm"),
            -- otherwise, set it as the selected map 
            map_action("select_map", { map_hash = variable("_map.map.beatmap_hash")})
        )
    )
}

local beatmap_list = {
    -- group items
    id = "list",
    debug_name = "groups list",
    width = "fill",
    height = "auto",
    flex_direction = "column",
    flex_shrink = 0.0,

    list = "beatmaps.groups",
    variable = "_group",
    scroll = true,

    element = col({ width = "fill", height = "auto", padding = 5.0, flex_shrink = 0.0, margin = 2.0, debug_name = "beatmaplist_col" }, {
        -- set info
        button({ width = "fill", height = "auto", padding = 5.0, active_cond = "_group.selected" }, 
            text(variable("_group.name"), 20.0, WHITE),
            -- set this as the selected set
            map_action("select_group", { group_id = variable("_group.id") })
        ),

        -- map items
        cond(
            "_group.selected", -- if this group is selected
            -- show a list of maps
            map_list
        )
    })
}

local menu = {
    id = "beatmap_select",

    events = {
        -- on entering menu, make sure song is playing, and is also at the proper rate
        {
            event = "menu_enter",
            actions = {
                -- song_action({ rate = 1.0 }),
                cond("!song.playing", song_action("play")),
                cursor_action("show"), -- also make sure the cursor is visible
                song_action({ rate = variable("global.mods.speed")})
            }
        },

        -- on song end, restart map
        {
            event = "song_end",
            actions = {
                song_action({ position = variable("beatmaps.current.map.preview")}),
                song_action("play")
            }
        },

        -- automatically select new maps
        { event = "map_added", action = map_action("select_map", { map_hash = passed_in() }) },

        -- make sure new songs are set with the correct rate
        { event = "song_start", action = song_action({ rate = variable("global.mods.speed")}) },

        --[[previous set]] key_event("Left", map_action("previous_set")),
        --[[next set]] key_event("Right", map_action("next_set")),
        --[[previous map]] key_event("Up", map_action("previous_map")),
        --[[next map]] key_event("Down", map_action("next_map")),
        --[[back to main menu]] key_event("Escape", menu_action("main_menu")),

        --[[mods dialog]] key_event("M", {"ctrl"}, dialog_action("mods")),

        -- mods
        --[[nofail]] key_event("N", {"ctrl"}, mod_action({ toggle = "no_fail" }) ),
        --[[autoplay]] key_event("A", {"ctrl"}, mod_action({ toggle = "autoplay" })),

        --[[add speed]] key_event("Equals", {"ctrl"}, mod_action({ add_speed = 0.1 })),
        --[[remove speed]] key_event("Minus", {"ctrl"}, mod_action({ add_speed = -0.1 })),
    },

    -- the beatmap select menu is broken up into rows
    element = col({ width = "fill", height = "fill" }, {
        -- the first row contains the dropdowns and search
        row({ width = "fill", height = "auto", justify_content = "space-between", debug_name="dropdowns" }, {
            -- score get method dropdown
            {
                id = "dropdown",
                debug_name = "score method dropdown",
                width = "percent(20.0)",
                font_size = 25.0,

                options_key = "enums.score_methods",
                selected_key = "settings.score_method",
                on_select = { id = "set_value", key = "settings.score_method", passed_in = true },
            },
            -- mode dropdown
            {
                id = "dropdown",
                debug_name = "playmode dropdown",
                width = "percent(20.0)",
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
                width = "percent(20.0)",
                font_size = 25.0,

                placeholder = "Sort",

                options_key = "enums.sort_by",
                selected_key = "settings.sort_by",
                on_select = { id = "set_value", key = "settings.sort_by", passed_in = true },
            },

            -- filter text input
            {
                id = "text_input",
                width = "percent(20.0)",
                font_size = 25.0,

                on_input = map_action("refresh_list"),
                placeholder = "search",
                variable = "beatmaps.filter_text",
            }
        }),

        -- the next row has the score list + back button, gameplay preview, and beatmap list
        row({ width = "fill", height = "fill" }, {
            -- score list and back back button
            col({ width = "percent(10.0)", height = "percent(95.0)", justify_content = "space-between", debug_name = "score_list"  }, {
                {
                    id = "styled_content",
                    debug_name = "scores list styled",
                    color = color(1.0, 1.0, 1.0, 0.1),
                    shape = { round = 5.0 },
                    width = "fill",
                    height = "fill",

                    element = cond(
                        "!score_list.loaded", -- if not loaded...
                        text("Loading...", 16.0, WHITE), -- show loading text
                        cond( -- otherwise,
                            "score_list.scores.is_empty", -- if empty
                            text("No scores", 16.0, WHITE), -- show no scores text
                            scores_list -- otherwise, show score list
                        )
                    )
                },

                -- back button
                row({ width = "fill", height = "auto" }, {
                    button(text("Back"), menu_action("main_menu"))
                })
            }),

            --- panel scroll
            row({ width = "percent(90.0)", height = "fill" }, {
                -- preview
                {
                    id = "gameplay_preview",
                    debug_name = "gameplay_preview",
                    width = "percent(40.0)",
                    height = "fill"
                },

                -- beatmap list
                {
                    id = "styled_content",
                    debug_name = "beatmap list styled",
                    color = color(1.0, 1.0, 1.0, 0.1),
                    shape = { round = 5.0 },
                    width = "percent(60.0)",
                    height = "fill",
                    element = beatmap_list
                }
            }),
        }),

    })

}

add_menu(menu)