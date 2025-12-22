use crate::*;
use tataku::{ Bounds, Vector2 };
use gameplay::widgets::{
    GameplayWidgetAnchor,
    GameplayWidgetContainer,
    Side,
};

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct GameplayWidgetLayout {
    /// Where this element is anchored
    pub anchor: GameplayWidgetAnchor,

    /// How to align this element
    pub align: tataku::Alignment,

    /// Local transform of this widget relative to
    /// the anchor and alignment.
    pub transform: graphics::Transform,
}
impl GameplayWidgetLayout {
    pub const fn new(
        anchor: GameplayWidgetAnchor,
        align: tataku::Alignment,
        transform: graphics::Transform,
    ) -> Self {
        Self {
            anchor,
            align,
            transform,
        }
    }
}

#[derive(Debug)]
pub enum GameplayWidgetLayoutError {
    CyclicDependencyDetected,
    InvalidElementReference(String),
}

pub fn layout(
    elements: &mut [GameplayWidgetContainer],
    playfield: Bounds,
    screen_size: Vector2,
) -> Result<(), GameplayWidgetLayoutError> {
    // list of elements that have been layed out
    let mut layed_out = Vec::with_capacity(elements.len());
    let mut remainder = Vec::with_capacity(elements.len());

    let screen_bounds = Bounds::new(
        Vector2::ZERO,
        screen_size,
    );

    // lay out everything with a screen or absolute anchor
    for element in elements.iter_mut() {
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
