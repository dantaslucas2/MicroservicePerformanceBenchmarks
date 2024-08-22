use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::fs::{self, OpenOptions, File};
use std::io::{Write, BufWriter};
use std::path::Path;
use chrono::Utc;
use std::env;

#[derive(Deserialize, Debug)]
struct BookTicker {
    u: u64,    // order book updateId
    s: String, // symbol
    b: String, // best bid price
    B: String, // best bid qty
    a: String, // best ask price
    A: String, // best ask qty
}

#[tokio::main]
async fn main() {
    let url = "wss://stream.binance.com/ws/btcusdt@bookTicker";

    let (ws_stream, _) = connect_async(url).await.expect("Fail to connected WebSocket");

    println!("WebSocket Connected");

    let (_write, read) = ws_stream.split();

    let base_dir = env::current_dir().expect("Failed to determine the current directory");
    let log_dir = base_dir.parent().unwrap_or(&base_dir).join("Logs");

    fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
    println!("Log directory has been ensured at: {}", log_dir.display());

    let log_path = log_dir.join("log_rust.log");

    match env::current_dir(){
        Ok(dir) => println!("O diretório atual é: {}", dir.display()),
        Err(e) => println!("Erro ao obter o diretório atual: {}", e),
    }
    // println!("{}", &log_path);

    let mut file = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file")
    );
    if file.get_ref().metadata().unwrap().len() == 0 {
        writeln!(file, "timestampReceive; timeParseNanosecond; u; s; b; B; a; A:").unwrap();
    }

    handle_messages(Box::pin(read), file).await;
}

async fn handle_messages(mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + std::marker::Unpin, mut file: BufWriter<File>) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    let now_before_parse = Utc::now().timestamp_millis();
                    let parsed: Result<BookTicker, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(ticker) => {
                            println!("Received: {:?}", ticker);
                            let now_after_parse = Utc::now().timestamp_millis();
                            writeln!(file, "{}; {}; {}; {}; {}; {}; {}; {}", now_before_parse, now_after_parse, ticker.u, ticker.s, ticker.b, ticker.B, ticker.a, ticker.A).unwrap();
                        }
                        Err(e) => {
                            eprintln!("Error deserialize json: {}", e);
                        }
                    }
                }
                Message::Binary(bin) => {
                    println!("Received Binary: {:?}", bin);
                }
                Message::Ping(ping) => {
                    println!("Received ping: {:?}", ping);
                }
                Message::Pong(pong) => {
                    println!("Received pong: {:?}", pong);
                }
                Message::Close(close) => {
                    println!("Received close: {:?}", close);
                    break;
                }
                Message::Frame(_) => {
                    println!("Received frame");
                }
            },
            Err(e) => {
                eprintln!("Error to recive menssage: {}", e);
                break;
            }
        }
    }
}
