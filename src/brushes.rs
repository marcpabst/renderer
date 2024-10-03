use std::sync::Arc;

use super::{colors::RGBA, shapes::Point};

#[derive(Debug, Clone)]
pub enum Brush<T> {
    Solid(RGBA),
    Gradient(Gradient),
    Texture(Arc<Image<T>>),
}

#[derive(Debug, Clone)]
pub struct Image<T> {
    pub data: T,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub extend: Extend,
    pub kind: GradientKind,
    pub stops: Vec<ColorStop>,
}

impl Gradient {
    pub fn new_equidistant(extend: Extend, kind: GradientKind, colors: &[RGBA]) -> Self {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, color)| ColorStop {
                offset: i as f32 / (colors.len() - 1) as f32,
                color: *color,
            })
            .collect();
        Self {
            extend,
            kind,
            stops,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Extend {
    /// Extends the image by repeating the edge color of the brush.
    Pad,
    /// Extends the image by repeating the brush.
    Repeat,
    /// Extends the image by reflecting the brush.
    Reflect,
}

#[derive(Debug, Clone)]
pub struct ColorStop {
    /// Normalized offset of the stop.
    pub offset: f32,
    /// Color at the specified offset.
    pub color: RGBA,
}

#[derive(Debug, Clone)]
pub enum GradientKind {
    /// Gradient that transitions between two or more colors along a line.
    Linear {
        /// Starting point.
        start: Point,
        /// Ending point.
        end: Point,
    },
    /// Gradient that transitions between two or more colors that radiate from an origin.
    Radial {
        /// Center of start circle.
        start_center: Point,
        /// Radius of start circle.
        start_radius: f32,
        /// Center of end circle.
        end_center: Point,
        /// Radius of end circle.
        end_radius: f32,
    },
    /// Gradient that transitions between two or more colors that rotate around a center
    /// point.
    Sweep {
        /// Center point.
        center: Point,
        /// Start angle of the sweep, counter-clockwise of the x-axis.
        start_angle: f32,
        /// End angle of the sweep, counter-clockwise of the x-axis.
        end_angle: f32,
    },
}