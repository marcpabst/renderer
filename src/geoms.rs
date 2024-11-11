use super::affine::Affine;
use super::brushes::{Brush, Image};
pub use super::scenes::Scene;
use super::shapes::{Point, Rectangle, Shape};
use super::styles::{FillStyle, ImageFitMode, Style};

// A geometric object that can be rendered, consisting of a shape and a brush.
#[derive(Debug, Clone)]
pub struct Geom<S: Shape> {
    pub style: Style,
    pub shape: S,
    pub brush: Brush,
    pub transform: Affine,
    pub brush_transform: Option<Affine>,
}

pub trait GeomTrait {
    fn new_image(
        image: Image,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        transform: Affine,
        fit_mode: ImageFitMode,
    ) -> Geom<Rectangle> {
        let shape = Rectangle {
            a: Point {
                x: x - width / 2.0,
                y: y - height / 2.0,
            },
            b: Point {
                x: x + width / 2.0,
                y: y + height / 2.0,
            },
        };

        let org_width = image.width as f64;
        let org_height = image.height as f64;

        let brush = Brush::Image(image);

        let brush_transform = match fit_mode {
            ImageFitMode::Original => None,
            ImageFitMode::Fill => Some(Affine::scale_xy(width / org_width, height / org_height)),
        };

        // Center the brush.
        let brush_transform =
            brush_transform.map(|t| t * Affine::translate(x - width / 2.0, y - height / 2.0));

        Geom {
            style: Style::Fill(FillStyle::NonZero),
            shape,
            brush,
            transform,
            brush_transform,
        }
    }
}

impl GeomTrait for Geom<Rectangle> {}
