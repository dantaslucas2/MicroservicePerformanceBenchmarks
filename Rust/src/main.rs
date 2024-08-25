use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
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
use tokio::time::{sleep, Duration};

const URL: &str = "wss://stream.binance.com/ws/btcusdt@bookTicker";
// let url = "wss://stream.binance.com/ws/btcusdt@bookTicker";

lazy_static! {
    static ref SPREAD: RwLock<f64> = RwLock::new(0.0);
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

//http://127.0.0.1:3030/Spread
#[tokio::main]
async fn main() {
    let mut execution_time: i32 = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        execution_time = args[1].parse().expect("Not a valid number");
    }

    let (ws_stream, _) = connect_async(URL).await.expect("Fail to connected WebSocket");

    println!("WebSocket Connected");

    let (_write, read) = ws_stream.split();

    let base_dir = env::current_dir().expect("Failed to determine the current directory");
    let log_dir = base_dir.parent().unwrap_or(&base_dir).join("Logs");

    fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
    println!("Log directory has been ensured at: {}", log_dir.display());

    let log_path = log_dir.join("log_rust.log");

    match env::current_dir(){
        Ok(dir) => println!("The current directory is: {}", dir.display()),
        Err(e) => println!("Erro ao obter o diretório atual: {}", e),
    }

    let mut file = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file")
    );
    if file.get_ref().metadata().unwrap().len() == 0 {
        writeln!(file, "timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread").unwrap();
    }

    let server_handle = tokio::spawn(async move {
        start_spread_microservice(execution_time).await;
    });

    handle_messages(Box::pin(read), file).await;
}

async fn start_spread_microservice(execution_time : i32){
    let get_route = warp::path!("Spread")
        .and_then(|| async {
            let data = get_spread().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&data))
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    
    let server = warp::serve(get_route).run(addr);

    tokio::select! {
        _ = server => {},
        _ = sleep(Duration::from_secs(execution_time as u64 * 60)) => {
        }
    }
}

async fn get_spread() -> f64 {
    let spread = SPREAD.read().await;
    *spread
}

async fn handle_messages(mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + std::marker::Unpin, mut file: BufWriter<File>) {
    while let Some(message) = read.next().await {
        let now = Utc::now().timestamp_millis();
        let instant_before_parse = Instant::now(); 

        // Tratar o erro de leitura do stream uma vez
        if let Err(e) = message {
            eprintln!("Error receiving message: {}", e);
            break;
        }

        // Tratar a mensagem recebida
        let msg = match message {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        match msg {
            Message::Text(text) => {
                // Processar a mensagem de texto
                let parsed: Result<BookTicker, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(ticker) => {
                        // Processar ticker
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