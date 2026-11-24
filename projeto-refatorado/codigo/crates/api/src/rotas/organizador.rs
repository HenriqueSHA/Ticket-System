//! Rotas do organizador: cadastro (Algoritmo 3) e gestão de eventos, relatório e check-in (Algoritmo 6).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use contrato::*;
use dominio::entidades::NovaCategoria;

use crate::{Autenticado, Corpo, EstadoApi, FalhaApi, RespostaApi};

/// Converte a categoria recebida pelos construtores do domínio (que validam tudo).
fn nova_categoria(c: &CategoriaNova) -> Resultado<NovaCategoria> {
    match c.setor {
        Some(s) => NovaCategoria::com_assento(&c.nome, c.preco, s.fileiras, s.assentos_por_fileira),
        None => NovaCategoria::sem_assento(&c.nome, c.preco, c.quantidade),
    }
}

pub async fn eu(State(e): State<EstadoApi>, a: Autenticado) -> RespostaApi<Organizador> {
    Ok(Json(e.app.eventos.organizador_ativo(&a.sessao).await?))
}

pub async fn eventos(State(e): State<EstadoApi>, a: Autenticado) -> RespostaApi<Vec<EventoComCategorias>> {
    let lista = e.app.eventos.listar_do_organizador_com_categorias(&a.sessao).await?;
    Ok(Json(lista.into_iter().map(|(evento, categorias)| EventoComCategorias { evento, categorias }).collect()))
}

/// Algoritmo 3: valida os dados, depois **cada** categoria (laço) — devolve todas as mensagens
/// de uma vez, separadas por `SEPARADOR_MENSAGENS` — e só então grava.
pub async fn cadastrar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Corpo(novo): Corpo<EventoNovo>,
) -> Result<(StatusCode, Json<Evento>), FalhaApi> {
    let mut builder = e.app.eventos.iniciar_cadastro(&a.sessao, novo.dados).await?;
    let mut erros = Vec::new();
    for (i, c) in novo.categorias.iter().enumerate() {
        if let Err(erro) = nova_categoria(c).and_then(|n| builder.adicionar_categoria(n).map(|_| ())) {
            erros.push(format!("Categoria {} ({}): {}", i + 1, c.nome.trim(), erro.mensagem()));
        }
    }
    if !erros.is_empty() {
        return Err(Erro::validacao(erros.join(&SEPARADOR_MENSAGENS.to_string())).into());
    }
    let evento = e.app.eventos.cadastrar(&a.sessao, &builder.construir()?).await?;
    Ok((StatusCode::CREATED, Json(evento)))
}

pub async fn gerenciar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
) -> RespostaApi<EventoComCategorias> {
    let (evento, categorias) = e.app.eventos.gerenciar(&a.sessao, id).await?;
    Ok(Json(EventoComCategorias { evento, categorias }))
}

pub async fn alterar(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
    Corpo(alt): Corpo<EventoAlterado>,
) -> RespostaApi<Evento> {
    Ok(Json(e.app.eventos.alterar(&a.sessao, id, alt.dados, alt.status).await?))
}

pub async fn remover(State(e): State<EstadoApi>, a: Autenticado, Path(id): Path<i64>) -> Result<StatusCode, FalhaApi> {
    e.app.eventos.remover(&a.sessao, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn adicionar_categoria(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
    Corpo(c): Corpo<CategoriaNova>,
) -> Result<(StatusCode, Json<Ingresso>), FalhaApi> {
    let ingresso = e.app.eventos.adicionar_categoria(&a.sessao, id, nova_categoria(&c)?).await?;
    Ok((StatusCode::CREATED, Json(ingresso)))
}

pub async fn categoria(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path((id, categoria)): Path<(i64, i64)>,
) -> RespostaApi<CategoriaDoEvento> {
    let (evento, ingresso) = e.app.eventos.categoria_do_organizador(&a.sessao, id, categoria).await?;
    Ok(Json(CategoriaDoEvento { evento, ingresso }))
}

pub async fn alterar_categoria(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path((id, categoria)): Path<(i64, i64)>,
    Corpo(alt): Corpo<CategoriaAlterada>,
) -> RespostaApi<Ingresso> {
    let ingresso =
        e.app.eventos.alterar_categoria(&a.sessao, id, categoria, &alt.nome, alt.preco, alt.quantidade).await?;
    Ok(Json(ingresso))
}

pub async fn remover_categoria(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path((id, categoria)): Path<(i64, i64)>,
) -> Result<StatusCode, FalhaApi> {
    e.app.eventos.remover_categoria(&a.sessao, id, categoria).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Algoritmo 6 — relatório do evento.
pub async fn relatorio(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
) -> RespostaApi<RelatorioEvento> {
    Ok(Json(e.app.relatorios.relatorio(&a.sessao, id).await?))
}

pub async fn vendas(
    State(e): State<EstadoApi>,
    a: Autenticado,
    Path(id): Path<i64>,
) -> RespostaApi<Vec<VendaDetalhada>> {
    Ok(Json(e.app.relatorios.vendas_do_evento(&a.sessao, id).await?))
}

/// Check-in: registra a entrada do ingresso.
pub async fn checkin(State(e): State<EstadoApi>, a: Autenticado, Path(venda): Path<i64>) -> RespostaApi<Venda> {
    Ok(Json(e.app.relatorios.registrar_presenca(&a.sessao, venda).await?))
}
