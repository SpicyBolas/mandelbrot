use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use svg::Document;
use svg::{node::Attributes, Node};

use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use serde::{Deserialize, Serialize};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const CANVAS_WIDTH: f64 = 3600.0;
const CANVAS_HEIGHT: f64 = 3600.0;
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

#[tokio::main]
async fn main() {
    let params = RequestParams {
        height: CANVAS_HEIGHT as i32,
        width: CANVAS_WIDTH as i32,
        max_iter: 300,
        scale_factor: 1,
    };
    let points = post_mandelbrot_request(Json(params)).await.0;
    let save_path = PathBuf::from("./out.svg");
    render_svg(points, &save_path)
    //tracing_subscriber::registry()
    //    .with(
    //        tracing_subscriber::EnvFilter::try_from_default_env()
    //            .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
    //    )
    //    .with(tracing_subscriber::fmt::layer())
    //    .init();

    //let assets_dir = PathBuf::from("../");

    //// build our application with some routes
    //let app = Router::new()
    //    .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
    //    .route("/post-mandelbrot-request", post(post_mandelbrot_request))
    //    // logging so we can see whats going on
    //    .layer(
    //        TraceLayer::new_for_http()
    //            .make_span_with(DefaultMakeSpan::default().include_headers(true)),
    //    );

    //// run it with hyper
    //let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    //    .await
    //    .unwrap();
    //tracing::debug!("listening on {}", listener.local_addr().unwrap());
    //axum::serve(
    //    listener,
    //    app.into_make_service_with_connect_info::<SocketAddr>(),
    //)
    //.await
    //.unwrap();
}

fn render_svg(points: MandelbrotResponse, save_path: &PathBuf) {
    let mut document = Document::new();
    let colors = COLOR_PALETTE
        .iter()
        .map(|color| {
            (
                color.clone(),
                format!("rgb({},{},{})", color.red, color.green, color.blue),
            )
        })
        .collect::<HashMap<Color, String>>();
    let black = "rgb(0,0,0)".to_string();

    for point in points.points {
        let c = colors.get(&point.color).unwrap_or(&black);
        //arc(x, y, radius, startAngle, endAngle, counterclockwise)
        let arc = svg::node::element::Circle::new()
            .set("cx", point.x)
            .set("cy", point.y)
            .set("r", 0.1)
            .set("fill", c.clone());
        document.append(arc);
    }

    svg::save(save_path, &document).unwrap();
}

async fn post_mandelbrot_request(Json(request): Json<RequestParams>) -> Json<MandelbrotResponse> {
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

    Json(MandelbrotResponse { points })
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
