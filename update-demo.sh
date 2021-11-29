echo "Building demo ..."
cargo build --release --target wasm32-unknown-unknown
echo "Optimizing demo ..."
wasm-opt target/wasm32-unknown-unknown/release/life_web.wasm -Oz -o target/wasm32-unknown-unknown/release/life_web_opt.wasm
echo "Finalizing demo ..."
mv target/wasm32-unknown-unknown/release/life_web_opt.wasm demo/life_web.wasm
echo "done"
