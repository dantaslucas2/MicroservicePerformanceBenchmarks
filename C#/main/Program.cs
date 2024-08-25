using System.Diagnostics;
using System.Net.WebSockets;
using System.Text;
using System.Globalization;

namespace main
{
    public class Program
    {
        public static string uri = "wss://stream.binance.com/ws/btcusdt@bookTicker";
        static int receiveChunkSize = 32768;
        static int offset = 0;
        static int dataPerPacket = 16384;
        private static double spread = 0;
        private static ReaderWriterLockSlim _lockSpread = new ReaderWriterLockSlim();
        public async static Task Main(string[] args)
        {
            int executionTime = 0;
            if (args.Length > 0)
            {
                executionTime = int.Parse(args[0]);
            }
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

            Task longRunningTask = HandleMessages(logFile);
            
            var cts = new CancellationTokenSource(TimeSpan.FromMinutes(executionTime));

            var builder = WebApplication.CreateBuilder(args);

            var app = builder.Build();

            app.MapGet("/Spread", () => {
            try{
                _lockSpread.EnterReadLock();
                return spread;
            }catch{
                return 0;
            }finally{
                _lockSpread.ExitReadLock();
            }

            });
            app.RunAsync(cts.Token).Wait();
        }
        
        public static async Task HandleMessages(string logFile)
        {
            ClientWebSocket webSocket = new ClientWebSocket();
            await webSocket.ConnectAsync(new Uri(uri), CancellationToken.None);
            var receiveBuffer = new byte[receiveChunkSize];

            while (webSocket.State == WebSocketState.Open)
            {
                ArraySegment<byte> bytesReceived = new ArraySegment<byte>(receiveBuffer, offset, dataPerPacket);
                var result = await webSocket.ReceiveAsync(bytesReceived, CancellationToken.None);
                var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
                var stopwatch = Stopwatch.StartNew();
                var response = (Encoding.UTF8.GetString(receiveBuffer, offset, result.Count));
                var responseDeseralize = Utf8Json.JsonSerializer.Deserialize<BookTicker>(response);


                // Console.WriteLine($"{response}");
                _lockSpread.EnterWriteLock();
                try
                {
                    spread = Convert.ToDouble(responseDeseralize.a, CultureInfo.InvariantCulture) - Convert.ToDouble(responseDeseralize.b, CultureInfo.InvariantCulture);
                    // Console.WriteLine(spread);
                }
                finally
                {
                    _lockSpread.ExitWriteLock();
                }

                stopwatch.Stop();
                long timeParseNanosecond = stopwatch.ElapsedTicks * (1000000000L / Stopwatch.Frequency);
                using (StreamWriter sw = File.AppendText(logFile))
                {
                    _lockSpread.EnterReadLock();
                    try
                    {
                        sw.WriteLine($"{now}; {timeParseNanosecond}; {responseDeseralize.u}; {responseDeseralize.s}; {responseDeseralize.b}; {responseDeseralize.B}; {responseDeseralize.a}; {responseDeseralize.A}; {spread}");
                    }
                    finally
                    {
                        _lockSpread.ExitReadLock();
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
}