#[cfg(not(target_arch = "wasm32"))]
use loft::lsp::run_server;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run_server().await;
}

#[cfg(target_arch = "wasm32")]
fn main() {}
