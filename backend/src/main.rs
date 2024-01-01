use serde::{Deserialize, Serialize};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use std::borrow::Cow;

const CANVAS_WIDTH: f64 = 8000.0;
const CANVAS_HEIGHT: f64 = 8000.0;
const COLOR_PALETTE: [Color; 6] = [
    Color::new(0, 0, 255),
    Color::new(32, 107, 203),
    Color::new(255, 100, 100),
    Color::new(255, 170, 100),
    Color::new(255, 200, 100),
    Color::new(0, 255, 0),
];

struct Offset {
    x: i32,
    y: i32,
}

struct Canvas {
    height: i32,
    width: i32,
}

#[derive(Debug, Deserialize)]
struct RequestParams {
    height: i32,
    width: i32,
    max_iter: i32,
    scale_factor: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
    color: Color,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd, Ord, Hash, Eq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Color { red, green, blue }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MandelbrotResponse {
    points: Vec<Point>,
}

fn main() {
    let params = RequestParams {
        height: CANVAS_HEIGHT as i32,
        width: CANVAS_WIDTH as i32,
        max_iter: 300,
        scale_factor: 1,
    };
    //let points = post_mandelbrot_request(params);

    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    pollster::block_on(run(event_loop, window));
}

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window).unwrap() };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, target, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        match event {
            Event::RedrawRequested(_window_id) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                match event {
                    WindowEvent::Resized(new_size) => {
                        // Reconfigure the surface with the new size
                        config.width = new_size.width.max(1);
                        config.height = new_size.height.max(1);
                        surface.configure(&device, &config);
                        // On macos the window needs to be redrawn manually after resizing
                        window.request_redraw();
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    });
}

fn post_mandelbrot_request(request: RequestParams) -> MandelbrotResponse {
    let start = std::time::SystemTime::now();

    let points = calculate_all_mandelbrot_points(
        request.width,
        request.height,
        request.max_iter,
        request.scale_factor,
    );
    let end = std::time::SystemTime::now();
    let dur = end.duration_since(start).unwrap();
    println!("time to calculate {} points was {:?}", points.len(), dur);

    MandelbrotResponse { points }
}

fn calculate_all_mandelbrot_points(
    width: i32,
    height: i32,
    max_iterations: i32,
    scale_factor: i32,
) -> Vec<Point> {
    let mut points = Vec::new();
    for i in 0..=width {
        for j in 0..=height {
            let point = calculate_mandelbrot_point_with_color(
                i as f64,
                j as f64,
                max_iterations,
                scale_factor,
            );
            points.push(point);
        }
    }
    return points;
}

fn calculate_mandelbrot_point_with_color(
    x_pos: f64,
    y_pos: f64,
    max_iterations: i32,
    scale_factor: i32,
) -> Point {
    let offset = Offset { x: 0, y: 0 };
    let (mut x, mut y) = pixels_to_cartesian_coords(x_pos, y_pos, offset, scale_factor);
    let c = (x, y);

    for i in 0..max_iterations {
        let (x_new, y_new) = single_mandelbrot_calc((x, y), c);
        let distance = ((x_new.powi(2) + y_new.powi(2)) as f64).sqrt();

        if distance > 2.0 {
            let color = COLOR_PALETTE[i as usize % COLOR_PALETTE.len()];
            return Point {
                x: x_pos,
                y: y_pos,
                color,
            };
        }
        x = x_new;
        y = y_new;
    }
    return Point {
        x,
        y: y_pos,
        color: Color::new(0, 0, 0),
    };
}

fn single_mandelbrot_calc(z_prev: (f64, f64), c: (f64, f64)) -> (f64, f64) {
    let a = z_prev.0;
    let b = z_prev.1;

    let new_z_tuple = (a.powi(2) - b.powi(2) + c.0, 2.0 * a * b + c.1);
    return new_z_tuple;
}

fn pixels_to_cartesian_coords(
    x_pos: f64,
    y_pos: f64,
    offset: Offset,
    scale_factor: i32,
) -> (f64, f64) {
    let ax_factor = 2.0 * scale_factor as f64;
    let x = x_pos / (CANVAS_WIDTH / (2.0 * ax_factor)) - ax_factor + offset.x as f64;
    let y = -y_pos / (CANVAS_HEIGHT / (2.0 * ax_factor)) + ax_factor + offset.y as f64;
    return (x, y);
}
