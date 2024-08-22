#include <websocketpp/config/asio_client.hpp>
#include <websocketpp/client.hpp>
#include <rapidjson/document.h>
#include <iostream>
#include <filesystem>
#include <chrono>
#include <list>
#include <fstream>
#include "rapidjson/document.h"
#include "rapidjson/writer.h"
#include "rapidjson/stringbuffer.h"

typedef websocketpp::config::asio_client::message_type::ptr message_ptr;
typedef websocketpp::client<websocketpp::config::asio_tls_client> client;
typedef std::shared_ptr<boost::asio::ssl::context> context_ptr;
using websocketpp::lib::placeholders::_1;
using websocketpp::lib::placeholders::_2;
using websocketpp::lib::bind;
using namespace rapidjson;

std::string get_current_timestamp() {
    using namespace std::chrono;
    auto now = system_clock::now();
    auto now_ms = time_point_cast<milliseconds>(now).time_since_epoch().count();
    return std::to_string(now_ms);
}

void on_message(client* c, websocketpp::connection_hdl hdl, message_ptr msg) {
    std::string timestampReceive = get_current_timestamp();
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

    std::string timestampAfterParse = get_current_timestamp();

    namespace fs = std::filesystem;
    fs::path log_dir = fs::current_path().parent_path().parent_path() / "Logs";
    fs::create_directories(log_dir);
    fs::path log_file = log_dir / "log_GCC.log";

    bool file_exists = fs::exists(log_file);
    std::ofstream logfile(log_file, std::ios_base::app);

    if (logfile.is_open()) {
        if (!file_exists) {
            logfile << "timestampReceive; timeParseNanosecond; u; s; b; B; a; A:" << std::endl;
        }
        logfile << timestampReceive << "; " << timestampAfterParse << "; " << u << "; " << s << "; " << b << "; " << B << "; " << a << "; " << A << std::endl;
        logfile.close();
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

int main(int argc, char* argv[]) {
    Get_Binance_BTC_BBO_Book();
}
