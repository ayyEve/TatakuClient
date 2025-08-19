use crate::prelude::*;

pub struct SpatialNagivation<'a, Action: Send + Sync> {
    tree: &'a mut Tree<Action>,
    rects: HashMap<taffy::NodeId, NodeInfo>,
}
impl<'a, Action: Send + Sync + 'static> SpatialNagivation<'a, Action> {
    pub fn new(tree: &'a mut Tree<Action>) -> Self {
        Self {
            tree,
            rects: HashMap::new(),
        }
    }
    fn remove_target(list: &mut Vec<taffy::NodeId>, target: taffy::NodeId) {
        // remove target from candidates
        let mut existing = None;
        for (n, i) in list.iter().enumerate() {
            if i == &target {
                existing = Some(n);
                break;
            }
        }
        if let Some(index) = existing {
            list.remove(index);
        }
    }
}

impl<Action: Send + Sync + 'static> SpatialNagivation<'_, Action> {
    /// https://developer.mozilla.org/en-US/docs/Web/API/Element/getBoundingClientRect
    fn get_bounding_client_rect(&self, node: taffy::NodeId) -> Option<BoundingBox> {
        let layout = self.tree.get_layout(node)?;
        let c = self.tree.get_context(node)?;

        // TODO: this is likely going to be incorrect for containers that can scroll
        let viewport = c.absolute_bounds;

        let node_size = layout.content_box_size();
        let node_pos = viewport.pos + Vector2::new(
            layout.location.x,
            layout.location.y
        );

        Some(BoundingBox {
            x: node_pos.x,
            y: node_pos.y,

            top: node_pos.x,
            left: node_pos.y,
            bottom: node_pos.y + node_size.height,
            right: node_pos.x + node_size.width,

            width: node_size.width,
            height: node_size.height,
        })
    }

    fn get_rect(&mut self, node: taffy::NodeId) -> Option<NodeInfo> {
        if let Some(info) = self.rects.get(&node) {
            return Some(*info)
        }

        let cr = self.get_bounding_client_rect(node)?;

        let center = Vector2::new(
            cr.left + (cr.width / 2.0).floor(),
            cr.top + (cr.height / 2.0).floor(),
        );
        
        let rect = NodeInfo {
            bounding_box: cr,
            element: node,
            center: BoundingBox {
                x: center.x,
                y: center.y,
                left: center.x,
                right: center.x,

                top: center.y,
                bottom: center.y,

                // the js code doesnt set these?
                width: 0.0,
                height: 0.0
            }
        };

        self.rects.insert(node, rect);
        Some(rect)
    } 


    fn partition(
        &self, 
        rects: &[NodeInfo],
        target_rect: BoundingBox,
        straight_overlap_threshold: f32,
    ) -> [Vec<NodeInfo>; 9] {
        let mut groups = [
            Vec::new(), Vec::new(), Vec::new(),
            Vec::new(), Vec::new(), Vec::new(),
            Vec::new(), Vec::new(), Vec::new(),
        ];

        for rect in rects.iter().copied() {
            let center = &rect.center;
            let x;
            let y;

            if center.x < target_rect.left {
                x = 0;
            } else if center.x <= target_rect.right {
                x = 1;
            } else {
                x = 2;
            }

            if center.y < target_rect.top {
                y = 0;
            } else if center.y <= target_rect.bottom {
                y = 1;
            } else {
                y = 2;
            }

            let group_id = y * 3 + x;
            groups[group_id].push(rect);

            if ![0, 2, 6, 8].contains(&group_id) { continue }

            let threshold = straight_overlap_threshold;
    
            if rect.left <= target_rect.right - target_rect.width * threshold {
                if group_id == 2 {
                    groups[1].push(rect);
                } else if group_id == 8 {
                    groups[7].push(rect);
                }
            }
    
            if rect.right >= target_rect.left + target_rect.width * threshold {
                if group_id == 0 {
                    groups[1].push(rect);
                } else if group_id == 6 {
                    groups[7].push(rect);
                }
            }
    
            if rect.top <= target_rect.bottom - target_rect.height * threshold {
                if group_id == 6 {
                    groups[3].push(rect);
                } else if group_id == 8 {
                    groups[5].push(rect);
                }
            }
    
            if rect.bottom >= target_rect.top + target_rect.height * threshold {
                if group_id == 0 {
                    groups[3].push(rect);
                } else if group_id == 2 {
                    groups[5].push(rect);
                }
            }
        
        }

        groups
    }


    fn prioritize(
        &self,
        priorities: &[Priority<'_>],
    ) -> Option<Vec<NodeInfo>> {
        let dest_priority = priorities
            .iter()
            .find(|i| 
                i.group.iter().any(|i| !i.is_empty())
            )?;

        let mut group = dest_priority.group
            .iter()
            .flat_map(|i| i.iter())
            .copied()
            .collect::<Vec<_>>();

        if group.is_empty() {
            error!("group empty!");
            return None
        }
        
        let dest_distance = &dest_priority.distance;

        group.sort_by(|a, b| {
            use std::cmp::Ordering::*;
            for distance in dest_distance.iter() {
                let delta = (distance)(a) - (distance)(b);
                
                return match delta {
                    0.0.. => Greater,
                    ..0.0 => Less,
                    _ => continue,
                }
            }

            Equal
        });

        Some(group)
    }


    fn navigate(
        &mut self,
        target: taffy::NodeId,
        direction: Direction,
        candidates: &[taffy::NodeId],
        config: &NavigateConfig
    ) -> Option<taffy::NodeId> {
        if candidates.is_empty() { 
            error!("candidates empty!");
            return None 
        }

        let rects = candidates
            .iter()
            .copied()
            .filter_map(|c| self.get_rect(c))
            .collect::<Vec<_>>();

        if rects.is_empty() { 
            error!("rects empty!");
            return None 
        }

        let target_rect = self.get_rect(target)?;
        let distance_function = DistanceFunctions::generate(target_rect);

        let groups = self.partition(
            &rects,
            target_rect.bounding_box,
            config.straight_overlap_threshold
        );

        let internal_groups = self.partition(
            &groups[4],
            target_rect.center,
            config.straight_overlap_threshold
        );
        
        let mut priorities = match direction {
            Direction::Left => vec![
                Priority {
                    group: vec![ 
                        &internal_groups[0], 
                        &internal_groups[3], 
                        &internal_groups[6]
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.top,
                    ]
                },
                Priority {
                    group: vec![ 
                        &groups[3] 
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.top,
                    ]
                },
                Priority {
                    group: vec![
                        &groups[0],
                        &groups[6],
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.right,
                        &distance_function.near_target_top,
                    ]
                },
            ],

            Direction::Right => vec![
                Priority {
                    group: vec![
                        &internal_groups[2],
                        &internal_groups[5],
                        &internal_groups[8],
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.top
                    ]
                },
                Priority {
                    group: vec![
                        &groups[5]
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.top
                    ]
                },
                Priority {
                    group: vec![
                        &groups[2],
                        &groups[8],
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.left,
                        &distance_function.near_target_top
                    ]
                }
            ],

            Direction::Up => vec![
                Priority {
                    group: vec![ 
                        &internal_groups[0], 
                        &internal_groups[1], 
                        &internal_groups[2],
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.left,
                    ]
                },
                Priority {
                    group: vec![ 
                        &groups[1], 
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.left,
                    ]
                },
                Priority {
                    group: vec![
                        &groups[0],
                        &groups[2],
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.bottom,
                        &distance_function.near_target_left,
                    ]
                },
            ],

            Direction::Down => vec![
                Priority {
                    group: vec![ 
                        &internal_groups[6], 
                        &internal_groups[7], 
                        &internal_groups[8],
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.left,
                    ]
                },
                Priority {
                    group: vec![ 
                        &groups[7], 
                    ],
                    distance: vec![
                        &distance_function.near_horizon,
                        &distance_function.left,
                    ]
                },
                Priority {
                    group: vec![
                        &groups[6],
                        &groups[8],
                    ],
                    distance: vec![
                        &distance_function.near_plumbline,
                        &distance_function.top,
                        &distance_function.near_target_left,
                    ]
                },
            ],
        };


        if config.straight_only { priorities.pop(); }

        let dest_group = self.prioritize(&priorities)?;

        let mut dest = None;
        if let Some(previous) = config.previous
            .as_ref()
            .filter(|p| {
                config.remember_source
                && p.destination == target
                && p.reverse == direction
            }) 
        {
            dest = dest_group
                .iter()
                .find(|g| g.element == previous.element)
                .map(|g| g.element);
        }

        
        Some(
            dest.unwrap_or(dest_group[0].element)
        )
    }

    pub fn run(&mut self, config: &NavigateConfig) {
        // debug!("starting navigation");
        let all_selectable = self.tree.all_children()
            .filter(|i| self.tree.get_context(*i).unwrap().selectable())
            .map(|i| i.get_id())
            .collect::<Vec<_>>();

        let selectable = all_selectable.clone();
        
        for i in selectable {
            let mut candidates = all_selectable.clone();
            Self::remove_target(&mut candidates, i);

            for dir in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let node = self.navigate(
                    i, 
                    dir, 
                    &candidates, 
                    config
                );

                let context = self.tree.get_context_mut(i).unwrap();
                context.set_node_direction(dir, node);
            }
            // debug!("got adjacent nodes for node {:?}: above: {above:?} | below: {below:?} | left: {left:?} | right: {right:?}", context.element_data);

        }

    }
}


#[derive(Copy, Clone)]
struct NodeInfo {
    bounding_box: BoundingBox,
    element: taffy::NodeId,
    center: BoundingBox,
}
impl Deref for NodeInfo {
    type Target = BoundingBox;
    fn deref(&self) -> &Self::Target {
        &self.bounding_box
    }
}


#[derive(Copy, Clone, Debug)]
struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,

    top: f32,
    left: f32,
    bottom: f32,
    right: f32,
}

#[derive(Default2)]
pub struct NavigateConfig {
    #[default(0.5)]
    straight_overlap_threshold: f32,
    straight_only: bool,

    remember_source: bool,
    previous: Option<ConfigPrevious>
}

#[allow(unused)]
struct ConfigPrevious {
    target: taffy::NodeId,
    element: taffy::NodeId,
    destination: taffy::NodeId,
    reverse: Direction,
}

type DistanceFunction = Box<dyn Fn(&NodeInfo) -> f32>;
struct DistanceFunctions {
    near_plumbline: DistanceFunction,
    near_horizon: DistanceFunction,
    near_target_left: DistanceFunction,
    near_target_top: DistanceFunction,

    top: DistanceFunction,
    bottom: DistanceFunction,
    left: DistanceFunction,
    right: DistanceFunction,
}
impl DistanceFunctions {
    fn generate(target_rect: NodeInfo) -> Self {
        Self {
            near_plumbline: Box::new(move |rect| 
                if rect.center.x < target_rect.center.x {
                    target_rect.center.x - rect.right
                } else {
                    rect.left - target_rect.center.x
                }.max(0.0)
            ),

            near_horizon: Box::new(move |rect| 
                if rect.center.y < target_rect.center.y {
                    target_rect.center.y - rect.bottom
                } else {
                    rect.top - target_rect.center.y
                }.max(0.0)
            ),

            near_target_left: Box::new(move |rect| 
                if rect.center.x < target_rect.center.x {
                    target_rect.left - rect.right
                } else {
                    rect.left - target_rect.left
                }.max(0.0)
            ),

            near_target_top: Box::new(move |rect| 
                if rect.center.y < target_rect.center.y {
                    target_rect.top - rect.bottom
                } else {
                    rect.top - target_rect.top
                }.max(0.0)
            ),

            top: Box::new(move |r| r.top),
            left: Box::new(move |r| r.left),
            bottom: Box::new(move |r| r.bottom),
            right: Box::new(move |r| r.right),
        }
    }
}


struct Priority<'a> {
    group: Vec<&'a Vec<NodeInfo>>,
    distance: Vec<&'a DistanceFunction>
}
