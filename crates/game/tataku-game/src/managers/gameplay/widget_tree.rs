use crate::prelude::*;

// TODO: NEB!!!! 
// TODO: NEB!!!! 
// TODO: NEB!!!! cyclic dependencies
// TODO: NEB!!!! 
// TODO: NEB!!!! 

use engine::gameplay::widgets::{
    GameplayWidgetContainer,
    GameplayWidgetAnchor,
};

use std::collections::hash_map::Entry;



use tataku::{ Bounds, Vector2, };
use engine::{
    gameplay::widgets::{
        GameplayWidgetLayoutError,
        Side,
    }
};


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



    // TODO: NEB! i dont know what you want to do with this so i put it here for now:tm:
    pub fn layout(
        &mut self,
        playfield: Bounds,
        screen_size: Vector2,
    ) -> Result<(), GameplayWidgetLayoutError> {
        // list of elements that have been layed out
        let mut layed_out = Vec::with_capacity(self.elements.len());
        let mut remainder = Vec::with_capacity(self.elements.len());

        let screen_bounds = Bounds::new(
            Vector2::ZERO,
            screen_size,
        );

        // lay out everything with a screen or absolute anchor
        for element in self.elements.iter_mut() {
            let layout = element.layout();

            match layout.anchor {
                GameplayWidgetAnchor::Screen => {
                    element.resolved_pos = layout.align.resolve(
                        &screen_bounds,
                        element.preferred_size,
                        true,
                        true,
                    );

                    layed_out.push(element);
                },
                GameplayWidgetAnchor::Playfield { horizontal_side, vertical_side } => {
                    element.resolved_pos = layout.align.resolve(
                        &playfield,
                        element.preferred_size,
                        matches!(horizontal_side, Side::Inside),
                        matches!(vertical_side, Side::Inside),
                    );

                    layed_out.push(element);
                },
                // Do these in later passes
                GameplayWidgetAnchor::Element { .. } => {
                    remainder.push(element);
                },
            };
        }

        // there is likely a better way of doing this
        while !remainder.is_empty() {
            let remaining_count = remainder.len();

            for i in (0..remaining_count).rev() {
                let Some(element) = remainder
                    .get_mut(i)
                else { break };

                let layout = element.layout();

                let GameplayWidgetAnchor::Element {
                    element: anchored_to,
                    horizontal_side,
                    vertical_side,
                } = &layout.anchor else { unreachable!("Screen and Playfield anchors have already been resolved") };

                let Some(anchored_to) = layed_out
                    .iter()
                    .find(|e| &e.name == anchored_to)
                    else { continue };

                let anchored_bounds = Bounds::new(
                    anchored_to.resolved_pos,
                    anchored_to.preferred_size,
                );

                element.resolved_pos = layout.align.resolve(
                    &anchored_bounds,
                    element.preferred_size,
                    matches!(horizontal_side, Side::Inside),
                    matches!(vertical_side, Side::Inside),
                );

                layed_out.push(remainder.swap_remove(i));
            }

            // Keep iterating until one loop through no longer makes progress
            if remaining_count != remainder.len() { continue }

            for remaining in remainder.iter() {
                let layout = remaining.layout();

                let GameplayWidgetAnchor::Element {
                    element: anchored_to,
                    ..
                } = &layout.anchor else { unreachable!("Screen and Playfield anchors have already been resolved") };

                let found = layed_out
                    .iter()
                    .chain(remainder.iter())
                    .any(|e| &e.name == anchored_to);

                if !found {
                    return Err(GameplayWidgetLayoutError::InvalidElementReference(
                        anchored_to.clone().into_owned()
                    ));
                }
            }

            return Err(GameplayWidgetLayoutError::CyclicDependencyDetected);
        }

        Ok(())
    }
}
