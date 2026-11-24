//! Rotas do SuperAdmin: gestão de organizadores (Algoritmo 7).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use contrato::*;

use crate::{Autenticado, Corpo, EstadoApi, FalhaApi, RespostaApi};

pub async fn listar(State(e): State<EstadoApi>, a: Autenticado) -> RespostaApi<Vec<Organizador>> {
    Ok(Json(e.app.administracao.listar(&a.sessao).await?))
}

pub async fn cadastrar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Corpo(dados): Corpo<CadastroOrganizador>,
) -> Result<(StatusCode, Json<Organizador>), FalhaApi> {
    let org = e.app.administracao.cadastrar_organizador(&a.sessao, &dados).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn buscar(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> RespostaApi<Organizador> {
    Ok(Json(e.app.administracao.buscar(&a.sessao, id).await?))
}

pub async fn editar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
    Corpo(d): Corpo<OrganizadorEditado>,
) -> RespostaApi<Organizador> {
    Ok(Json(e.app.administracao.editar(&a.sessao, id, &d.razao_social, &d.contato).await?))
}

pub async fn ativar(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> RespostaApi<Organizador> {
    Ok(Json(e.app.administracao.ativar(&a.sessao, id).await?))
}

pub async fn bloquear(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> RespostaApi<Organizador> {
    Ok(Json(e.app.administracao.bloquear(&a.sessao, id).await?))
}

pub async fn remover(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> Result<StatusCode, FalhaApi> {
    e.app.administracao.remover(&a.sessao, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
