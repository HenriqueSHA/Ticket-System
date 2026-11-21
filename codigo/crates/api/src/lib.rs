//! # Back-end — API REST/JSON
//!
//! Expõe os casos de uso da [`aplicacao`] por HTTP. É o **único** serviço que conhece o banco
//! de dados; o front (`web`) só conversa com ele por aqui.
//!
//! - autenticação: `POST /api/sessoes` devolve um token; as demais rotas protegidas exigem o
//!   cabeçalho `Authorization: Bearer <token>`;
//! - erros: sempre JSON `{"tipo": ..., "mensagem": ...}` com o código HTTP adequado.

pub mod rotas;
mod seguranca;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use aplicacao::{Aplicacao, Sessao};
use axum::Json;
use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use dominio::Erro;

pub use seguranca::Autenticado;

/// Tempo de validade de um login. Depois disso o usuário precisa entrar de novo.
pub const DURACAO_SESSAO: Duration = Duration::from_secs(8 * 60 * 60);

/// Estado compartilhado por todas as requisições do back.
#[derive(Clone)]
pub struct EstadoApi {
    pub app: Arc<Aplicacao>,
    /// token → (sessão, momento do login)
    tokens: Arc<RwLock<HashMap<String, (Sessao, Instant)>>>,
    validade: Duration,
}

impl EstadoApi {
    pub fn novo(app: Aplicacao) -> Self {
        Self::com_validade(app, DURACAO_SESSAO)
    }

    /// Validade do login configurável (usada nos testes para simular a expiração).
    pub fn com_validade(app: Aplicacao, validade: Duration) -> Self {
        EstadoApi { app: Arc::new(app), tokens: Arc::default(), validade }
    }

    /// Abre uma sessão e devolve o token (código aleatório de 128 bits).
    pub fn emitir_token(&self, sessao: Sessao) -> String {
        let token = uuid::Uuid::new_v4().simple().to_string();
        let mut tokens = self.tokens.write().expect("tokens");
        let validade = self.validade;
        tokens.retain(|_, (_, inicio)| inicio.elapsed() < validade); // limpa os vencidos
        tokens.insert(token.clone(), (sessao, Instant::now()));
        token
    }

    /// Sessão do token, se existir e ainda estiver dentro da validade.
    pub fn sessao_do_token(&self, token: &str) -> Option<Sessao> {
        let tokens = self.tokens.read().expect("tokens");
        let (sessao, inicio) = tokens.get(token)?;
        (inicio.elapsed() < self.validade).then(|| sessao.clone())
    }

    pub fn revogar_token(&self, token: &str) {
        self.tokens.write().expect("tokens").remove(token);
    }
}

/// Erro devolvido pela API: o `Erro` do domínio em JSON, com o código HTTP correspondente.
#[derive(Debug)]
pub struct FalhaApi(pub Erro);

impl From<Erro> for FalhaApi {
    fn from(e: Erro) -> Self {
        FalhaApi(e)
    }
}

/// JSON malformado ou com valor inválido (ex.: CPF inválido dentro do JSON) vira erro de validação.
impl From<JsonRejection> for FalhaApi {
    fn from(r: JsonRejection) -> Self {
        FalhaApi(Erro::validacao(format!("requisição inválida: {}", r.body_text())))
    }
}

pub fn status_http(erro: &Erro) -> StatusCode {
    match erro {
        Erro::Validacao(_) => StatusCode::UNPROCESSABLE_ENTITY,
        Erro::NaoEncontrado(_) => StatusCode::NOT_FOUND,
        Erro::RegraNegocio(_) => StatusCode::CONFLICT,
        Erro::AcessoNegado(_) => StatusCode::FORBIDDEN,
        Erro::Infraestrutura(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Mensagem mostrada no lugar dos detalhes de uma falha técnica.
pub const MENSAGEM_FALHA_INTERNA: &str = "erro interno no servidor. Tente novamente em instantes";

impl IntoResponse for FalhaApi {
    fn into_response(self) -> Response {
        let erro = match self.0 {
            // detalhes técnicos (ex.: texto do PostgreSQL) ficam só no log do servidor
            Erro::Infraestrutura(detalhe) => {
                eprintln!("[erro interno] {detalhe}");
                Erro::Infraestrutura(MENSAGEM_FALHA_INTERNA.to_string())
            }
            outro => outro,
        };
        (status_http(&erro), Json(erro)).into_response()
    }
}

/// Resposta padrão dos *handlers*.
pub type RespostaApi<T> = Result<Json<T>, FalhaApi>;

/// Corpo JSON da requisição; erros de leitura saem no formato da API.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(FalhaApi))]
pub struct Corpo<T>(pub T);

#[cfg(test)]
mod testes {
    use super::*;

    /// Detalhes técnicos (ex.: texto do banco) não podem chegar ao usuário.
    #[tokio::test]
    async fn falha_tecnica_nao_vaza_detalhes() {
        let resposta = FalhaApi(Erro::Infraestrutura("senha do banco: segredo123".into())).into_response();
        assert_eq!(resposta.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let corpo = axum::body::to_bytes(resposta.into_body(), 10_000).await.unwrap();
        let texto = String::from_utf8(corpo.to_vec()).unwrap();
        assert!(!texto.contains("segredo123"), "{texto}");
        assert!(texto.contains(MENSAGEM_FALHA_INTERNA));
    }

    #[tokio::test]
    async fn erros_de_negocio_continuam_com_a_mensagem() {
        let resposta = FalhaApi(Erro::regra("evento lotado")).into_response();
        assert_eq!(resposta.status(), StatusCode::CONFLICT);
        let corpo = axum::body::to_bytes(resposta.into_body(), 10_000).await.unwrap();
        assert!(String::from_utf8_lossy(&corpo).contains("evento lotado"));
    }
}
