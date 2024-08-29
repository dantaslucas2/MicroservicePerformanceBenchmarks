from flask import Flask, jsonify
from multiprocessing import Process, Lock, Manager
import time
import asyncio
import websockets
import json
import os
import sys
import threading
import time
import signal

app = Flask(__name__)
URI = "wss://stream.binance.com/ws/btcusdt@bookTicker"

spread = 0
def setup_log_file():
    """
    Sets up the log file for storing received messages and spread calculations.

    - Creates a "Logs" directory in the parent directory of the current working directory if it does not exist.
    - Creates a log file named "log_python.log" within this directory.
    - If the log file does not exist, creates it and writes a header with column names.

    Returns:
        str: Full path to the log file.
    """
    current_directory = os.getcwd()

    parent_directory = os.path.dirname(current_directory)

    log_directory = os.path.join(parent_directory, "Logs")
    log_file = os.path.join(log_directory, "log_python.log")

    if not os.path.exists(log_directory):
        os.makedirs(log_directory)

    if not os.path.exists(log_file):
        with open(log_file, 'w') as file:
            file.write("timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread\n")
    print(log_file)
    return log_file

async def handle_messages(log_file):
    """
    Asynchronous function that connects to a WebSocket and processes received messages.

    - Connects to the WebSocket provided by the global URI.
    - Receives and processes messages in an infinite loop.
    - Computes the spread between two values and logs the message and calculation to the log file.

    Args:
        log_file (str): Path to the log file where messages will be recorded.
    """
    async with websockets.connect(URI) as websocket:
        while True:
            now = int(time.time() * 1000) 
            start_time = time.perf_counter_ns()

            response = await websocket.recv()
            data = json.loads(response)

            temp_spread = float(data['a']) - float(data['b'])

            time_parse_nanosecond = time.perf_counter_ns() - start_time

            print(response)
            print(data)

            with open(log_file, 'a') as file:
                file.write(f"{now}; {time_parse_nanosecond}; {data['u']}; {data['s']}; {data['b']}; {data['B']}; {data['a']}; {data['A']}; {temp_spread}\n")

@app.route('/Spread', methods=['GET'])
def get_spread():
    """
    Flask API endpoint that provides information about the spread and the current timestamp.

    - Returns a JSON response with spread and the current timestamp in milliseconds.

    Returns:
        Response: JSON object containing the double_spread and the current timestamp.
    """
    double_spread = 2.0
    timestamp = int(time.time() * 1000)

    response = {
        'double_spread': double_spread,
        'timestamp': timestamp
    }
    return jsonify(response)

def run_flask_app():
    """
    Starts the Flask server.

    - Runs the Flask server with debugging enabled.
    """
    app.run(debug=True)

def start_async_loop(loop):
    """
    Sets up and starts an asynchronous event loop.

    - Sets the provided event loop as the current event loop.
    - Starts the event loop for continuous execution.

    Args:
        loop (asyncio.AbstractEventLoop): The asynchronous event loop to be started.
    """
    asyncio.set_event_loop(loop)
    loop.run_forever()

def run_for_duration_and_terminate(duration):
    print(f"Thread started, will terminate the program in {duration} seconds...")
    time.sleep(duration)
    print("Time's up! Terminating the program.")
    sys.exit(0)  # Encerra o programa

def main():
    params = 0
    if len(sys.argv) > 1:
        params = int(sys.argv[1])  # Tempo em segundos
    print("****")
    termination_thread = threading.Thread(target=run_for_duration_and_terminate, args=(params,))
    termination_thread.start()

    log_file = setup_log_file()
    loop = asyncio.new_event_loop()

    asyncio_thread = threading.Thread(target=start_async_loop, args=(loop,))
    asyncio_thread.start()
    asyncio.run_coroutine_threadsafe(handle_messages(log_file), loop)

    time.sleep(params)

    loop.call_soon_threadsafe(loop.stop)
    asyncio_thread.join()

    app.run(debug=False, use_reloader=False)
    print()

if __name__ == '__main__':
    sys.stdout = sys.stderr
    
    logic_thread = threading.Thread(target=main)
    logic_thread.start()


    # Configurar um sinal para parar o programa após o tempo especificado
    signal.signal(signal.SIGALRM, sys.exit(0))
    signal.alarm(params)

    run_flask_app()

    logic_thread.join()