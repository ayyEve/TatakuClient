use crate::prelude::*;

use engine::gameplay::widgets::{
    GameplayWidgetContainer,
    GameplayWidgetAnchor,
};

use std::collections::hash_map::Entry;

#[derive(Default)]
pub struct WidgetTree {
    elements: Vec<GameplayWidgetContainer>,

    elements_by_name: HashMap<CowStr, usize>,

    root: Node,
    /// Tree nodes for elements (matching at index), excluding root node
    nodes: Vec<Node>,
}

#[derive(Default)]
pub struct Node {
    dirty: bool,

    /// None means root
    parent: Option<usize>,
    children: Vec<usize>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn element(&self, name: CowStr) -> Option<&GameplayWidgetContainer> {
        Some(&self.elements[*self.elements_by_name.get(&name)?])
    }

    pub fn element_mut(&mut self, name: CowStr) -> Option<&mut GameplayWidgetContainer> {
        Some(&mut self.elements[*self.elements_by_name.get(&name)?])
    }

    /// Elements in an arbitrary order.
    pub fn elements(&self) -> impl Iterator<Item = &GameplayWidgetContainer> {
        self.elements.iter()
    }

    /// Elements in an arbitrary order.
    pub fn elements_mut(&mut self) -> impl Iterator<Item = &mut GameplayWidgetContainer> {
        self.elements.iter_mut()
    }

    pub fn breadth_first(&self) -> impl Iterator<Item = &GameplayWidgetContainer> {
        todo!();
        vec![].into_iter()
    }

    pub fn breadth_first_mut(&mut self) -> impl Iterator<Item = &mut GameplayWidgetContainer> {
        todo!();
        vec![].into_iter()
    }

    pub fn add_elements(&mut self, elements: impl IntoIterator<Item = GameplayWidgetContainer>) {
        let mut element_children: HashMap<CowStr, Vec<usize>> = HashMap::new();

        for element in elements {
            let index = self.elements.len();

            let mut parent = None;

            let layout = element.layout();

            if let GameplayWidgetAnchor::Element { element, .. } = &layout.anchor {
                if let Some(i) = self.elements_by_name.get(element).copied() {
                    // If the element already exists in the tree,
                    // add to its children.
                    self.nodes[i].children.push(index);
                    parent = Some(i);
                } else {
                    // Otherwise add to our local hashmap
                    match element_children.entry(element.clone()) {
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().push(index);
                        },
                        Entry::Vacant(vacant) => {
                            vacant.insert(vec![index]);
                        },
                    }
                }
            }

            let children = element_children.remove(&element.name)
                .unwrap_or_default();

            for child in children.iter().copied() {
                self.nodes[child].parent = Some(index);
            }

            self.elements_by_name.insert(element.name.clone(), index);
            self.nodes.push(Node {
                children,
                parent,
                ..Default::default()
            });
            self.elements.push(element);
        }

        for (element_name, children) in element_children {
            let children = children.into_iter()
                .map(|index| &self.elements[index].name)
                .fold(
                    String::new(),
                    |acc, string| format!("{acc}{string}, ")
                );

            warn!("Dangling references in gameplay widgets! Expected parent {element_name} with {children}");
        }
    }

    pub fn remove_elements(&mut self, elements: impl IntoIterator<Item = CowStr>) {
        todo!()
    }

    pub fn mark_dirty(&mut self, name: CowStr) {
        todo!()
    }
}
