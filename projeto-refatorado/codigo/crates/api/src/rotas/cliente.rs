//! Rotas do cliente: compra (Algoritmo 4), histórico, cancelamento (Algoritmo 5) e remarcação (5b).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use contrato::*;

use crate::{Autenticado, Corpo, EstadoApi, FalhaApi, RespostaApi};

/// Todas as verificações da compra, sem gravar (resumo antes de confirmar).
pub async fn previa(State(e): State<EstadoApi>, a: Autenticado, Corpo(p): Corpo<Pedido>) -> RespostaApi<ResumoCompra> {
    Ok(Json(e.app.vendas.preparar(&a.sessao, &p).await?))
}

/// Algoritmo 4 — registra a compra em transação.
pub async fn confirmar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Corpo(p): Corpo<Pedido>,
) -> Result<(StatusCode, Json<CompraConfirmada>), FalhaApi> {
    let (venda, resumo) = e.app.vendas.confirmar(&a.sessao, &p).await?;
    Ok((StatusCode::CREATED, Json(CompraConfirmada { venda, resumo })))
}

pub async fn historico(State(e): State<EstadoApi>, a: Autenticado) -> RespostaApi<Vec<ItemHistorico>> {
    Ok(Json(e.app.vendas.historico(&a.sessao).await?))
}

pub async fn comprovante(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
) -> RespostaApi<ItemHistorico> {
    Ok(Json(e.app.vendas.comprovante(&a.sessao, id).await?))
}

/// Algoritmo 5 — cancelar compra.
pub async fn cancelar(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> RespostaApi<Venda> {
    Ok(Json(e.app.vendas.cancelar(&a.sessao, id).await?))
}

/// Algoritmo 5b — remarcação (troca de assento).
pub async fn remarcar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
    Corpo(r): Corpo<Remarcacao>,
) -> RespostaApi<Venda> {
    Ok(Json(e.app.vendas.remarcar(&a.sessao, id, &r.fileira, r.numero).await?))
}
