use super::content::ButtonIcon;
use crate::style::button::SlicedImage;
use bevy::{ecs::system::SystemParam, prelude::*};

/// Draws or removes the image surface of a button.
#[derive(SystemParam)]
pub(super) struct ButtonSurface<'w, 's> {
    commands: Commands<'w, 's>,
    images: Query<'w, 's, &'static mut ImageNode, Without<ButtonIcon>>,
}

impl ButtonSurface<'_, '_> {
    /// `ImageNode` exists only while the style has a surface, because a default one paints a white box.
    pub(super) fn apply(&mut self, button: Entity, surface: Option<&SlicedImage>, tint: Color) {
        match (surface, self.images.get_mut(button)) {
            (Some(surface), Ok(mut node)) => {
                if node.color != tint {
                    node.color = tint;
                }
                if node.image != surface.image {
                    node.image = surface.image.clone();
                }
                let mode = NodeImageMode::Sliced(surface.slicer.clone());
                if node.image_mode != mode {
                    node.image_mode = mode;
                }
                if node.visual_box != VisualBox::BorderBox {
                    node.visual_box = VisualBox::BorderBox;
                }
            }
            (Some(surface), Err(_)) => {
                self.commands.entity(button).try_insert(ImageNode {
                    image: surface.image.clone(),
                    color: tint,
                    image_mode: NodeImageMode::Sliced(surface.slicer.clone()),
                    // `ImageNode::default()`'s visual_box paints inside the padding (shrinks the frame to the label). Use this to avoid that.
                    visual_box: VisualBox::BorderBox,
                    ..default()
                });
            }
            (None, Ok(_)) => {
                self.commands.entity(button).try_remove::<ImageNode>();
            }
            (None, Err(_)) => {}
        }
    }
}
