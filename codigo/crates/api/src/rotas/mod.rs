//! Rotas da API. Cada rota chama um caso de uso da `Aplicacao` (Algoritmos 1 a 7).

mod admin;
mod cliente;
mod organizador;
mod publico;

use axum::Router;
use axum::routing::{get, post};

use crate::EstadoApi;

pub fn criar(estado: EstadoApi) -> Router {
    Router::new()
        // ---- públicas ----
        .route("/api/saude", get(|| async { "ok" }))
        .route("/api/hoje", get(publico::hoje))
        .route("/api/sessoes", post(publico::entrar).delete(publico::sair))
        .route("/api/sessao", get(publico::sessao))
        .route("/api/clientes", post(publico::cadastrar_cliente))
        .route("/api/organizadores", post(publico::cadastrar_organizador))
        .route("/api/eventos", get(publico::eventos_disponiveis))
        .route("/api/eventos/{id}/venda", get(publico::evento_para_venda))
        .route("/api/eventos/{id}/categorias", get(publico::categorias))
        .route("/api/eventos/{id}/categorias/{categoria}/venda", get(publico::categoria_para_venda))
        .route("/api/eventos/{id}/categorias/{categoria}/mapa", get(publico::mapa))
        // ---- cliente ----
        .route("/api/compras/previa", post(cliente::previa))
        .route("/api/compras", post(cliente::confirmar).get(cliente::historico))
        .route("/api/compras/{id}", get(cliente::comprovante))
        .route("/api/compras/{id}/cancelamento", post(cliente::cancelar))
        // ---- organizador ----
        .route("/api/organizador", get(organizador::eu))
        .route("/api/organizador/eventos", get(organizador::eventos).post(organizador::cadastrar))
        .route(
            "/api/organizador/eventos/{id}",
            get(organizador::gerenciar).put(organizador::alterar).delete(organizador::remover),
        )
        .route("/api/organizador/eventos/{id}/categorias", post(organizador::adicionar_categoria))
        .route(
            "/api/organizador/eventos/{id}/categorias/{categoria}",
            get(organizador::categoria).put(organizador::alterar_categoria).delete(organizador::remover_categoria),
        )
        .route("/api/organizador/eventos/{id}/relatorio", get(organizador::relatorio))
        .route("/api/organizador/eventos/{id}/vendas", get(organizador::vendas))
        .route("/api/organizador/vendas/{venda}/checkin", post(organizador::checkin))
        // ---- superadmin ----
        .route("/api/admin/organizadores", get(admin::listar).post(admin::cadastrar))
        .route("/api/admin/organizadores/{id}", get(admin::buscar).put(admin::editar).delete(admin::remover))
        .route("/api/admin/organizadores/{id}/ativar", post(admin::ativar))
        .route("/api/admin/organizadores/{id}/bloquear", post(admin::bloquear))
        .fallback(|| async { crate::FalhaApi(dominio::Erro::nao_encontrado("rota inexistente")) })
        .with_state(estado)
}

