#!/bin/bash

parent_dir=".."
cpp_dir="$parent_dir/C++/build"
rust_dir="$parent_dir/Rust"
csharp_dir="$parent_dir/C#/main"

execution_time=1

(
    cd "$cpp_dir" && ./main "$execution_time"
) & pid_cpp=$!

(
    cd "$rust_dir" && cargo run --release -- "$execution_time"
) & pid_rust=$!

(
    cd "$csharp_dir" && dotnet run --configuration Release -- "$execution_time"
) & pid_csharp=$!

#sleep 600
sleep 30

kill $pid_cpp
kill $pid_rust
kill $pid_csharp

sleep 2

if ps -p $pid_cpp > /dev/null; then
    kill -9 $pid_cpp
fi

if ps -p $pid_rust > /dev/null; then
    kill -9 $pid_rust
fi

if ps -p $pid_csharp > /dev/null; then
    kill -9 $pid_csharp
fi

echo "The programs were closed."
