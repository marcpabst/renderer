// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate renderer;

use anyhow::Result;

use renderer::prelude::*;
use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::{
    util::{RenderContext, RenderSurface},
    DebugLayers,
};
use vello::{AaConfig, Renderer, RendererOptions};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

const ROBOTO_FONT: &[u8] = include_bytes!("./ba.ttf");

use wgpu;

// Simple struct to hold the state of the renderer
pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<Window>,
    i: u32,
}

enum RenderState<'s> {
    Active(ActiveRenderState<'s>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<Window>>),
}

struct SimpleVelloApp<'s> {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState<'s>,

    // the font used for text rendering
    font: Font<vello::peniko::Font>,

    // the image used for image rendering
    image: Image<vello::peniko::Image>,
}

impl<'s> ApplicationHandler for SimpleVelloApp<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        // Get the winit window cached in a previous Suspended event or else create a new window
        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        // Create a vello Surface
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::Fifo,
        );

        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);

        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.state = RenderState::Active(ActiveRenderState {
            window,
            surface,
            i: 0,
        });

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        if let RenderState::Active(state) = &self.state {
            self.state = RenderState::Suspended(Some(state.window.clone()));
        }
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Ignore the event (return from the function) if
        //   - we have no render_state
        //   - OR the window id of the event doesn't match the window id of our render_state
        //
        // Else extract a mutable reference to the render state from its containing option for use below
        let render_state = match &mut self.state {
            RenderState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            // Exit the event loop when a close is requested (e.g. window's close button is pressed)
            WindowEvent::CloseRequested => event_loop.exit(),

            // Resize the surface when the window is resized
            WindowEvent::Resized(size) => {
                self.context
                    .resize_surface(&mut render_state.surface, size.width, size.height);
                render_state.window.request_redraw();
            }

            // This is where all the rendering happens
            WindowEvent::RedrawRequested => {
                // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
                // the same Scene is reused so that the underlying memory allocation can also be reused.
                //self.scene.reset();

                let (width, height) = render_state.window.inner_size().into();

                let mut scene = Scene::new(RGBA::BLUE, width, height);
                // increment the counter
                render_state.i += 1;

                let sine_grating_colors: Vec<RGBA> = (0..256)
                    .map(|i| {
                        let x = i as f32 / 256.0 * 1.0 * std::f32::consts::PI;
                        let t = x.sin();
                        RGBA {
                            r: t,
                            g: t,
                            b: t,
                            a: 1.0,
                        }
                    })
                    .collect();

                let gaussian_colors: Vec<RGBA> = (0..256)
                    .map(|i| {
                        let sigma: f32 = 0.3;
                        // we need a Gaussian function scaled to values between 0 and 1
                        // i.e., f(x) = exp(-x^2 / (2 * sigma^2))
                        let x = (i as f32 / 256.0);
                        let t = (-x.powi(2) / (2.0 * sigma.powi(2))).exp();
                        RGBA {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: t,
                        }
                    })
                    .collect();

                let sine_grating = Geom {
                    style: Style::Fill(FillStyle::NonZero),
                    shape: Circle {
                        center: Point { x: 0.0, y: 0.0 },
                        radius: 2000.0,
                    },
                    brush: Brush::Gradient(Gradient::new_equidistant(
                        Extend::Repeat,
                        GradientKind::Linear {
                            start: Point { x: 0.0, y: 0.0 },
                            end: Point { x: 100.0, y: 0.0 },
                        },
                        &sine_grating_colors,
                    )),
                    transform: Affine::identity(),
                    brush_transform: Some(Affine::translate(render_state.i as f64, 0.0)),
                };

                let gaussian = Geom {
                    style: Style::Fill(FillStyle::NonZero),
                    shape: Circle {
                        center: Point { x: 0.0, y: 0.0 },
                        radius: 2000.0,
                    },
                    brush: Brush::Gradient(Gradient::new_equidistant(
                        Extend::Pad,
                        GradientKind::Radial {
                            start_center: Point { x: 0.0, y: 0.0 },
                            start_radius: 0.0,
                            end_center: Point { x: 0.0, y: 0.0 },
                            end_radius: 2000.0,
                        },
                        &gaussian_colors,
                    )),
                    transform: Affine::identity(),
                    brush_transform: None,
                };

                scene.draw_alpha_mask(
                    |scene| {
                        scene.draw(sine_grating);
                    },
                    |scene| {
                        scene.draw(gaussian);
                    },
                    Circle {
                        center: Point { x: 0.0, y: 0.0 },
                        radius: 2000.0,
                    },
                    Affine::identity(),
                );

                let anchor = Geom {
                    style: Style::Fill(FillStyle::NonZero),
                    shape: Circle {
                        center: Point { x: 0.0, y: 0.0 },
                        radius: 10.0,
                    },
                    brush: Brush::Solid(RGBA::new(1.0, 0.0, 0.0, 1.0)),
                    transform: Affine::identity(),
                    brush_transform: None,
                };

                scene.draw(anchor);

                // Draw some text
                let text = FormatedText {
                    x: 0.0,
                    y: 0.0,
                    text: "click anywhere to start".to_string(),
                    size: 100.0,
                    color: RGBA::new(1.0, 1.0, 0.0, 1.0),
                    weight: 100.0,
                    font: self.font.clone(),
                    style: FontStyle::Normal,
                    alignment: Alignment::Center,
                    vertical_alignment: VerticalAlignment::Middle,
                    transform: Affine::identity(),
                    glyph_transform: None,
                };

                scene.draw(text);

                let image = Geom::new_image(
                    self.image.clone(),
                    200.0,
                    200.0,
                    500.0,
                    500.0,
                    Affine::translate(500.0, 20.0),
                    ImageFitMode::Fill,
                );

                //scene.draw(image);

                // Get the RenderSurface (surface + config)
                let surface = &render_state.surface;

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &self.context.devices[surface.dev_id];

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Render to the surface's texture
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &scene.backend.vello_scene,
                        &surface_texture,
                        &vello::RenderParams {
                            base_color: vello::peniko::Color {
                                r: 255,
                                g: 127,
                                b: 127,
                                a: 255,
                            }, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                            debug: DebugLayers::none(),
                        },
                    )
                    .expect("failed to render to surface");

                // Queue the texture to be presented on the surface
                surface_texture.present();

                // ask for a new frame
                render_state.window.request_redraw();

                device_handle.device.poll(wgpu::Maintain::Poll);
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    // load image using image crate
    let img = image::open("/Users/marc/renderer/examples/einstein.jpg")?;
    // Setup a bunch of state:
    let mut app = SimpleVelloApp {
        context: RenderContext::new(),
        renderers: vec![],
        state: RenderState::Suspended(None),
        font: Font::new(vello::peniko::Font::new(
            vello::peniko::Blob::new(Arc::new(ROBOTO_FONT)),
            0,
        )),
        image: Image::new(img),
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(1044, 800))
        .with_resizable(true)
        .with_title("Vello Shapes");
    Arc::new(event_loop.create_window(attr).unwrap())
}

/// Helper function that creates a vello `Renderer` for a given `RenderContext` and `RenderSurface`
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Couldn't create renderer")
}
