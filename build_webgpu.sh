#!/bin/bash
# 构建WASM并启动服务器

echo "===== 构建 WebGPU 版本 ====="
RUSTFLAGS="--cfg=web_sys_unstable_apis" wasm-pack build --target web --out-dir pkg_webgpu -- --no-default-features --features webgpu

simple-http-server -i --port 8080