$ErrorActionPreference = "Stop"

Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
cargo build --target wasm32-unknown-unknown --release
Copy-Item "target\wasm32-unknown-unknown\release\star-crusher.wasm" "star-crusher.wasm" -Force
Write-Host "Built star-crusher.wasm"
