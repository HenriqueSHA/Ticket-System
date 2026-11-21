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

use aplicacao::{Aplicacao, Sessao};
use axum::Json;
use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use dominio::Erro;

pub use seguranca::Autenticado;

/// Estado compartilhado por todas as requisições do back.
#[derive(Clone)]
pub struct EstadoApi {
    pub app: Arc<Aplicacao>,
    tokens: Arc<RwLock<HashMap<String, Sessao>>>,
}

impl EstadoApi {
    pub fn novo(app: Aplicacao) -> Self {
        EstadoApi { app: Arc::new(app), tokens: Arc::default() }
    }

    /// Abre uma sessão e devolve o token (código aleatório de 128 bits).
    pub fn emitir_token(&self, sessao: Sessao) -> String {
        let token = uuid::Uuid::new_v4().simple().to_string();
        self.tokens.write().expect("tokens").insert(token.clone(), sessao);
        token
    }

    pub fn sessao_do_token(&self, token: &str) -> Option<Sessao> {
        self.tokens.read().expect("tokens").get(token).cloned()
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

impl IntoResponse for FalhaApi {
    fn into_response(self) -> Response {
        (status_http(&self.0), Json(self.0)).into_response()
    }
}

/// Resposta padrão dos *handlers*.
pub type RespostaApi<T> = Result<Json<T>, FalhaApi>;

/// Corpo JSON da requisição; erros de leitura saem no formato da API.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(FalhaApi))]
pub struct Corpo<T>(pub T);
