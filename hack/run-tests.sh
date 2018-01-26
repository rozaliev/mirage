#! /bin/bash
trap 'kill -TERM $PID' TERM INT

cargo build --all & PID=$!
wait $PID

cargo test --all & PID=$!
wait $PID