use serde::{Deserialize, Serialize};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
    run();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("plot")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
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
