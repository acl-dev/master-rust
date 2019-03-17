#!/bin/sh
cargo build
./target/debug/echo -s "127.0.0.1:8822, 127.0.0.1:8823" -l log4rs-console.yaml
