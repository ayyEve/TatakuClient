-- TODO!!: lists
menu = {
    id = "beatmap_select",

    -- the the beatmap select menu is broken up into rows
    element = col({
        -- the first row contains the dropdowns
        row({
            -- mode dropdown
            {
                id = "dropdown",
                item = "playmode",
            }
        }),

        -- the next row has the score list, gameplay preview, and beatmap list
        row({
            -- score list


            -- preview
            {
                id = "gameplay_preview",
                width = "fill_portion(4)",
                height = "fill"
            },

            -- beatmap list
        }),
        
        -- the last row currently has nothing lol
    })
    
}

add_menu(menu)