//! Ponto de entrada do **back** (*composition root*): o único lugar que conhece as
//! implementações concretas. Cria o PostgreSQL, o Argon2 e o relógio, injeta tudo na
//! `Aplicacao` e publica a API.

use std::sync::Arc;

use api::{EstadoApi, rotas};
use aplicacao::{Aplicacao, Dependencias, dados_exemplo};
use infraestrutura::postgres::BancoPostgres;
use infraestrutura::relogio::RelogioSistema;
use infraestrutura::seguranca::HashArgon2;

const BANCO_PADRAO: &str = "postgres://ingressos:ingressos@localhost:5433/ingressos";
const ENDERECO_PADRAO: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv(); // lê o .env da pasta atual ou de uma pasta acima
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| BANCO_PADRAO.to_string());
    let endereco = std::env::var("ENDERECO_API").unwrap_or_else(|_| ENDERECO_PADRAO.to_string());

    let banco = match BancoPostgres::conectar(&url).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("✖ {e}\n  Suba o banco com:  docker compose up -d db");
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

    preparar_dados_iniciais(&app).await;

    let listener = match tokio::net::TcpListener::bind(&endereco).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("✖ Não foi possível usar o endereço {endereco}: {e}");
            std::process::exit(1);
        }
    };
    println!("API (back) no ar: http://{endereco}/api");
    let servidor = axum::serve(listener, rotas::criar(EstadoApi::novo(app))).with_graceful_shutdown(encerramento());
    if let Err(e) = servidor.await {
        eprintln!("✖ Servidor encerrado com erro: {e}");
    }
    println!("API encerrada.");
}

/// Banco vazio: com `DADOS_DEMONSTRACAO=true` (padrão, desenvolvimento) cria os usuários e eventos
/// de exemplo; com `false` (produção) cria só o SuperAdmin de `ADMIN_NOME/ADMIN_CPF/ADMIN_EMAIL/ADMIN_SENHA`
/// — assim o servidor nunca fica com senhas de demonstração conhecidas.
async fn preparar_dados_iniciais(app: &Aplicacao) {
    let var = |nome: &str| std::env::var(nome).unwrap_or_default();
    if !var("DADOS_DEMONSTRACAO").eq_ignore_ascii_case("false") {
        match dados_exemplo::popular_se_vazio(app).await {
            Ok(true) => {
                println!("Banco vazio: dados de demonstração criados (DADOS_DEMONSTRACAO=true). Logins:");
                for (perfil, email, senha) in dados_exemplo::LOGINS {
                    println!("  {perfil:<12} {email:<26} senha: {senha}");
                }
            }
            Ok(false) => {}
            Err(e) => eprintln!("Aviso: não foi possível criar os dados de demonstração: {e}"),
        }
        return;
    }
    let nome = var("ADMIN_NOME");
    let nome = if nome.is_empty() { "Administrador".to_string() } else { nome };
    match dados_exemplo::criar_superadmin_se_vazio(app, &nome, &var("ADMIN_CPF"), &var("ADMIN_EMAIL"), &var("ADMIN_SENHA")).await {
        Ok(true) => println!("Banco vazio: SuperAdmin {} criado.", var("ADMIN_EMAIL")),
        Ok(false) => {}
        Err(e) => eprintln!("✖ Não foi possível criar o SuperAdmin ({e}). Confira ADMIN_CPF, ADMIN_EMAIL e ADMIN_SENHA no .env."),
    }
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
