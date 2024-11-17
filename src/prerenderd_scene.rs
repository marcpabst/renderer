use crate::affine::Affine;
use crate::Drawable;
use crate::vello_backend::VelloBackend;

#[derive(Clone)]
pub struct PrerenderedScene {
    pub scene: vello::Scene,
    pub transform: Affine,
}

impl PrerenderedScene {
    pub fn new(scene: vello::Scene, transform: Affine) -> Self {
        Self { scene, transform }
    }

    pub fn from_svg_string(svg: & str, transform: Affine) -> Self {
        let scene = vello_svg::render(svg).unwrap();
        Self::new(scene, transform)
    }
}