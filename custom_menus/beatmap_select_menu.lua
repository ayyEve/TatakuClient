-- TODO!!: lists
menu = {
    id = "beatmap_select_menu",

    -- list of components we want to add, to add extra functionality
    components = {
        -- a component which adds beatmap list values
        {
            id = "beatmap_list",
        }
    },

    -- the the beatmap select menu is broken up into rows
    element = col({ width = "fill", height = "fill" }, {
        -- the first row contains the dropdowns
        row({ width = "fill", height = "shrink", debug_name="dropdowns" }, {
            -- -- mode dropdown
            -- {
            --     id = "dropdown",
            --     item = "playmode",
            -- }
            space("fill", "shrink"), --TODO!
        }),

        -- the next row has the score list, gameplay preview, and beatmap list
        row({ width = "fill", height = "fill", debug_name = "score list" }, {
            -- score list
            space("fill", "fill"), --TODO!

            -- preview
            {
                id = "gameplay_preview",
                debug_name = "gameplay_preview",
                width = "fill_portion(4)",
                height = "fill"
            },

            -- beatmap list
            {
                -- group items
                id = "list",
                debug_name = "groups list",
                width = "fill_portion(4)",
                height = "shrink",

                list = "beatmap_list.groups",
                variable = "_group",
                scroll = true,

                element = col({ width = "fill", height = "shrink", spacing = 5.0, debug_name = "col" }, {
                    -- set info
                    button(
                        text({value="_group.name"}, 20.0, WHITE),
                        -- set this as the selected set
                        {
                            id = "custom",
                            tag = "beatmap_list.set_set",
                            variable = "_group.id"
                        },
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
                                            text_list({ {value="_map.display_mode"}, " - ", {value="_map.version"} }),
                                            20, 
                                            WHITE
                                        ),
                                        cond(
                                            "map.hash == _map.hash", -- if the map is selected
                                            -- play it
                                            map_action("play"),
                                            -- otherwise, set it as the selected map
                                            { 
                                                id = "custom",
                                                tag = "beatmap_list.set_beatmap",
                                                variable = "_map.hash",
                                            }
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
        }),
        
        -- the last row currently has nothing lol
    })
    
}

add_menu(menu)