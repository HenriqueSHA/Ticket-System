//! Páginas do cliente: eventos, compra (Algoritmo 4), comprovante, histórico, cancelamento (Algoritmo 5)
//! e remarcação (Algoritmo 5b).

use axum::Form;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use contrato::Pedido;
use dominio::entidades::{Codificavel, FormaPagamento, StatusVenda, TipoIngresso, total_disponivel};
use dominio::{Erro, Resultado};
use serde::Deserialize;

use super::{hoje, redirecionar, render, tentar};
use crate::Estado;
use crate::paginas::*;
use crate::sessao::{Aviso, Logado};

pub async fn eventos(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar) -> Response {
    let lista = tentar!(estado.back.eventos.listar_disponiveis().await, s, "/");
    let cartoes = lista
        .into_iter()
        .map(|(evento, categorias)| CartaoEvento { disponiveis: total_disponivel(&categorias), evento, categorias })
        .collect();
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    (jar, render(PaginaEventos { layout: Layout::novo("Eventos disponíveis", Some(&s), aviso), cartoes }))
        .into_response()
}

#[derive(Deserialize)]
pub struct Escolha {
    categoria: Option<i64>,
}

/// Página de compra: escolha de categoria, tipo, assento (mapa) e pagamento.
pub async fn comprar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Query(escolha): Query<Escolha>,
) -> Response {
    let app = &estado.back;
    // fluxograma 4, passo 1: o evento está à venda?
    let evento = tentar!(app.vendas.evento_para_venda(id).await, s, "/eventos");
    let categorias = tentar!(app.eventos.categorias(id).await, s, "/eventos");
    let escolhida =
        escolha.categoria.or_else(|| categorias.iter().find(|c| !c.esgotado()).map(|c| c.id())).unwrap_or_default();
    // passo 2: a categoria existe e tem ingressos?
    let voltar = format!("/eventos/{id}");
    let ingresso = tentar!(app.vendas.ingresso_para_venda(&evento, escolhida).await, s, voltar);
    let mapa = if ingresso.com_assento() { Some(tentar!(app.eventos.mapa(&ingresso).await, s, voltar)) } else { None };

    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let pagina = PaginaComprar {
        layout: Layout::novo(evento.nome(), Some(&s), aviso),
        preco_inteira: TipoIngresso::Inteira.politica().calcular(ingresso.preco()),
        preco_meia: TipoIngresso::Meia.politica().calcular(ingresso.preco()),
        disponiveis: total_disponivel(&categorias),
        evento,
        categorias,
        ingresso,
        mapa,
    };
    (jar, render(pagina)).into_response()
}

#[derive(Deserialize)]
pub struct FormCompra {
    categoria: i64,
    tipo: Option<String>,
    assento: Option<String>,
    pagamento: Option<String>,
}

/// Converte o formulário no pedido (tipo, assento "B-2" e forma de pagamento).
fn montar_pedido(evento_id: i64, f: &FormCompra) -> Resultado<Pedido> {
    let tipo = f
        .tipo
        .as_deref()
        .and_then(TipoIngresso::de_codigo)
        .ok_or_else(|| Erro::validacao("escolha inteira ou meia-entrada"))?;
    let forma_pagamento = f
        .pagamento
        .as_deref()
        .and_then(FormaPagamento::de_codigo)
        .ok_or_else(|| Erro::validacao("escolha a forma de pagamento"))?;
    let assento = f.assento.as_deref().filter(|a| !a.is_empty()).map(ler_assento).transpose()?;
    Ok(Pedido { evento_id, ingresso_id: f.categoria, tipo, assento, forma_pagamento })
}

/// Lê o código do assento vindo do mapa ("B-2" → fileira B, número 2).
fn ler_assento(codigo: &str) -> Resultado<(String, i32)> {
    let (fileira, numero) = codigo.split_once('-').ok_or_else(|| Erro::validacao("assento inválido"))?;
    Ok((fileira.to_string(), numero.parse().map_err(|_| Erro::validacao("assento inválido"))?))
}

/// Mostra o resumo antes de confirmar (todas as verificações do fluxograma 4, sem gravar).
pub async fn resumo(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormCompra>,
) -> Response {
    let voltar = format!("/eventos/{id}?categoria={}", f.categoria);
    let preparado = match montar_pedido(id, &f) {
        Ok(pedido) => estado.back.vendas.preparar(&s, &pedido).await,
        Err(e) => Err(e),
    };
    match preparado {
        Ok(resumo) => render(PaginaResumo {
            layout: Layout::novo("Confirmar compra", Some(&s), None),
            assento_codigo: f.assento.unwrap_or_default(),
            resumo,
        }),
        Err(e) => redirecionar(&estado, jar, &voltar, Aviso::erro(e.mensagem())),
    }
}

/// Algoritmo 4 — grava a compra (transação) e mostra o comprovante.
pub async fn confirmar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormCompra>,
) -> Response {
    let voltar = format!("/eventos/{id}?categoria={}", f.categoria);
    let confirmado = match montar_pedido(id, &f) {
        Ok(pedido) => estado.back.vendas.confirmar(&s, &pedido).await,
        Err(e) => Err(e),
    };
    match confirmado {
        Ok((venda, _)) => {
            redirecionar(&estado, jar, &format!("/compras/{}", venda.id()), Aviso::sucesso("Compra realizada!"))
        }
        Err(e) => redirecionar(&estado, jar, &voltar, Aviso::erro(e.mensagem())),
    }
}

pub async fn comprovante(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let item = tentar!(estado.back.vendas.comprovante(&s, id).await, s, "/meus-ingressos");
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let pagina =
        PaginaComprovante { layout: Layout::novo("Comprovante", Some(&s), aviso), cliente: s.nome.clone(), item };
    (jar, render(pagina)).into_response()
}

pub async fn meus_ingressos(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar) -> Response {
    let itens = tentar!(estado.back.vendas.historico(&s).await, s, "/eventos");
    let hoje = hoje(&estado).await;
    let linhas = itens
        .into_iter()
        .map(|item| {
            let passado = item.data_evento <= hoje;
            let pode_cancelar = !passado && item.permite_cancelamento && item.venda.status() == StatusVenda::Confirmada;
            let pode_remarcar = pode_cancelar && item.assento.is_some();
            LinhaIngresso { item, pode_cancelar, pode_remarcar, passado }
        })
        .collect();
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    (jar, render(PaginaMeusIngressos { layout: Layout::novo("Meus ingressos", Some(&s), aviso), linhas }))
        .into_response()
}

/// Algoritmo 5 — cancelar compra.
pub async fn cancelar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let aviso = match estado.back.vendas.cancelar(&s, id).await {
        Ok(v) => Aviso::sucesso(format!("Compra nº {} cancelada. Valor a estornar: {}.", v.id(), v.valor())),
        Err(e) => Aviso::erro(e.mensagem()),
    };
    redirecionar(&estado, jar, "/meus-ingressos", aviso)
}

/// Algoritmo 5b — página de remarcação: mapa da mesma categoria para escolher o novo assento.
pub async fn remarcacao(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let app = &estado.back;
    let voltar = "/meus-ingressos";
    let item = tentar!(app.vendas.comprovante(&s, id).await, s, voltar);
    let categorias = tentar!(app.eventos.categorias(item.venda.evento_id()).await, s, voltar);
    let Some(ingresso) = categorias.into_iter().find(|c| c.id() == item.venda.ingresso_id()) else {
        return redirecionar(&estado, jar, voltar, Aviso::erro("categoria da compra não encontrada"));
    };
    let mapa = tentar!(app.eventos.mapa(&ingresso).await, s, voltar);
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let pagina = PaginaRemarcar { layout: Layout::novo("Trocar assento", Some(&s), aviso), item, mapa };
    (jar, render(pagina)).into_response()
}

#[derive(Deserialize)]
pub struct FormRemarcar {
    assento: Option<String>,
}

/// Algoritmo 5b — grava a troca de assento (transação) e volta ao comprovante.
pub async fn remarcar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormRemarcar>,
) -> Response {
    let codigo = f.assento.unwrap_or_default();
    let resultado = match ler_assento(&codigo) {
        Ok((fileira, numero)) => estado.back.vendas.remarcar(&s, id, &fileira, numero).await,
        Err(_) => Err(Erro::validacao("escolha o novo assento no mapa")),
    };
    match resultado {
        Ok(_) => redirecionar(
            &estado,
            jar,
            &format!("/compras/{id}"),
            Aviso::sucesso(format!("Assento trocado para {}.", codigo.replace('-', ""))),
        ),
        Err(e) => redirecionar(&estado, jar, &format!("/compras/{id}/remarcar"), Aviso::erro(e.mensagem())),
    }
}
