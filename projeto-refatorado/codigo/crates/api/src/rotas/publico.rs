//! Rotas sem login: entrada, cadastro e consulta de eventos à venda (Algoritmos 1, 2 e 4).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use contrato::*;
use dominio::entidades::MapaAssentos;

use crate::{Autenticado, Corpo, EstadoApi, FalhaApi, RespostaApi};

pub async fn hoje(State(e): State<EstadoApi>) -> Json<Hoje> {
    Json(Hoje { hoje: e.app.hoje() })
}

/// Algoritmo 1 — login.
pub async fn entrar(State(e): State<EstadoApi>, Corpo(p): Corpo<PedidoLogin>) -> RespostaApi<RespostaLogin> {
    let sessao = e.app.autenticacao.entrar(&p.email, &p.senha).await?;
    let token = e.emitir_token(sessao.clone());
    Ok(Json(RespostaLogin { token, sessao }))
}

pub async fn sair(State(e): State<EstadoApi>, a: Autenticado) -> StatusCode {
    e.revogar_token(&a.token);
    StatusCode::NO_CONTENT
}

pub async fn sessao(a: Autenticado) -> Json<Sessao> {
    Json(a.sessao)
}

/// Algoritmo 2 — cadastro de cliente.
pub async fn cadastrar_cliente(
    State(e): State<EstadoApi>,
    Corpo(dados): Corpo<CadastroCliente>,
) -> Result<(StatusCode, Json<UsuarioCriado>), FalhaApi> {
    let u = e.app.autenticacao.cadastrar_cliente(&dados).await?;
    let criado = UsuarioCriado { id: u.id(), nome: u.nome().to_string(), email: u.email().to_string() };
    Ok((StatusCode::CREATED, Json(criado)))
}

/// Cadastro público de organizador.
pub async fn cadastrar_organizador(
    State(e): State<EstadoApi>,
    Corpo(dados): Corpo<CadastroOrganizador>,
) -> Result<(StatusCode, Json<Organizador>), FalhaApi> {
    let org = e.app.autenticacao.cadastrar_organizador(&dados).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn eventos_disponiveis(State(e): State<EstadoApi>) -> RespostaApi<Vec<EventoComCategorias>> {
    let lista = e.app.eventos.listar_disponiveis().await?;
    Ok(Json(lista.into_iter().map(|(evento, categorias)| EventoComCategorias { evento, categorias }).collect()))
}

/// Algoritmo 4, passo 1: o evento está à venda?
pub async fn evento_para_venda(State(e): State<EstadoApi>, Path(id): Path<i64>) -> RespostaApi<Evento> {
    Ok(Json(e.app.vendas.evento_para_venda(id).await?))
}

pub async fn categorias(State(e): State<EstadoApi>, Path(id): Path<i64>) -> RespostaApi<Vec<Ingresso>> {
    Ok(Json(e.app.eventos.categorias(id).await?))
}

/// Algoritmo 4, passo 2: a categoria é do evento e ainda tem ingressos?
pub async fn categoria_para_venda(
    State(e): State<EstadoApi>,
    Path((id, categoria)): Path<(i64, i64)>,
) -> RespostaApi<Ingresso> {
    let evento = e.app.vendas.evento_para_venda(id).await?;
    Ok(Json(e.app.vendas.ingresso_para_venda(&evento, categoria).await?))
}

/// Mapa de assentos (matriz) de uma categoria com assento marcado.
pub async fn mapa(State(e): State<EstadoApi>, Path((id, categoria)): Path<(i64, i64)>) -> RespostaApi<MapaAssentos> {
    let ingresso = e
        .app
        .eventos
        .categorias(id)
        .await?
        .into_iter()
        .find(|c| c.id() == categoria)
        .ok_or_else(|| Erro::nao_encontrado("categoria inexistente neste evento"))?;
    Ok(Json(e.app.eventos.mapa(&ingresso).await?))
}
