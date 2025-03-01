
local menu = {
    id = "menu_list",

    element = col({ width = "fill", height = "fill" }, {
        text("Menu list"),
        {
            id = "list",
            width = "fill",
            height = "auto",

            list = "global.menu_list",
            variable = "_menu",
            scroll = true,

            element = row({ width = "fill", height = "auto", margin = 10.0 }, {
                button({ width = "fill", height = "auto", padding = 10.0 }, 
                    text(variable("_menu")),
                    menu_action(variable("_menu"))
                )
            })
        }
    })
}

add_menu(menu)