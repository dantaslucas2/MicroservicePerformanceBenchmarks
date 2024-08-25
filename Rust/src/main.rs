use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::stream::StreamExt;
use serde::{Deserialize};
use std::fs::{self, OpenOptions, File};
use std::io::{Write, BufWriter};
use chrono::Utc;
use std::env;
use std::time::Instant;
use warp::Filter;
use tokio::sync::RwLock;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::net::SocketAddr;
use tokio::time::{self, Duration};

const URL: &str = "wss://stream.binance.com/ws/btcusdt@bookTicker";

lazy_static! {
    static ref SPREAD: RwLock<f64> = RwLock::new(0.0);
}

//http://127.0.0.1:3030/Spread
#[tokio::main]
async fn main() {
    let mut execution_time: u64 = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        execution_time = args[1].parse().expect("Not a valid number");
    }

    let file = setup_log_file().expect("Failed to set up logging");

    let server_handle = tokio::spawn(async move {
        start_spread_microservice().await;
    });

    let (ws_stream, _) = connect_async(URL).await.expect("Fail to connected WebSocket");
    let (_write, read) = ws_stream.split();

    let result = time::timeout(Duration::from_secs(execution_time * 60), async move {
        handle_messages(Box::pin(read), file).await;
    }).await;

    match result {
        Ok(_) => println!(" Closing Rust program "),
        Err(_) => println!(" Closing Rust program "),
    }

}
/// Initializes a log file within the "Logs" directory.
///
/// Ensures the "Logs" directory exists and appends to or creates "log_rust.log".
/// Initializes the file with headers if it's newly created.
///
/// # Errors
/// Returns an error if unable to determine the current directory, create the logs directory, 
/// or open the log file.
///
/// # Returns
/// A `BufWriter` wrapped around the log file on success.
fn setup_log_file() -> std::io::Result<BufWriter<std::fs::File>> {
    let base_dir = env::current_dir().expect("Failed to determine the current directory");
    let log_dir = base_dir.parent().unwrap_or(&base_dir).join("Logs");

    fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
    println!("Log directory has been ensured at: {}", log_dir.display());

    let log_path = log_dir.join("log_rust.log");

    let file = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?
    );

    let mut file = file;
    if file.get_ref().metadata()?.len() == 0 {
        writeln!(file, "timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread")?;
    }

    Ok(file)
}
/// Launches an asynchronous microservice for handling "Spread" requests.
///
/// This service sets up an HTTP server on localhost at port 3030, listening for requests to the "/Spread" path.
/// It asynchronously fetches and returns spread data as JSON.
///
/// # Example
/// ```
/// #[tokio::main]
/// async fn main() {
///     start_spread_microservice().await;
/// }
/// ```
async fn start_spread_microservice(){
    let get_route = warp::path!("Spread")
        .and_then(|| async {
            let data = get_spread().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&data))
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    
    let server = warp::serve(get_route).run(addr);

    server.await;
}
/// Asynchronously retrieves the current spread value from a shared state.
///
/// This function reads from a global `SPREAD` variable, assumed to be a concurrently accessible state, 
/// such as an `async_rwlock` or similar structure, and returns the value.
///
/// # Returns
/// Returns the current spread as a `f64`.
async fn get_spread() -> f64 {
    let spread = SPREAD.read().await;
    *spread
}
/// Asynchronously handles incoming WebSocket messages and logs spread data.
///
/// This function processes incoming messages as a stream, extracts spread data from text messages,
/// updates a shared `SPREAD` variable, and logs detailed timing and value data to a file.
/// It handles various message types, including Text, Binary, Ping, Pong, Close, and Frame.
///
/// # Parameters
/// - `read`: A mutable stream of WebSocket messages (`Message`), wrapped in a `Result` to handle potential errors.
/// - `file`: A `BufWriter<File>` to log the processed data.
///
/// # Behavior
/// The function breaks the loop if a message error occurs or on receiving a `Close` message.
/// It logs errors to stderr and skips erroneous deserializations without stopping the processing.

async fn handle_messages(mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + std::marker::Unpin, mut file: BufWriter<File>) {
    while let Some(message) = read.next().await {
        let now = Utc::now().timestamp_millis();
        let instant_before_parse = Instant::now(); 

        if let Err(e) = message {
            eprintln!("Error receiving message: {}", e);
            break;
        }

        let msg = match message {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        match msg {
            Message::Text(text) => {
                let parsed: Result<BookTicker, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ticker) => {
                        if let (Ok(bid), Ok(ask)) = (f64::from_str(&ticker.b), f64::from_str(&ticker.a)) {
                            let spread = ask - bid;
                            let mut spread_lock = SPREAD.write().await;
                            *spread_lock = spread;

                            let instant_after_parse = Instant::now();
                            let duration = instant_after_parse.duration_since(instant_before_parse);
                            writeln!(file, "{}; {}; {}; {}; {}; {}; {}; {}; {}", now, duration.as_nanos(), ticker.u, ticker.s, ticker.b, ticker.B, ticker.a, ticker.A, spread).unwrap();
                        }
                    }
                    Err(e) => eprintln!("Error deserializing JSON: {}", e),
                }
            }
            Message::Binary(bin) => println!("Received Binary: {:?}", bin),
            Message::Ping(ping) => println!("Received ping: {:?}", ping),
            Message::Pong(pong) => println!("Received pong: {:?}", pong),
            Message::Close(close) => {
                println!("Received close: {:?}", close);
                break;
            }
            Message::Frame(_) => println!("Received frame"),
        }
    }
}
#[derive(Deserialize, Debug)]
struct BookTicker {
    u: u64,    // order book updateId
    s: String, // symbol
    b: String, // best bid price
    B: String, // best bid qty
    a: String, // best ask price
    A: String, // best ask qty
}