#!/bin/bash

echo "===== 清理旧构建 ====="
rm -rf pkg_webgpu pkg_webgl

echo "===== 构建 WebGPU 版本 ====="
RUSTFLAGS="--cfg=web_sys_unstable_apis" wasm-pack build --target web --out-dir pkg_webgpu -- --no-default-features --features webgpu

echo "===== 构建 WebGL 版本 ====="
wasm-pack build --target web --out-dir pkg_webgl -- --no-default-features --features webgl

echo "===== 构建完成 ====="
echo "WebGPU 版本: pkg_webgpu/"
echo "WebGL 版本: pkg_webgl/"

echo "===== 文件列表 ====="
echo "WebGPU 文件:"
ls -la pkg_webgpu/
echo
echo "WebGL 文件:"
ls -la pkg_webgl/
