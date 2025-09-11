use crate::prelude::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    widget::*,
    message::*,
};
use input::InputEvent;

#[derive(ChainableInitializer)]
pub struct TabbedWidget {
    name: ArcStr,
    tabs: TabProvider,
    #[chain] selected: usize,

    node_id: NodeId,
}
impl TabbedWidget {
    pub fn new(
        name: impl Into<ArcStr>,
        tabs: impl Into<TabProvider>,
    ) -> Self {
        Self {
            name: name.into(),
            tabs: tabs.into(),
            selected: 0,
            node_id: ui::EMPTY_NODE
        }
    }

    // #[allow(clippy::borrowed_box, reason = "Box<dyn Widget> doesnt implement dyn Widget, and dereferencing and re-referencing is unecessary and ugly")]
    fn get_ele(&self) -> Option<&Tab> {
        self.tabs
            .tabs()
            .get(self.selected)
    }

    fn get_ele_mut(&mut self) -> Option<&mut Tab> {
        self.tabs
            .tabs_mut()
            .get_mut(self.selected)
    }
}
impl Widget<actions::Action> for TabbedWidget {
    fn name(&self) -> CowStr { format!("tabbed_widget({})", self.name).into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn operation(
        &mut self,
        operation: &UiOperation,
        tree: &mut Tree<actions::Action>,
    ) {
        if operation.target.resolve(self, tree) {
            #[allow(clippy::single_match, reason = "will want to add more later")]
            match &operation.operation {
                UiOperationType::SetTab(tab_name) => {
                    for (n, tab) in self.tabs.tabs().iter().enumerate() {
                        if &tab.name == tab_name {
                            self.selected = n;
                            break;
                        }
                    }
                }

                _ => {}
            }
        } else {
            for tab in self.tabs.tabs_mut() {
                tab.element.operation(operation, tree);
            }
        }

    }

    // fn update_styles(
    //     &mut self,
    //     shell: &mut StyleShell,
    //     display_override: Option<DisplayType>
    // ) {
    //     for tab in self.tabs.tabs_mut() {
    //         tab.element.update_styles(shell, display_override);
    //     }
    // }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId>  {
        let children = self.tabs
            .tabs_mut()
            .iter_mut()
            .map(|i| i.element.layout(shell))
            .collect::<Result<Vec<_>, _>>()?;

        self.node_id = shell.tree.new_with_children(&children)?;
        Ok(self.node_id)
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(child) = self.get_ele() else { return };
        child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(child) = self.get_ele() else { return };
        child.draw(shell);
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.input(event, shell);
    }



    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let Some(child) = self.get_ele_mut() else { return };
        child.update(shell);
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.handle_message(message, shell);
    }

    fn handle_event(
        &mut self,
        event: &input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let Some(child) = self.get_ele_mut() else { return };
        child.handle_event(event, event_value, shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell<actions::Action>) {
        for tab in self.tabs.tabs_mut() {
            tab.element.reload_skin(shell);
        }
    }
}


pub enum TabProvider {
    Static(Vec<Tab>),
    Programmatic {
        list: String,
        variable: String,
        name_path: String,
        template: Element,
        cached: Vec<Tab>,
    }
}
impl TabProvider {
    fn tabs(&self) -> &Vec<Tab> {
        match self {
            Self::Static(tabs) => tabs,
            Self::Programmatic { cached, .. } => cached,
        }
    }
    fn tabs_mut(&mut self) -> &mut Vec<Tab> {
        match self {
            Self::Static(tabs) => tabs,
            Self::Programmatic { cached, .. } => cached,
        }
    }
}
impl From<Vec<Tab>> for TabProvider {
    fn from(value: Vec<Tab>) -> Self {
        Self::Static(value)
    }
}



pub struct Tab {
    name: String,
    element: Box<dyn Widget<actions::Action>>
}
impl Tab {
    pub fn new(
        name: String,
        element: Box<dyn Widget<actions::Action>>
    ) -> Self {
        Self {
            name,
            element,
        }
    }
}
impl Deref for Tab {
    type Target = dyn Widget<actions::Action>;
    fn deref(&self) -> &Self::Target {
        &*self.element
    }
}
impl DerefMut for Tab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.element
    }
}
