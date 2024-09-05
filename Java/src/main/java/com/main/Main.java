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

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;
import org.springframework.web.socket.client.WebSocketClient;
import org.springframework.web.socket.client.standard.StandardWebSocketClient;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.TextWebSocketHandler;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

import com.google.gson.*;
@SpringBootApplication
public class Main {
    private static final String URL = "wss://stream.binance.com/ws/btcusdt@bookTicker";
    private static double spread = 0;
    private static final ReentrantReadWriteLock lockSpread = new ReentrantReadWriteLock();
    private static final Gson gson = new Gson();

    public static void main(String[] args) {
        SpringApplication.run(Main.class, args);
        int executionTime = 0;
        if (args.length > 0) {
            executionTime = Integer.parseInt(args[0]);
        }

        Path logFile = setupLogFile();
        ExecutorService executor = Executors.newSingleThreadExecutor();

        scheduleShutdown(executionTime);

        CompletableFuture<Void> handleMessagesFuture = handleMessages(logFile, executor);
    }
    @RestController
    public class SpreadController {

        @GetMapping("/Spread")
        public SpreadResponse getSpread() {
            SpreadResponse spreadResponse = new SpreadResponse();
            lockSpread.readLock().lock();
            try {
                spreadResponse.setSpread(spread);
                spreadResponse.setTimestamp(String.valueOf(Instant.now().toEpochMilli()));
            } finally {
                lockSpread.readLock().unlock();
            }
            return spreadResponse;
        }
    }
    private static void scheduleShutdown(int executionTime) {
        if (executionTime > 0) {
            ScheduledExecutorService scheduler = Executors.newSingleThreadScheduledExecutor();
            
            scheduler.schedule(() -> {
                System.out.println("Closing Java program");
                System.exit(0); 
            }, executionTime, TimeUnit.MINUTES);
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

    public static class SpreadResponse {
        private double spread;
        private String timestamp;

        public double getSpread() {
            return spread;
        }

        public void setSpread(double spread) {
            this.spread = spread;
        }

        public String getTimestamp() {
            return timestamp;
        }

        public void setTimestamp(String timestamp) {
            this.timestamp = timestamp;
        }
    }
}