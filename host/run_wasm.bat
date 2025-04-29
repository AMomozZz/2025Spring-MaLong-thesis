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
@REM wasmtime -S inherit-network=y .\target\wasm32-wasip1\release\host.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
@REM wasmtime -S inherit-network=y ./main.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100



@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q1
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q1-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q2
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q2-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q3
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q3-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q4
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q4-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q5
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q5-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q6
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid q6-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q7
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid q7-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q8
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson q8-opt

@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid qw 10000 100
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid qw-opt-pruned 10000 100
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid qw-opt-operator 10000 100

wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/bid io
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionPerson io
@REM wasmtime -O opt-level=0 -S cli=y -S inherit-network=y --dir ../nexmark-data ./target/wasm32-wasip2/release/host.wasm ../nexmark-data/auctionBid io