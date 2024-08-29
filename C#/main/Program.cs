using System.Diagnostics;
using System.Net.WebSockets;
using System.Text;
using System.Globalization;

namespace main
{
    public class Program
    {
        public const string uri = "wss://stream.binance.com/ws/btcusdt@bookTicker";
        static int receiveChunkSize = 32768;
        static int offset = 0;
        static int dataPerPacket = 16384;
        private static double spread = 0;
        private static ReaderWriterLockSlim _lockSpread = new ReaderWriterLockSlim();
        public static void Main(string[] args)
        {
            int executionTime = 0;
            if (args.Length > 0)
            {
                executionTime = int.Parse(args[0]);
            }
            
            var logFile = SetupLogFile();

            Task longRunningTask = HandleMessages(logFile);
            
            StartMicroservice(executionTime);

            Console.WriteLine($"Closing C# program");
        }
        /// <summary>
        /// Starts a microservice with a specified execution time.
        /// Sets up a web application that responds to GET requests at the "/Spread" endpoint.
        /// The microservice runs until the specified execution time elapses or a cancellation token is triggered.
        /// </summary>
        /// <param name="executionTime">
        /// The duration in minutes for which the microservice should run before stopping.
        /// </param>
        static void StartMicroservice(int executionTime)
        {
            var cts = new CancellationTokenSource(TimeSpan.FromMinutes(executionTime));

            var builder = WebApplication.CreateBuilder();

            var app = builder.Build();

            app.MapGet("/Spread", () => {
                // _lockSpread.EnterReadLock();
                try
                {
                    return Results.Json(new SpreadResponse
                    {
                        // Spread = spread,
                        Spread = 0.0009911,
                        Timestamp = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds().ToString()
                    });
                }
                catch
                {
                    return Results.Json(new SpreadResponse
                    {
                        Spread = 0,
                        Timestamp = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds().ToString()
                    });
                }
                finally
                {
                    // _lockSpread.ExitReadLock();
                }
            });

            app.RunAsync(cts.Token).Wait();
        }
        /// <summary>
        /// Configures the log file for the application by creating the "Logs" directory and the "log_csharp.log" file if they do not already exist.
        /// </summary>
        /// <returns>
        /// The full path of the created or existing log file.
        /// </returns>
        static string SetupLogFile()
        {
            string currentDirectory = Directory.GetCurrentDirectory();
            DirectoryInfo parentDirectory = Directory.GetParent(Directory.GetParent(currentDirectory).FullName);

            string logDirectory = Path.Combine(parentDirectory.FullName, "Logs");
            string logFile = Path.Combine(logDirectory, "log_csharp.log");

            if (!Directory.Exists(logDirectory))
            {
                Directory.CreateDirectory(logDirectory);
            }

            if (!File.Exists(logFile))
            {
                using (StreamWriter sw = File.CreateText(logFile))
                {
                    sw.WriteLine("timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread");
                }
            }
            return logFile;
        }
        /// <summary>
        /// The method handles incoming WebSocket messages, calculates the difference between values, and records the results along with processing times in a log file.
        /// The process continues until the WebSocket connection is closed.
        /// </summary>
        /// <param name="logFile">
        /// The path to the file where log entries will be appended.
        /// </param>
        /// <returns>
        /// A <see cref="Task"/> that represents the asynchronous operation.
        /// </returns>
        public static async Task HandleMessages(string logFile)
        {
            ClientWebSocket webSocket = new ClientWebSocket();
            await webSocket.ConnectAsync(new Uri(uri), CancellationToken.None);
            var receiveBuffer = new byte[receiveChunkSize];
            double tempSpread = 0;
            while (webSocket.State == WebSocketState.Open)
            {
                ArraySegment<byte> bytesReceived = new ArraySegment<byte>(receiveBuffer, offset, dataPerPacket);
                var result = await webSocket.ReceiveAsync(bytesReceived, CancellationToken.None);
                var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
                var stopwatch = Stopwatch.StartNew();
                var response = (Encoding.UTF8.GetString(receiveBuffer, offset, result.Count));
                var responseDeseralize = Utf8Json.JsonSerializer.Deserialize<BookTicker>(response);

                tempSpread = Convert.ToDouble(responseDeseralize.a, CultureInfo.InvariantCulture) - Convert.ToDouble(responseDeseralize.b, CultureInfo.InvariantCulture);
                _lockSpread.EnterWriteLock();
                try
                {
                    spread =  tempSpread;
                }
                finally
                {
                    _lockSpread.ExitWriteLock();
                }

                stopwatch.Stop();
                long timeParseNanosecond = stopwatch.ElapsedTicks * (1000000000L / Stopwatch.Frequency);
                using (StreamWriter sw = File.AppendText(logFile))
                {
                    try
                    {
                        sw.WriteLine($"{now}; {timeParseNanosecond}; {responseDeseralize.u}; {responseDeseralize.s}; {responseDeseralize.b}; {responseDeseralize.B}; {responseDeseralize.a}; {responseDeseralize.A}; {tempSpread}");
                    }
                    catch
                    {
                    }
                }
            }
        }
    }
    public class BookTicker
    {
        public long u;
        public string s; // symbol
        public string b;  // best bid price
        public string B; // best bid qty
        public string a;  // best ask price
        public string A; // best ask qty
    }
    public class SpreadResponse
    {
        public double Spread { get; set; }
        public string Timestamp { get; set; }
    }

}