@echo off
setlocal enabledelayedexpansion

echo Building the project...
cargo component build --target wasm32-wasip2 --release
@REM rustc ./src/main.rs --target wasm32-wasip2
if %errorlevel% neq 0 (
        echo Build failed.
        exit /b %errorlevel%
)

echo Running with Wasmtime...
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q1
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q1-opt

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q2
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q2-opt

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q3
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q3-opt

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q4
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q4-opt

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q5
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q5-opt

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid io
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson io
wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid io
@REM wasmtime -S inherit-network=y .\target\wasm32-wasip1\release\host.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
@REM wasmtime -S inherit-network=y ./main.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
