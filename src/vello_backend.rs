use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use image::GenericImageView;
use vello::peniko::BlendMode;
use vello::{DebugLayers, RendererOptions};

use crate::brushes::Extend;

use crate::geoms::Geom;
use crate::shapes::Shape;
use crate::styles::{CompositeMode, FillStyle, MixMode, Style};
use crate::{affine::Affine, scenes::Scene, Drawable};

use super::brushes::{Gradient, GradientKind, Image};
use super::scenes::SceneTrait;
use super::text::{Alignment, FormatedText, VerticalAlignment};

use super::{
    brushes::{Brush, ColorStop},
    colors::RGBA,
    shapes::{Circle, Point, Rectangle, RoundedRectangle},
};

#[derive(Clone)]
pub struct VelloBackend {
    pub vello_scene: vello::Scene,
    pub global_transform: Affine,
}

pub struct VelloRenderer {
    pub renderer: vello::Renderer,
}

impl VelloRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let renderer = vello::Renderer::new(
            &device,
            RendererOptions {
                surface_format: Some(surface_format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: std::num::NonZeroUsize::new(1),
            },
        )
        .unwrap();
        Self { renderer }
    }

    pub fn render_to_surface(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::SurfaceTexture,
        scene: &Scene<VelloBackend>,
    ) {
        let vello_scene = &scene.backend.vello_scene;
        let render_params = vello::RenderParams {
            base_color: scene.background_color.into(),
            width: surface.texture.width(),
            height: surface.texture.height(),
            antialiasing_method: vello::AaConfig::Msaa16,
            debug: DebugLayers::none(),
        };
        self.renderer
            .render_to_surface(device, queue, vello_scene, surface, &render_params);
    }
}

impl VelloBackend {
    /// Create a new Vello backend.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            vello_scene: vello::Scene::new(),
            global_transform: Affine::translate(width as f64 / 2.0, height as f64 / 2.0),
        }
    }
}

impl Scene<VelloBackend> {
    /// Create a new scene.
    pub fn new(background_color: RGBA, width: u32, height: u32) -> Self {
        Self {
            background_color,
            width,
            height,
            backend: VelloBackend::new(width, height),
        }
    }

    /// draw a renderable object.
    pub fn draw(&mut self, mut object: impl Drawable<VelloBackend>) {
        // Draw the object.
        object.draw(self);
    }
}

impl Image<vello::peniko::Image> {
    pub fn new(image: image::DynamicImage) -> Self {
        let rgba = image.to_rgba8();
        let data = rgba.into_raw();
        let width = image.width();
        let height = image.height();
        let blob = vello::peniko::Blob::new(Arc::new(data));
        let image = vello::peniko::Image::new(blob, vello::peniko::Format::Rgba8, width, height);
        Self {
            data: image,
            width,
            height,
        }
    }
}

impl<S: IntoVelloShape + Shape> Drawable<VelloBackend> for Geom<S, vello::peniko::Image> {
    fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
        let transform = (scene.backend.global_transform * self.transform).into();

        let brush_transform = self.brush_transform.map(|t| t.into());

        // convert the brush

        // if image, use brushref
        let brush = self.brush.as_brush_or_brushref();

        let shape = &self.shape.clone().into_vello_shape();
        // match the style (stroke or fill)

        match self.style {
            Style::Fill(style) => {
                // fill the shape
                scene.backend.vello_scene.fill(
                    style.into(),
                    transform,
                    &brush,
                    brush_transform,
                    &shape,
                );
            }
            Style::Stroke(_) => {
                todo!();
            }
        }
    }
}

impl<ClipShape: IntoVelloShape + Shape> SceneTrait<VelloBackend, ClipShape>
    for Scene<VelloBackend>
{
    fn scene_mut(&mut self) -> &mut Scene<VelloBackend> {
        self
    }

    fn scene(&self) -> &Scene<VelloBackend> {
        self
    }

    fn start_layer(
        &mut self,
        mix_mode: MixMode,
        composite_mode: CompositeMode,
        clip: ClipShape,
        clip_transform: Affine,
        layer_transform: Option<Affine>,
        alpha: f32,
    ) {
        // error if a layer transform is provided
        if layer_transform.is_some() {
            todo!();
        }
        let clip_shape = clip.into_vello_shape();
        let global_transform = self.backend.global_transform;
        let clip_transform = (global_transform * clip_transform).into();

        self.backend.vello_scene.push_layer(
            BlendMode::new(mix_mode.into(), composite_mode.into()),
            alpha,
            clip_transform,
            &clip_shape,
        );
    }

    fn end_layer(&mut self) {
        self.backend.vello_scene.pop_layer();
    }
}

// allow converting different types into the vello types

// Point2D
impl From<Point> for vello::kurbo::Point {
    fn from(point: Point) -> Self {
        vello::kurbo::Point::new(point.x, point.y)
    }
}

// Affine
impl From<Affine> for vello::kurbo::Affine {
    fn from(affine: Affine) -> Self {
        vello::kurbo::Affine::new(affine.0)
    }
}

// FillStyle
impl From<FillStyle> for vello::peniko::Fill {
    fn from(style: FillStyle) -> Self {
        match style {
            FillStyle::NonZero => vello::peniko::Fill::NonZero,

            FillStyle::EvenOdd => vello::peniko::Fill::EvenOdd,
        }
    }
}

// BrushRef (this needs to be refactored)
impl<'a> Brush<vello::peniko::Image> {
    fn as_brush_or_brushref(&'a self) -> VelloBrushOrBrushRef<'a> {
        match self {
            Brush::Texture(image) => {
                VelloBrushOrBrushRef::BrushRef(vello::peniko::BrushRef::Image(&image.data))
            }
            Brush::Solid(rgba) => {
                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Solid(rgba.clone().into()))
            }
            Brush::Gradient(gradient) => {
                VelloBrushOrBrushRef::Brush(vello::peniko::Brush::Gradient(gradient.clone().into()))
            }
        }
    }
}

pub enum VelloBrushOrBrushRef<'a> {
    Brush(vello::peniko::Brush),
    BrushRef(vello::peniko::BrushRef<'a>),
}

// allow VelloBrushOrBrushRef to BrushRef
impl<'a> From<&'a VelloBrushOrBrushRef<'a>> for vello::peniko::BrushRef<'a> {
    fn from(brush: &'a VelloBrushOrBrushRef<'a>) -> Self {
        match brush {
            VelloBrushOrBrushRef::BrushRef(brush_ref) => brush_ref.clone(),
            VelloBrushOrBrushRef::Brush(brush) => brush.into(),
        }
    }
}

// implement vello Shape trait for different shapes
trait IntoVelloShape {
    type VelloShape: vello::kurbo::Shape;
    fn into_vello_shape(self) -> Self::VelloShape;
}

// rectangle
impl IntoVelloShape for Rectangle {
    type VelloShape = vello::kurbo::Rect;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::Rect::new(self.a.x, self.a.y, self.b.x, self.b.y)
    }
}

// rounded rectangle
impl IntoVelloShape for RoundedRectangle {
    type VelloShape = vello::kurbo::RoundedRect;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::RoundedRect::new(self.a.x, self.a.y, self.b.x, self.b.y, self.radius)
    }
}

// circle
impl IntoVelloShape for Circle {
    type VelloShape = vello::kurbo::Circle;
    fn into_vello_shape(self) -> Self::VelloShape {
        vello::kurbo::Circle::new(self.center, self.radius)
    }
}

// Colors
impl From<RGBA> for vello::peniko::Color {
    fn from(color: RGBA) -> Self {
        vello::peniko::Color::rgba(
            color.r as f64,
            color.g as f64,
            color.b as f64,
            color.a as f64,
        )
    }
}

// MixMode
impl From<MixMode> for vello::peniko::Mix {
    fn from(mode: MixMode) -> Self {
        match mode {
            MixMode::Normal => vello::peniko::Mix::Normal,
            MixMode::Clip => vello::peniko::Mix::Clip,
            MixMode::Multiply => vello::peniko::Mix::Multiply,
        }
    }
}

// CompositeMode
impl From<CompositeMode> for vello::peniko::Compose {
    fn from(mode: CompositeMode) -> Self {
        match mode {
            CompositeMode::SourceIn => vello::peniko::Compose::SrcIn,
            CompositeMode::SourceOut => vello::peniko::Compose::SrcOut,
            CompositeMode::SourceOver => vello::peniko::Compose::SrcOver,
            CompositeMode::DestinationOver => vello::peniko::Compose::DestOver,
            CompositeMode::DestinationIn => vello::peniko::Compose::DestIn,
            CompositeMode::DestinationOut => vello::peniko::Compose::DestOut,
            CompositeMode::DestinationAtop => vello::peniko::Compose::DestAtop,
            CompositeMode::Xor => vello::peniko::Compose::Xor,
            CompositeMode::SourceAtop => vello::peniko::Compose::SrcAtop,
            CompositeMode::Lighter => vello::peniko::Compose::PlusLighter,
            CompositeMode::Copy => vello::peniko::Compose::Copy,
        }
    }
}

// ColorStop
impl From<ColorStop> for vello::peniko::ColorStop {
    fn from(color_stop: ColorStop) -> Self {
        vello::peniko::ColorStop {
            offset: color_stop.offset,
            color: color_stop.color.into(),
        }
    }
}

// Extend
impl From<Extend> for vello::peniko::Extend {
    fn from(extend: Extend) -> Self {
        match extend {
            Extend::Pad => vello::peniko::Extend::Pad,
            Extend::Repeat => vello::peniko::Extend::Repeat,
            Extend::Reflect => vello::peniko::Extend::Reflect,
        }
    }
}

// GradientKind
impl From<GradientKind> for vello::peniko::GradientKind {
    fn from(kind: GradientKind) -> Self {
        match kind {
            GradientKind::Linear { start, end } => vello::peniko::GradientKind::Linear {
                start: start.into(),
                end: end.into(),
            },
            GradientKind::Radial {
                start_center,
                start_radius,
                end_center,
                end_radius,
            } => vello::peniko::GradientKind::Radial {
                start_center: start_center.into(),
                start_radius,
                end_center: end_center.into(),
                end_radius,
            },
            GradientKind::Sweep {
                center,
                start_angle,
                end_angle,
            } => vello::peniko::GradientKind::Sweep {
                center: center.into(),
                start_angle,
                end_angle,
            },
        }
    }
}

// Gradient
impl From<Gradient> for vello::peniko::Gradient {
    fn from(gradient: Gradient) -> Self {
        vello::peniko::Gradient {
            kind: gradient.kind.into(),
            stops: gradient.stops.into_iter().map(|stop| stop.into()).collect(),
            extend: gradient.extend.into(),
        }
    }
}

// Text
#[derive(Debug, Clone)]
pub struct VelloFont(vello::peniko::Font);

impl VelloFont {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let blob = vello::peniko::Blob::new(Arc::new(bytes.to_vec()));
        let font = vello::peniko::Font::new(blob, 0);

        Self(font)
    }
}

impl Drawable<VelloBackend> for FormatedText<VelloFont> {
    fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
        let transform: vello::kurbo::Affine =
            (scene.backend.global_transform * self.transform).into();

        let font = &self.font.0;
        let font_size = vello::skrifa::instance::Size::new(self.size);
        let text = &self.text;

        let font_ref = vello_font_to_font_ref(font).expect("Failed to load font");
        let axes = vello::skrifa::MetadataProvider::axes(&font_ref);
        let variations = [("wght", 100.0), ("wdth", 500.0)];
        let var_loc = axes.location(variations.iter().copied());

        let charmap = vello::skrifa::MetadataProvider::charmap(&font_ref);
        let metrics = vello::skrifa::MetadataProvider::metrics(&font_ref, font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics =
            vello::skrifa::MetadataProvider::glyph_metrics(&font_ref, font_size, &var_loc);

        let mut pen_x = (self.x * 2.0) as f32;
        let mut pen_y = (self.y * 2.0) as f32;

        let brush_color: vello::peniko::Color = self.color.into();

        let glyphs = text
            .chars()
            .filter_map(|ch| {
                if ch == '\n' {
                    pen_y += line_height;
                    pen_x = 0.0;
                    return None;
                }
                let gid = charmap.map(ch).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;
                Some(vello::glyph::Glyph {
                    id: gid.to_u16() as u32,
                    x,
                    y: pen_y,
                })
            })
            .collect::<Vec<_>>();

        let text_width = pen_x as f64;
        let text_height = pen_y as f64 + line_height as f64;

        let transform_x = match self.alignment {
            Alignment::Left => 0.0,
            Alignment::Center => -text_width / 2.0,
            Alignment::Right => -text_width,
        };

        let transform_y = match self.vertical_alignment {
            VerticalAlignment::Top => 0.0,
            VerticalAlignment::Middle => text_height / 2.0,
            VerticalAlignment::Bottom => text_height,
        };

        let transform = transform.pre_translate(vello::kurbo::Vec2::new(transform_x, transform_y));

        scene
            .backend
            .vello_scene
            .draw_glyphs(font)
            .font_size(self.size)
            .transform(transform)
            .glyph_transform(self.glyph_transform.map(|t| t.into()))
            .normalized_coords(var_loc.coords())
            .brush(brush_color)
            .hint(false)
            .draw(vello::peniko::Fill::NonZero, glyphs.into_iter());
    }
}

fn vello_font_to_font_ref(font: &vello::peniko::Font) -> Option<vello::skrifa::FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

// #[derive(Clone)]
// pub struct LottieFile {
//     data: velato::Composition,
//     renderer: Rc<RefCell<velato::Renderer>>,
//     start_time: std::time::Instant,
//     transform: Affine,
// }

// impl LottieFile {
//     pub fn new(filename: &str, transform: Affine) -> Self {
//         // load file as string
//         let v = std::fs::read_to_string(filename).expect("Failed to read file");
//         let data = velato::Composition::from_str(&v).expect("Failed to parse file");
//         let renderer = velato::Renderer::new();
//         Self {
//             data,
//             renderer: Rc::new(RefCell::new(renderer)),
//             start_time: std::time::Instant::now(),
//             transform,
//         }
//     }
// }

// impl Drawable<VelloBackend> for LottieFile {
//     fn draw(&mut self, scene: &mut Scene<VelloBackend>) {
//         let animation = &self.data;
//         let elapsed = self.start_time.elapsed().as_secs_f64();
//         let frame = self.data.frame_rate * elapsed;

//         let transform = (scene.backend.global_transform * self.transform).into();

//         self.renderer.borrow_mut().append(
//             animation,
//             frame,
//             transform,
//             1.0,
//             &mut scene.backend.vello_scene,
//         );
//     }
// }