
local menu = {
    id = "menu_list",

    element = col({ width = "fill", height = "fill" }, {
        text("Menu list"),
        {
            id = "list",
            width = "fill",
            height = "shrink",

            list = "global.menu_list",
            variable = "_menu",
            scroll = true,

            element = row({ width = "fill", height = "shrink", spacing = 10.0 }, {
                button(
                    text(variable("_menu")),
                    menu_action(variable("_menu")),
                    "fill",
                    "shrink",
                    10.0
                )
            })
        }
    })
}

add_menu(menu)