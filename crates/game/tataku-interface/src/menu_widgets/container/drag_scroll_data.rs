use crate::prelude::*;

use tataku::Vector2;
use super::ScrollPosition;

use ui::{
    tree::*,
    widget::*,
};
use input::{ 
    InputType,
    InputEvent, 
    MouseButton, 
};

/// how far the
const DRAG_THRESHOLD:f32 = 5.0;

#[derive(Default)]
pub(super) struct DragScrollData {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub pressed_at: Vector2,
    pub did_move: bool,
}
impl DragScrollData {
    pub fn check_input(
        &mut self,
        node_id: NodeId,
        shell: &mut InputShell<actions::Action>,
        event: &InputEvent,
    ) -> ScrollPosition {
        let Some(bounds) = shell.tree.absolute_bounds(node_id)
        else { return ScrollPosition::None };

        let hover = bounds.contains(shell.mouse_pos);

        match event.event {
            InputType::MousePress(b) if hover => {
                match b {
                    MouseButton::Left if !self.right_pressed
                        => self.left_pressed = true,
                    MouseButton::Right if !self.left_pressed
                        => self.right_pressed = true,

                    _ => return ScrollPosition::None,
                }

                self.pressed_at = shell.mouse_pos;
            }

            InputType::MouseRelease(b) => {
                match b {
                    MouseButton::Left if self.left_pressed
                        => self.left_pressed = false,
                    MouseButton::Right if self.right_pressed
                        => self.right_pressed = false,
                    _ => return ScrollPosition::None
                }
                // if the mouse moved, we dont want to register the release key,
                // so return that it was consumed
                self.did_move = false;

                // FIXME: dont use this hack lmao
                return ScrollPosition::Relative(Vector2::ZERO);
            }
            InputType::MouseMove(position) if hover => {
                if !self.did_move
                    && (self.left_pressed || self.right_pressed)
                    && position.distance(self.pressed_at) > DRAG_THRESHOLD {
                    self.did_move = true;
                }

                // check left click
                if self.left_pressed && self.did_move {
                    // let diff = AbsoluteOffset {
                    //     x: -(position.x - self.pressed_at.x),
                    //     y: -(position.y - self.pressed_at.y),
                    // };
                    let diff = position - self.pressed_at;

                    // reset the clicked pos to move the delta
                    self.pressed_at = position;

                    // perform scroll
                    return ScrollPosition::Relative(diff)
                } else

                // check right click
                if self.right_pressed && self.did_move {
                    let pos = bounds.pos;
                    let size = bounds.size;

                    let move_to = ((position - pos) / size)
                        .clamp(Vector2::ZERO, Vector2::ONE);

                    return ScrollPosition::Absolute(move_to)
                    // let move_to = Vector2::new(
                    //     move_to.x.clamp(0.0, 1.0),
                    //     ((position.y - pos.y) / size.height).clamp(0.0, 1.0),
                    // );

                    // let mut operation = snap_to(RelativeOffset {
                    //     x: ((position.x - pos.x) / size.width).clamp(0.0, 1.0),
                    //     y: ((position.y - pos.y) / size.height).clamp(0.0, 1.0)
                    // });
                    // self.operate_on_children(tree, layout, renderer, &mut operation);
                }
            }

            _ => {}
        }

        ScrollPosition::None
    }

    // fn should_capture(&self) -> bool {
    //     (self.left_pressed || self.right_pressed) && self.did_move
    // }
}
