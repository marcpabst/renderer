// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use std::hint::black_box;

extern crate simple;

use simple::renderer::{
    affine::Affine,
    brushes::{Brush, Extend, Gradient, GradientKind},
    colors::RGBA,
    shapes::{Circle, Point},
    text::{Alignment, Font, FontStyle, FormatedText, VerticalAlignment},
    vello_backend::*,
    CompositeMode, Drawable, FillStyle, Geom, Layer, LayerExt, MixMode, Scene, Style,
};
use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, DebugLayers, Renderer, RendererOptions};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

const ROBOTO_FONT: &[u8] = include_bytes!("./ba.ttf");

fn main() -> Result<()> {
    // benchmarking benchmark_function by running it 1 million times
    let sine_grating_colors = vec![
        RGBA::new(1.0, 0.0, 0.0, 1.0),
        RGBA::new(0.0, 1.0, 0.0, 1.0),
        RGBA::new(0.0, 0.0, 1.0, 1.0),
    ];

    let gaussian_colors = vec![
        RGBA::new(1.0, 0.0, 0.0, 1.0),
        RGBA::new(0.0, 1.0, 0.0, 1.0),
        RGBA::new(0.0, 0.0, 1.0, 1.0),
    ];

    let n = 10;
    let start = std::time::Instant::now();

    let mut num_draws = 0;

    for _ in 0..n {
        let out = black_box(benchmark_function(
            &sine_grating_colors,
            &gaussian_colors,
            0.0,
        ));
        // count the number of draws
        num_draws += 1;
    }

    let elapsed = start.elapsed();
    let per_iteration = elapsed / n as u32;

    println!("Number of draws: {}", num_draws);

    println!("Elapsed: {:?}, per iteration: {:?}", elapsed, per_iteration);

    Ok(())
}

fn benchmark_function(
    sine_grating_colors: &[RGBA],
    gaussian_colors: &[RGBA],
    i: f64,
) -> Vec<Box<dyn Drawable<VelloBackend>>> {
    let sine_grating = Geom {
        style: Style::Fill(FillStyle::NonZero),
        shape: Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 400.0,
        },
        brush: Brush::Gradient(Gradient::new_equidistant(
            Extend::Repeat,
            GradientKind::Linear {
                start: Point { x: 0.0, y: 0.0 },
                end: Point {
                    x: 100.0 + i,
                    y: 0.0,
                },
            },
            &sine_grating_colors,
        )),
        transform: Affine::identity(),
        brush_transform: Some(Affine::translate(0.0, 0.0)),
    };

    let gaussian = Geom {
        style: Style::Fill(FillStyle::NonZero),
        shape: Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 301.0,
        },
        brush: Brush::Gradient(Gradient::new_equidistant(
            Extend::Pad,
            GradientKind::Radial {
                start_center: Point { x: 0.0, y: 0.0 },
                start_radius: 0.0,
                end_center: Point { x: 0.0, y: 0.0 },
                end_radius: 300.0,
            },
            &gaussian_colors,
        )),
        transform: Affine::identity(),
        brush_transform: None,
    };

    let clip = Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 301.0,
    };

    // let mut item_layer = Layer::new(
    //     MixMode::Normal,
    //     CompositeMode::SourceOver,
    //     clip.clone(),
    //     Affine::identity(),
    // );
    // let mut mask_layer = Layer::new(
    //     MixMode::Multiply,
    //     CompositeMode::SourceIn,
    //     clip.clone(),
    //     Affine::identity(),
    // );
    // item_layer.add_child(sine_grating);
    // mask_layer.add_child(gaussian);
    // item_layer.add_child(mask_layer);

    vec![Box::new(gaussian), Box::new(sine_grating)]
}
