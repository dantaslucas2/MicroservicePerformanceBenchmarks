package com.main;
import java.io.*;
import java.net.URI;
import java.net.http.*;
import java.net.http.WebSocket.Listener;
import java.nio.file.*;
import java.time.*;
import java.util.concurrent.*;
import java.util.concurrent.locks.*;
import java.net.InetSocketAddress;
import com.sun.net.httpserver.HttpServer;

import com.google.gson.*;

public class Main {
    private static final String URL = "wss://stream.binance.com/ws/btcusdt@bookTicker";
    private static double spread = 0;
    private static final ReentrantReadWriteLock lockSpread = new ReentrantReadWriteLock();
    private static final Gson gson = new Gson();

    public static void main(String[] args) {
        int executionTime = 0;
        if (args.length > 0) {
            executionTime = Integer.parseInt(args[0]);
        }

        Path logFile = setupLogFile();
        ExecutorService executor = Executors.newSingleThreadExecutor();

        CompletableFuture<Void> handleMessagesFuture = handleMessages(logFile, executor);

        startMicroservice(executionTime);

        System.out.println("Closing Java program");
    }
    private static void startMicroservice(int executionTime) {
        try {
            var server = HttpServer.create(new InetSocketAddress(8080), 0);
            server.setExecutor(Executors.newFixedThreadPool(40));
            server.createContext("/Spread", (exchange -> {
                String response;
                lockSpread.readLock().lock();
                var tempSpread = spread;
                lockSpread.readLock().unlock();
                System.out.println(server.getExecutor());

                try {
                    SpreadResponse spreadResponse = new SpreadResponse();
                    spreadResponse.Spread = tempSpread;
                    spreadResponse.Timestamp = String.valueOf(Instant.now().toEpochMilli());
                    response = gson.toJson(spreadResponse);
                } finally {
                    
                }
    
                exchange.sendResponseHeaders(200, response.getBytes().length);
                try (OutputStream os = exchange.getResponseBody()) {
                    os.write(response.getBytes());
                }
            }));
    
            server.start();
    
            try {
                Thread.sleep(TimeUnit.MINUTES.toMillis(executionTime));
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
    
            server.stop(0);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    private static Path setupLogFile() {
        String workingDir = System.getProperty("user.dir");
        Path pathCurrent = Paths.get(workingDir).getParent();
        Path logDirectory = pathCurrent.resolve("Logs");
        System.out.println( logDirectory);
        Path logFile = logDirectory.resolve("log_java.log");
        System.out.println("logFile: " + logFile);

        try {
            Files.createDirectories(logDirectory);
            if (!Files.exists(logFile)) {
                Files.createFile(logFile);
                Files.writeString(logFile, "timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread\n", StandardOpenOption.APPEND);
            }
        } catch (IOException e) {
            e.printStackTrace();
        }
        return logFile;
    }

    private static CompletableFuture<Void> handleMessages(Path logFile, ExecutorService executor) {
        HttpClient client = HttpClient.newHttpClient();
        CompletableFuture<WebSocket> wsFuture = client.newWebSocketBuilder()
            .buildAsync(URI.create(URL), new WebSocket.Listener() {
                public void onOpen(WebSocket webSocket) {
                    System.out.println("WebSocket opened");
                    webSocket.request(1);
                }

                public CompletionStage<?> onText(WebSocket webSocket, CharSequence data, boolean last) {
                    processMessage(data.toString(), logFile);
                    webSocket.request(1);
                    return null;
                }
            });

        return wsFuture.thenAccept(webSocket -> {});
    }

    private static void processMessage(String message, Path logFile) {
        BookTicker ticker = gson.fromJson(message, BookTicker.class);
        double tempSpread = Double.parseDouble(ticker.a) - Double.parseDouble(ticker.b);
        
        lockSpread.writeLock().lock();
        try {
            spread = tempSpread;
        } finally {
            lockSpread.writeLock().unlock();
        }

        long now = Instant.now().toEpochMilli();
        try {
            Files.writeString(logFile, String.format("%d; %s; %s; %s; %s; %s; %s; %.2f\n", now, ticker.u, ticker.s, ticker.b, ticker.B, ticker.a, ticker.A, tempSpread), StandardOpenOption.APPEND);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    static class BookTicker {
        String u;
        String s; // symbol
        String b;  // best bid price
        String B; // best bid qty
        String a;  // best ask price
        String A; // best ask qty
    }

    static class SpreadResponse {
        double Spread;
        String Timestamp;
    }
}