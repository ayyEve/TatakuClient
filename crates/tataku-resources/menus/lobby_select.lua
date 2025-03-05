
local menu = {
    id = "lobby_select",

    events = {
        key_event("Escape", { id = "action", menu = "main_menu" }),
    },

    element = col({ width = "fill", height = "fill" }, {
        -- list
        {
            id = "list",
            debug_name = "lobby list",
            
            list = "global.lobbies",
            variable = "_lobby",
            scroll = true,
            width = "fill",
            height = "auto",

            element = button({ width = "fill", height = "auto", padding = 5.0 },
                text(variable("_lobby.name"), 30.0),
                multiplayer_action("join_lobby", { lobby_id = variable("_lobby.id") })
            )
        },

        -- move buttons to the bottom
        space("fill", "fill"),

        -- buttons
        row({ width = "fill", height = "auto"}, {
            button(text("Create Lobby", 30.0), dialog_action("create_lobby")),
            -- button(text("Create Lobby", 30.0), multiplayer_action("create")),
        })
    })
    
}


add_menu(menu)