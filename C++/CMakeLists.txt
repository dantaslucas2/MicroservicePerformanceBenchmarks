cmake_minimum_required(VERSION 3.10)
project(YourProject)

# Definir a versão do C++ (C++11, C++14, C++17, etc)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Encontrar e configurar o Boost
find_package(Boost REQUIRED system)
include_directories(${Boost_INCLUDE_DIRS})

# Encontrar e configurar o OpenSSL
find_package(OpenSSL REQUIRED)
include_directories(${OPENSSL_INCLUDE_DIR})

# Incluir diretórios de cabeçalhos da WebSocket++
include_directories(/usr/include/websocketpp)

# Adicionar o executável
add_executable(main main.cpp)

# Linkar bibliotecas necessárias
target_link_libraries(main ${Boost_LIBRARIES} ${OPENSSL_LIBRARIES} ssl crypto)
