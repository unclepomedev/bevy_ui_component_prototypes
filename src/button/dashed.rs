use super::style::SlicedImage;
use bevy::{
    asset::{Handle, RenderAssetUsages},
    image::{Image, ImageSampler},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// A white frame made of dashes, as an image for [`SlicedImage::tiled`].
///
/// A button style draws it as its `surface`, so the frame is tinted by `bg` per state and is
/// stretched to any size of button while its dashes stay as they are. The image is made here,
/// so no art file is needed.
///
/// It is a square of three corners a side. One edge piece is one period of the dashes: a dash of
/// [`dash`](Self::dash) pixels and a gap of the rest. The last gap of an edge is filled, so
/// each corner closes. Draw it with a nearest sampler, which [`image`](Self::image) sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DashedFrame {
    /// The side of a corner, and the period of the dashes, in pixels. At least 2.
    pub corner: u32,
    /// How thick the line is. At least 1, at most `corner`.
    pub thickness: u32,
    /// How long a dash is. At least 1, less than `corner`.
    pub dash: u32,
}

impl Default for DashedFrame {
    /// Dashes of 4 pixels and gaps of 2, a line of 2 pixels.
    fn default() -> Self {
        Self {
            corner: 6,
            thickness: 2,
            dash: 4,
        }
    }
}

impl DashedFrame {
    /// The sizes brought into the ranges above.
    fn clamped(self) -> Self {
        let corner = self.corner.max(2);
        Self {
            corner,
            thickness: self.thickness.clamp(1, corner),
            dash: self.dash.clamp(1, corner - 1),
        }
    }

    /// Whether the pixel (`x`, `y`) of the image is part of the line.
    fn is_line(&self, x: u32, y: u32) -> bool {
        let side = self.corner * 3;
        let last = side - 1;
        // A position along an edge is a dash if it is in the first `dash` of its period, or in the
        // last gap, which closes the corner there.
        let dashed = |along: u32| {
            along % self.corner < self.dash || along > last - (self.corner - self.dash)
        };
        let on_horizontal = y.min(last - y) < self.thickness; // top or bottom edge: runs along x
        let on_vertical = x.min(last - x) < self.thickness; // left or right edge: runs along y
        (on_horizontal && dashed(x)) || (on_vertical && dashed(y))
    }

    /// The image, with nearest sampling.
    pub fn image(self) -> Image {
        let frame = self.clamped();
        let side = frame.corner * 3;
        let mut data = Vec::with_capacity((side * side * 4) as usize);
        for y in 0..side {
            for x in 0..side {
                let alpha = if frame.is_line(x, y) { 255 } else { 0 };
                data.extend_from_slice(&[255, 255, 255, alpha]);
            }
        }
        let mut image = Image::new(
            Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        image
    }

    /// `image`, once added to `Assets<Image>`, as a button surface.
    pub fn sliced(self, image: Handle<Image>) -> SlicedImage {
        SlicedImage::tiled(image, self.clamped().corner as f32)
    }
}

// ============================================================================================
// UNIT TESTS
// ============================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn picture(frame: DashedFrame) -> Vec<String> {
        let image = frame.image();
        let side = image.width() as usize;
        let data = image.data.as_ref().expect("the image has its pixels");
        (0..side)
            .map(|y| {
                (0..side)
                    .map(|x| {
                        if data[(y * side + x) * 4 + 3] > 0 {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn the_default_frame_is_dashes_of_four_and_gaps_of_two() {
        let expected = [
            "####..####..######",
            "####..####..######",
            "##..............##",
            "##..............##",
            "..................",
            "..................",
            "##..............##",
            "##..............##",
            "##..............##",
            "##..............##",
            "..................",
            "..................",
            "##..............##",
            "##..............##",
            "##..............##",
            "##..............##",
            "####..####..######",
            "####..####..######",
        ];
        assert_eq!(picture(DashedFrame::default()), expected);
    }

    #[test]
    fn the_image_is_three_corners_square() {
        let image = DashedFrame {
            corner: 8,
            thickness: 1,
            dash: 5,
        }
        .image();
        assert_eq!((image.width(), image.height()), (24, 24));
    }

    #[test]
    fn every_corner_is_closed() {
        let frame = DashedFrame {
            corner: 8,
            thickness: 2,
            dash: 5,
        };
        let side = frame.corner * 3;
        for (x, y) in [(0, 0), (side - 1, 0), (0, side - 1), (side - 1, side - 1)] {
            assert!(frame.is_line(x, y), "the corner pixel ({x}, {y}) is empty");
        }
    }

    #[test]
    fn sizes_out_of_range_are_brought_in() {
        let frame = DashedFrame {
            corner: 0,
            thickness: 0,
            dash: 99,
        };
        let image = frame.image();
        assert_eq!(image.width(), 6); // a corner of 2
        assert_eq!(
            frame.sliced(Handle::default()).slicer.border,
            bevy::prelude::BorderRect::all(2.0)
        );
    }
}
