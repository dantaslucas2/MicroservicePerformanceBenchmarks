#include <websocketpp/config/asio_client.hpp>
#include <websocketpp/client.hpp>
#include <rapidjson/document.h>
#include <iostream>
#include "rapidjson/document.h"
#include "rapidjson/writer.h"
#include "rapidjson/stringbuffer.h"
#include <chrono>
#include <list>

typedef websocketpp::config::asio_client::message_type::ptr message_ptr;
typedef websocketpp::client<websocketpp::config::asio_tls_client> client;
typedef std::shared_ptr<boost::asio::ssl::context> context_ptr;
using websocketpp::lib::placeholders::_1;
using websocketpp::lib::placeholders::_2;
using websocketpp::lib::bind;
using namespace rapidjson;

std::list<int> minhaLista;

void Write_in_file(){

}
void on_message(client* c, websocketpp::connection_hdl hdl, message_ptr msg) {
    Document doc;
    if (doc.Parse(msg->get_payload().c_str()).HasParseError()) {
        std::cerr << "Erro ao fazer o parse da mensagem JSON" << std::endl;
    }
    uint64_t u = doc["u"].GetUint64();
    std::string s = doc["s"].GetString();
    std::string b = doc["b"].GetString();
    std::string B = doc["B"].GetString();
    std::string a = doc["a"].GetString();
    std::string A = doc["A"].GetString();

    std::cout << "u: " << u << std::endl;
    std::cout << "s: " << s << std::endl;
    std::cout << "b: " << b << std::endl;
    std::cout << "B: " << B << std::endl;
    std::cout << "a: " << a << std::endl;
    std::cout << "A: " << A << std::endl;

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
