use super::scenes::Scene;

#[derive(Debug, Clone)]
pub enum Style {
    Fill(FillStyle),
    Stroke(StrokeStyle),
}

#[derive(Debug, Clone, Copy)]
pub enum FillStyle {
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone)]
pub struct StrokeStyle {
    pub width: f64,
    pub join: Join,
    pub miter_limit: f64,
    pub start_cap: Cap,
    pub end_cap: Cap,
    pub dash_pattern: Dashes,
    pub dash_offset: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Join {
    Bevel,
    Miter,
    Round,
}

#[derive(Debug, Clone, Copy)]
pub enum Cap {
    Butt,
    Square,
    Round,
}

pub type Dashes = Vec<[f64; 4]>;

#[derive(Debug, Clone, Copy)]
pub enum ImageFitMode {
    // Original size of the image buffer.
    Original,
    // Stretch the image to fill the shape.
    Fill,
    // Use exact width and height.
    Exact {
        width: f64,
        height: f64,
    },
}


#[derive(Debug, Clone, Copy)]
pub enum MixMode {
    Normal,
    Clip,
    Multiply,
}

#[derive(Debug, Clone, Copy)]
pub enum CompositeMode {
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
}
