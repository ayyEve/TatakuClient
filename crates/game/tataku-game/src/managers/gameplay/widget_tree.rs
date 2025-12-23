use crate::prelude::*;

use engine::gameplay::widgets::{
    GameplayWidgetContainer,
    GameplayWidgetAnchor,
};

use std::collections::{
    hash_map::Entry,
    VecDeque,
};

use tataku::{ Bounds, Vector2, };
use engine::{
    gameplay::widgets::{
        GameplayWidgetLayoutError,
        Side,
    }
};

#[derive(Debug)]
pub struct WidgetTree {
    /// Root node has None
    elements: Vec<Option<GameplayWidgetContainer>>,

    elements_by_name: HashMap<CowStr, usize>,

    /// Tree nodes for elements (matching at index), excluding root node
    nodes: Vec<Option<Node>>,

    /// Indices at which `elements` and `nodes` is None.
    empty_slots: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct Node {
    dirty: bool,

    /// None means root
    parent: Option<usize>,
    children: Vec<usize>,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            elements: vec![None],
            elements_by_name: HashMap::new(),
            nodes: vec![Some(Node::default())],
            empty_slots: Vec::new(),
        }
    }

    pub fn element(&self, name: &str) -> Option<&GameplayWidgetContainer> {
        let index = self.elements_by_name.get(name).copied()?;
        Some(self.elements[index].as_ref().expect("missing node"))
    }

    pub fn element_mut(&mut self, name: &str) -> Option<&mut GameplayWidgetContainer> {
        let index = self.elements_by_name.get(name).copied()?;
        Some(self.elements[index].as_mut().expect("missing node"))
    }

    /// Elements in an arbitrary order.
    pub fn elements(&self) -> impl Iterator<Item = &GameplayWidgetContainer> {
        self.elements.iter().filter_map(|x| x.as_ref())
    }

    /// Elements in an arbitrary order.
    pub fn elements_mut(&mut self) -> impl Iterator<Item = &mut GameplayWidgetContainer> {
        self.elements.iter_mut().filter_map(|x| x.as_mut())
    }

    /// Elements in an arbitrary but breadth-first order.
    pub fn breadth_first(&self) -> impl Iterator<Item = &GameplayWidgetContainer> {
        BreadthFirst {
            index: BreadthFirstIndex {
                nodes: &self.nodes,
                node_queue: self.nodes[0].as_ref().expect("missing root")
                    .children.clone().into(),
            },
            elements: &self.elements,
        }
    }

    /// Elements in an arbitrary but breadth-first order.
    pub fn breadth_first_mut(&mut self) -> impl Iterator<Item = &mut GameplayWidgetContainer> {
        BreadthFirstMut {
            index: BreadthFirstIndex {
                nodes: &self.nodes,
                node_queue: self.nodes[0].as_ref().expect("missing root")
                    .children.clone().into(),
            },
            elements: &mut self.elements,
        }
    }

    pub fn add_elements(&mut self, elements: impl IntoIterator<Item = GameplayWidgetContainer>) {
        let mut element_children: HashMap<CowStr, Vec<usize>> = HashMap::new();

        // todo: prevent cyclic dependencies

        for element in elements {
            let insert_into_index = self.empty_slots.pop()
                .unwrap_or(self.elements.len());

            let mut parent = None;

            let layout = element.layout();

            if let GameplayWidgetAnchor::Element { element, .. } = &layout.anchor {
                if let Some(i) = self.elements_by_name.get(element).copied() {
                    // If the element already exists in the tree,
                    // add to its children.
                    self.nodes[i].as_mut().expect("missing node")
                        .children.push(insert_into_index);
                    parent = Some(i);
                } else {
                    // Otherwise add to our local hashmap
                    match element_children.entry(element.clone()) {
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().push(insert_into_index);
                        },
                        Entry::Vacant(vacant) => {
                            vacant.insert(vec![insert_into_index]);
                        },
                    }
                }
            } else {
                // Add as a leaf to the root node.
                self.nodes[0].as_mut().expect("missing root")
                    .children.push(insert_into_index);
                parent = Some(0);
            }

            let children = element_children.remove(&element.name)
                .unwrap_or_default();

            for child in children.iter().copied() {
                self.nodes[child].as_mut().expect("missing node")
                    .parent = Some(insert_into_index);
            }

            self.elements_by_name.insert(element.name.clone(), insert_into_index);

            let node = Node {
                children,
                parent,
                ..Default::default()
            };

            match (self.nodes.get_mut(insert_into_index), self.elements.get_mut(insert_into_index)) {
                (Some(node_slot @ None), Some(element_slot @ None)) => {
                    *node_slot = Some(node);
                    *element_slot = Some(element);
                },
                (None, None) => {
                    self.nodes.push(Some(node));
                    self.elements.push(Some(element));
                },
                (Some(Some(_)), _) | (_, Some(Some(_))) => panic!("empty slot not empty"),
                (Some(_), None) | (None, Some(_)) => panic!("nodes and elements lists have different lengths"),
            }
        }

        for (element_name, children) in element_children {
            let children = children.into_iter()
                .map(|index| &self.elements[index].as_ref().expect("missing node").name)
                .fold(
                    String::new(),
                    |acc, string| format!("{acc}{string}, ")
                );

            warn!("Dangling references in gameplay widgets! Expected parent {element_name} with {children}");
            // todo: handle them properly instead of just warning
        }
    }

    /// Returns true if successful.
    /// Returns false if element does not exist or failure due to method.
    pub fn remove_element(&mut self, name: &str, method: Remove) -> bool {
        let Some(index) = self.elements_by_name.get(name).copied() else {
            return false;
        };

        let node = self.nodes[index].as_ref().expect("missing node");

        match method {
            Remove::OnlyLeaf => {
                if !node.children.is_empty() {
                    return false;
                }

                if let Some(parent) = node.parent {
                    let parent = self.nodes[parent].as_mut().expect("missing node");

                    let index = parent.children.iter().copied()
                        .position(|node| node == index)
                        .expect("parent node does not have child");

                    parent.children.swap_remove(index);
                }

                self.elements_by_name.remove(name);
                self.elements[index].take();
                self.nodes[index].take();

                self.empty_slots.push(index);

                true
            },
            Remove::WithChildren => unimplemented!("remove with children"),
        }
    }

    /// Returns true if successful.
    /// Returns false if element does not exist.
    pub fn mark_dirty(&mut self, name: &str) -> bool {
        let mut index = self.elements_by_name.get(name).copied();

        if index.is_none() {
            return false;
        };

        while let Some(i) = index {
            let current = self.nodes[i].as_mut().expect("missing node");

            current.dirty = true;

            index = current.parent;
        }

        true
    }

    pub fn layout(
        &mut self,
        playfield: Bounds,
        screen_size: Vector2,
    ) -> Result<(), GameplayWidgetLayoutError> {
        info!("{self:?}");

        let screen_bounds = Bounds::new(
            Vector2::ZERO,
            screen_size,
        );

        let iter = BreadthFirstIndex {
            nodes: &self.nodes,
            node_queue: self.nodes[0].as_ref().expect("missing root")
                .children.clone().into(),
        };

        for index in iter {
            let element = self.elements[index]
                .as_mut().expect("missing node");

            let layout = element.layout().clone();

            match layout.anchor {
                GameplayWidgetAnchor::Screen => {
                    element.resolved_pos = layout.align.resolve(
                        &screen_bounds,
                        element.preferred_size,
                        true,
                        true,
                    );
                },
                GameplayWidgetAnchor::Playfield { horizontal_side, vertical_side } => {
                    element.resolved_pos = layout.align.resolve(
                        &playfield,
                        element.preferred_size,
                        matches!(horizontal_side, Side::Inside),
                        matches!(vertical_side, Side::Inside),
                    );
                },
                // Because of the tree, these should be layed out already
                GameplayWidgetAnchor::Element {
                    element: anchored_to,
                    horizontal_side,
                    vertical_side,
                } => {
                    // Release borrow on this for one moment.
                    // In principle this is not needed unless the element is anchored
                    // to itself (which is invalid), but unsafe only saves us cloning a string.
                    let _ = element;

                    let anchored_bounds = {
                        let anchored_to = self.elements_by_name.get(&anchored_to).copied()
                            .expect("valid element anchor");

                        let anchored_to = self.elements[anchored_to].as_ref()
                            .expect("missing node");

                        Bounds::new(
                            anchored_to.resolved_pos,
                            anchored_to.preferred_size,
                        )
                    };

                    // Take it back
                    let element = self.elements[index].as_mut()
                        .expect("missing node");

                    element.resolved_pos = layout.align.resolve(
                        &anchored_bounds,
                        element.preferred_size,
                        matches!(horizontal_side, Side::Inside),
                        matches!(vertical_side, Side::Inside),
                    );
                },
            };
        }

        Ok(())
    }
}

/// Method of removal from the widget tree.
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Remove {
    /// Removes the node and all of its children.
    WithChildren,
    /// Removes only one node if it is a leaf.
    OnlyLeaf,
}

struct BreadthFirstIndex<'a> {
    nodes: &'a [Option<Node>],
    node_queue: VecDeque<usize>,
}

impl<'a> Iterator for BreadthFirstIndex<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node_queue.pop_front()?;

        let children = &self.nodes[node]
            .as_ref().expect("missing node")
            .children;

        self.node_queue.extend(children);

        Some(node)
    }
}

pub struct BreadthFirst<'a> {
    index: BreadthFirstIndex<'a>,
    elements: &'a [Option<GameplayWidgetContainer>],
}

impl<'a> Iterator for BreadthFirst<'a> {
    type Item = &'a GameplayWidgetContainer;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.index.next()?;

        let element = self.elements[node]
            .as_ref().expect("missing node");

        Some(element)
    }
}

pub struct BreadthFirstMut<'a> {
    index: BreadthFirstIndex<'a>,
    elements: &'a mut [Option<GameplayWidgetContainer>],
}

impl<'a> Iterator for BreadthFirstMut<'a> {
    type Item = &'a mut GameplayWidgetContainer;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.index.next()?;

        let element = self.elements[node]
            .as_mut().expect("missing node");

        // SAFETY: can never return two mutable references
        // to the same element at the same time.
        Some(unsafe { reborrow(element) })
    }
}

unsafe fn reborrow<'a, 'b, T: ?Sized>(value: &'a mut T) -> &'b mut T {
    unsafe {
        std::mem::transmute::<
            &'a mut T,
            &'b mut T,
        >(value)
    }
}
