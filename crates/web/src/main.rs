//! Ponto de entrada do sistema (*composition root*): é o único lugar que conhece as
//! implementações concretas. Ele cria o PostgreSQL, o Argon2 e o relógio, injeta tudo na
//! `Aplicacao` e entrega a aplicação para a interface web.

use std::sync::Arc;

use aplicacao::{Aplicacao, Dependencias, dados_exemplo};
use infraestrutura::postgres::BancoPostgres;
use infraestrutura::relogio::RelogioSistema;
use infraestrutura::seguranca::HashArgon2;

use web::{Estado, rotas};

const BANCO_PADRAO: &str = "postgres://ingressos:ingressos@localhost:5433/ingressos";
const ENDERECO_PADRAO: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv(); // lê o arquivo .env, se existir
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| BANCO_PADRAO.to_string());
    let endereco = std::env::var("ENDERECO").unwrap_or_else(|_| ENDERECO_PADRAO.to_string());

    let banco = match BancoPostgres::conectar(&url).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✖ {e}\n  Suba o banco com:  docker compose up -d");
            std::process::exit(1);
        }
    };
    if let Err(e) = banco.migrar().await {
        eprintln!("✖ {e}");
        std::process::exit(1);
    }

    // Injeção de dependências: a aplicação só recebe interfaces (traits).
    let app = Aplicacao::nova(Dependencias {
        usuarios: Arc::new(banco.usuarios()),
        organizadores: Arc::new(banco.organizadores()),
        eventos: Arc::new(banco.eventos()),
        vendas: Arc::new(banco.vendas()),
        hash: Arc::new(HashArgon2),
        relogio: Arc::new(RelogioSistema),
    });

    match dados_exemplo::popular_se_vazio(&app).await {
        Ok(true) => {
            println!("Banco vazio: dados de demonstração criados. Logins:");
            for (perfil, email, senha) in dados_exemplo::LOGINS {
                println!("  {perfil:<12} {email:<26} senha: {senha}");
            }
        }
        Ok(false) => {}
        Err(e) => eprintln!("Aviso: não foi possível criar os dados de demonstração: {e}"),
    }

    let listener = match tokio::net::TcpListener::bind(&endereco).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("✖ Não foi possível usar o endereço {endereco}: {e}");
            std::process::exit(1);
        }
    };
    println!("\n  Sistema de Ingressos no ar:  http://{endereco}\n  (Ctrl+C para encerrar)");
    if let Err(e) = axum::serve(listener, rotas::criar(Estado::novo(app))).await {
        eprintln!("✖ Servidor encerrado com erro: {e}");
    }
}
