#include <websocketpp/config/asio_client.hpp>
#include <websocketpp/client.hpp>
#include <iostream>
#include <filesystem>
#include <chrono>
#include <list>
#include <fstream>
#include <chrono>
#include "rapidjson/document.h"
#include "rapidjson/writer.h"
#include "rapidjson/stringbuffer.h"
#include <thread>
#include <boost/asio.hpp>
#include <boost/beast.hpp>
#include <shared_mutex>

typedef websocketpp::config::asio_client::message_type::ptr message_ptr;
typedef websocketpp::client<websocketpp::config::asio_tls_client> client;
typedef std::shared_ptr<boost::asio::ssl::context> context_ptr;
using websocketpp::lib::placeholders::_1;
using websocketpp::lib::placeholders::_2;
using websocketpp::lib::bind;
using namespace rapidjson;
using namespace std::chrono;

namespace beast = boost::beast;
namespace http = beast::http;
namespace net = boost::asio;
using tcp = net::ip::tcp;

std::shared_mutex spreadMutex;
double spread = 0.0;
std::ofstream* logfiles = nullptr; 
int requestCount = 0;


void handle_session_sync(std::shared_ptr<tcp::socket> socket) {
    try {
        beast::flat_buffer buffer;
        http::request<http::string_body> request;

        http::read(*socket, buffer, request);

        std::cout << "Request received " << requestCount++ << std::endl;

        http::response<http::string_body> response{http::status::ok, request.version()};
        response.set(http::field::server, "Boost.Beast");
        response.set(http::field::content_type, "text/plain");
        response.keep_alive(request.keep_alive());
        std::shared_lock<std::shared_mutex> lock(spreadMutex);
        response.body() = std::to_string(spread);
        response.prepare_payload();

        http::write(*socket, response);

        socket->shutdown(tcp::socket::shutdown_send);
    } catch (const std::exception& e) {
        std::cerr << "Exception in handle_session_sync: " << e.what() << std::endl;
    }
}

void http_server_sync(net::io_context& ioc, unsigned short port) {
    tcp::acceptor acceptor(ioc, tcp::endpoint(tcp::v4(), port));

    while (true) {
        tcp::socket socket(ioc);
        acceptor.accept(socket);

        // std::cout << "Accepted connection" << std::endl;

        handle_session_sync(std::make_shared<tcp::socket>(std::move(socket)));
    }
}

void run_server_sync() {
    net::io_context ioc;
    unsigned short port = 8080;
    http_server_sync(ioc, port);
    ioc.run();
}


std::string get_current_timestamp() {
    using namespace std::chrono;
    auto now = system_clock::now();
    auto now_ms = time_point_cast<milliseconds>(now).time_since_epoch().count();
    return std::to_string(now_ms);
}

void on_message(client* c, websocketpp::connection_hdl hdl, message_ptr msg) {
    std::string now = get_current_timestamp();
    auto timestampReceive = high_resolution_clock::now();

    Document doc;
    if (doc.Parse(msg->get_payload().c_str()).HasParseError()) {
        std::cerr << "Error parse JSON" << std::endl;
    }
    uint64_t u = doc["u"].GetUint64();
    std::string s = doc["s"].GetString();
    std::string b = doc["b"].GetString();
    std::string B = doc["B"].GetString();
    std::string a = doc["a"].GetString();
    std::string A = doc["A"].GetString();

    auto timestampAfterParse = high_resolution_clock::now(); 
    auto duration = duration_cast<nanoseconds>(timestampAfterParse - timestampReceive).count();

    if (logfiles->is_open()) {
        std::unique_lock<std::shared_mutex> lock(spreadMutex);
        spread = (std::stod(a) - std::stod(b));
        *logfiles << now << "; " << duration << "; " << u << "; " << s << "; " << b << "; " << B << "; " << a << "; " << A << "; " << spread <<std::endl;
        // logfiles->close();
    } else {
        std::cerr << "Unable to open log file" << std::endl;
    }

    websocketpp::lib::error_code ec;
    if (ec) {
        std::cout << "Echo failed because: " << ec.message() << std::endl;
    }
}

static context_ptr on_tls_init() {
    context_ptr ctx = std::make_shared<boost::asio::ssl::context>(boost::asio::ssl::context::sslv23);

    try {
        ctx->set_options(boost::asio::ssl::context::default_workarounds |
                         boost::asio::ssl::context::no_sslv2 |
                         boost::asio::ssl::context::no_sslv3 |
                         boost::asio::ssl::context::single_dh_use);
    } catch (std::exception &e) {
        std::cout << "Error in context pointer: " << e.what() << std::endl;
    }
    return ctx;
}

void Get_Binance_BTC_BBO_Book(){
    client c;

    std::string uri = "wss://stream.binance.com/ws/btcusdt@bookTicker";

    namespace fs = std::filesystem;
    fs::path log_dir = fs::current_path().parent_path().parent_path() / "Logs";
    fs::create_directories(log_dir);
    fs::path log_file = log_dir / "log_gcc.log";

    bool file_exists = fs::exists(log_file);
    std::ofstream logfile(log_file, std::ios_base::app);
    logfiles = &logfile;

    if (logfile.is_open()) {
        if (!file_exists) {
            logfile << "timestampReceive; timeParseNanosecond; u; s; b; B; a; A; Spread" << std::endl;
        }
    }

    try {

        c.init_asio();
        c.set_tls_init_handler(bind(&on_tls_init));

        c.set_message_handler(bind(&on_message,&c,::_1,::_2));

        websocketpp::lib::error_code ec;
        client::connection_ptr con = c.get_connection(uri, ec);
        if (ec) {
            std::cout << "could not create connection because: " << ec.message() << std::endl;
            return ;
        }

        c.connect(con);

        c.run();
    } catch (websocketpp::exception const & e) {
        std::cout << e.what() << std::endl;
    }
}
//http://127.0.0.1:8080/Spread
int main(int argc, char* argv[]) {
    int execution_time = 0;
    if (argc > 1) {
        execution_time = std::atoi(argv[1]);
    }
    try {
        std::thread server_thread(run_server_sync);
        server_thread.detach();

        std::thread websocket_thread(Get_Binance_BTC_BBO_Book);
        websocket_thread.detach();

        std::this_thread::sleep_for(std::chrono::minutes(execution_time));
        std::cout << " closing c++ program " << std::endl;
        // Get_Binance_BTC_BBO_Book();
    } catch (const std::exception& e) {
        std::cerr << "Exception in main: " << e.what() << std::endl;
    }
}
