//! Extrator de autenticação: lê o token do cabeçalho `Authorization`.

use aplicacao::Sessao;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use contrato::PREFIXO_TOKEN;
use dominio::Erro;

use crate::{EstadoApi, FalhaApi};

/// Sessão do usuário que fez a requisição (com o token usado).
pub struct Autenticado {
    pub sessao: Sessao,
    pub token: String,
}

impl FromRequestParts<EstadoApi> for Autenticado {
    type Rejection = FalhaApi;

    async fn from_request_parts(parts: &mut Parts, estado: &EstadoApi) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix(PREFIXO_TOKEN))
            .ok_or_else(|| Erro::acesso_negado("faça login para continuar"))?;
        let sessao =
            estado.sessao_do_token(token).ok_or_else(|| Erro::acesso_negado("sessão expirada. Entre novamente"))?;
        Ok(Autenticado { sessao, token: token.to_string() })
    }
}
