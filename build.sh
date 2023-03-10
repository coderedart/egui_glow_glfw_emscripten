#!/bin/sh
# this script is used to build and copy the files into a directory called dist.

echo "building for emscripten target"
cargo build --target=wasm32-unknown-emscripten --release

echo "copying files to dist directory"
mkdir -p dist
cp target/wasm32-unknown-emscripten/release/egui_glow_glfw_emscripten.wasm dist
cp target/wasm32-unknown-emscripten/release/egui_glow_glfw_emscripten.js dist
cp index.html dist