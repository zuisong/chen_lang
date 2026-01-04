use anyhow::Result;
use chen_lang_lsp::ChenLangLsp;
use tower_lsp_server::{LspService, Server};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Chen Lang LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(ChenLangLsp::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
