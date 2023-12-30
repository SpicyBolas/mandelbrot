use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use serde_json::de;

use std::borrow::Cow;
use std::ops::ControlFlow;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use serde::Deserialize;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures::{sink::SinkExt, stream::StreamExt};

const SCALE_FACTOR: i32 = 1;
const AX_FACTOR: i32 = 2; // TODO - lazy eval, should be 2*SCALE_FACTOR
const CANVAS_WIDTH: i32 = 800;
const CANVAS_HEIGHT: i32 = 800;
const MAX_ITER: i32 = 1000;
const COLOR_PALETTE: [&'static str; 6] = [
    "rgb(0,0,255)",
    "rgb(32,107,203)",
    "rgb(255,100,100)",
    "rgb(255,170,100)",
    "rgb(255,200,100)",
    "rgb(0,255,0)",
];

struct Offset {
    x: i32,
    y: i32,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from("../");

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // receive single message from a client (we can either receive or send with socket).
    // this will likely be the Pong for our Ping or a hello message from client.
    // waiting for message from a client will block this task, but will not block other client's
    // connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return;
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }

    // Since each client gets individual statemachine, we can pause handling
    // when necessary to wait for some external event (in this case illustrated by sleeping).
    // Waiting for this client to finish getting its greetings does not prevent other clients from
    // connecting to server and receiving their greetings.
    for i in 1..5 {
        if socket
            .send(Message::Text(format!("Hi {i} times!")))
            .await
            .is_err()
        {
            println!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            if sender
                .send(Message::Text(format!("Server message {i} ...")))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        println!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Cow::from("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e}, probably it is ok?");
        }
        n_msg
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            let message = process_message(msg, who);
            if process_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a} messages sent to {who}"),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {b} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

#[derive(Debug, Deserialize)]
struct RequestParams {
    height: i32,
    width: i32,
    max_iter: i32,
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> Option<RequestParams> {
    match msg {
        Message::Text(t) => {
            let params = serde_json::from_str::<RequestParams>(&*t).unwrap();
            println!(">>> {who} sent str: {t:?}");
            return Some(params);
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return None;
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    None
}

async fn root() -> Html<String> {
    let file = std::fs::read_to_string("../index.html").unwrap();
    Html(file)
}

fn calculate_mandelbrot_points_with_color(x_pos: i32, y_pos: i32) -> (i32, i32, &'static str) {
    let offset = Offset { x: 0, y: 0 };
    let (x_pos, y_pos) = pixels_to_cartesian_coords(x_pos, y_pos, offset);
    let c = (x_pos, y_pos);

    for i in 0..MAX_ITER {
        let (x_pos, y_pos) = single_mandelbrot_calc((x_pos, y_pos), c);
        let distance = ((x_pos.pow(2) + y_pos.pow(2)) as f64).sqrt();

        if distance > 2.0 {
            let hue = 100 * i / MAX_ITER;
            let color = COLOR_PALETTE[i as usize % COLOR_PALETTE.len()];
            return (x_pos, y_pos, color);
        }
    }
    return (x_pos, y_pos, "black");
}

fn single_mandelbrot_calc(z_prev: (i32, i32), c: (i32, i32)) -> (i32, i32) {
    let a = z_prev.0;
    let b = z_prev.1;

    let new_z_tuple = (a.pow(2) - b.pow(2) + c.0, 2 * a * b + c.1);
    return new_z_tuple;
}

fn pixels_to_cartesian_coords(x_pos: i32, y_pos: i32, offset: Offset) -> (i32, i32) {
    let x = x_pos / (CANVAS_WIDTH / (2 * AX_FACTOR)) - AX_FACTOR + offset.x;
    let y = -y_pos / (CANVAS_HEIGHT / (2 * AX_FACTOR)) + AX_FACTOR + offset.y;
    return (x, y);
}

fn cartesian_coords_to_pixels(x_pos: i32, y_pos: i32, offset: Offset) -> (i32, i32) {
    let x = CANVAS_WIDTH / (2 * AX_FACTOR) * ((x_pos - offset.x) + AX_FACTOR);
    let y = -CANVAS_HEIGHT / (2 * AX_FACTOR) * ((y_pos - offset.y) - AX_FACTOR);
    return (x, y);
}
