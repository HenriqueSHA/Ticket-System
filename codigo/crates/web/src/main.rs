//! Ponto de entrada do **front**: serve as páginas e conversa com o back pela API.
//! Não tem acesso ao banco de dados.

use web::{Estado, rotas};

const BACK_PADRAO: &str = "http://127.0.0.1:8080";
const ENDERECO_PADRAO: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv(); // lê o .env da pasta atual ou de uma pasta acima
    let url_back = std::env::var("URL_BACK").unwrap_or_else(|_| BACK_PADRAO.to_string());
    let endereco = std::env::var("ENDERECO").unwrap_or_else(|_| ENDERECO_PADRAO.to_string());
    let cookies_seguros = std::env::var("COOKIE_SEGURO").is_ok_and(|v| v.eq_ignore_ascii_case("true"));

    let listener = match tokio::net::TcpListener::bind(&endereco).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("✖ Não foi possível usar o endereço {endereco}: {e}");
            std::process::exit(1);
        }
    };
    println!("\n  Sistema de Ingressos (front) no ar:  http://{endereco}\n  Back-end: {url_back}\n  (Ctrl+C para encerrar)");
    let servidor = axum::serve(listener, rotas::criar(Estado::novo(&url_back, cookies_seguros)))
        .with_graceful_shutdown(encerramento());
    if let Err(e) = servidor.await {
        eprintln!("✖ Servidor encerrado com erro: {e}");
    }
    println!("Front encerrado.");
}

/// Ctrl+C ou SIGTERM (enviado pelo Docker ao parar o contêiner).
async fn encerramento() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let termino = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let termino = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = termino => {},
    }
}
