//! Páginas do organizador: eventos (Algoritmo 3), relatório e check-in (Algoritmo 6).

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::{CookieJar, Form};
use chrono::{NaiveDate, NaiveTime};
use dominio::entidades::{DadosEvento, NovaCategoria, StatusEvento};
use dominio::valores::Dinheiro;
use dominio::{Erro, Resultado};
use serde::Deserialize;

use super::{inteiro, redirecionar, render, tentar};
use crate::Estado;
use crate::paginas::*;
use crate::sessao::{Aviso, Logado};

pub async fn eventos(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar) -> Response {
    let lista = tentar!(estado.app.eventos.listar_do_organizador(&s).await, s, "/");
    let hoje = estado.app.hoje();
    let eventos = lista
        .into_iter()
        .map(|evento| LinhaEventoOrganizador { realizado: evento.ja_ocorreu(hoje), evento })
        .collect();
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    (jar, render(PaginaEventosOrganizador { layout: Layout::novo("Meus eventos", Some(&s), aviso), eventos }))
        .into_response()
}

/// Formulário do evento. Os campos de categoria vêm repetidos (uma linha por categoria).
#[derive(Deserialize)]
pub struct FormEvento {
    nome: String,
    #[serde(default)]
    descricao: String,
    local: String,
    data: String,
    horario: String,
    #[serde(default)]
    capacidade: String,
    permite_cancelamento: Option<String>,
    vendas_abertas: Option<String>,
    #[serde(default)]
    cat_nome: Vec<String>,
    #[serde(default)]
    cat_preco: Vec<String>,
    #[serde(default)]
    cat_assento: Vec<String>,
    #[serde(default)]
    cat_quantidade: Vec<String>,
    #[serde(default)]
    cat_fileiras: Vec<String>,
    #[serde(default)]
    cat_por_fileira: Vec<String>,
}

impl FormEvento {
    fn campos(&self) -> CamposEvento {
        CamposEvento {
            nome: self.nome.clone(),
            descricao: self.descricao.clone(),
            local: self.local.clone(),
            data: self.data.clone(),
            horario: self.horario.clone(),
            capacidade: self.capacidade.clone(),
            permite_cancelamento: self.permite_cancelamento.is_some(),
            vendas_abertas: self.vendas_abertas.is_some(),
        }
    }

    /// Linhas de categoria preenchidas (linhas totalmente vazias são ignoradas).
    fn categorias(&self) -> Vec<LinhaCategoria> {
        let pegar = |v: &Vec<String>, i: usize| v.get(i).cloned().unwrap_or_default();
        (0..self.cat_nome.len())
            .map(|i| LinhaCategoria {
                nome: pegar(&self.cat_nome, i),
                preco: pegar(&self.cat_preco, i),
                com_assento: pegar(&self.cat_assento, i) == "sim",
                quantidade: pegar(&self.cat_quantidade, i),
                fileiras: pegar(&self.cat_fileiras, i),
                por_fileira: pegar(&self.cat_por_fileira, i),
            })
            .filter(|l| !(l.nome.trim().is_empty() && l.preco.trim().is_empty()))
            .collect()
    }

    fn dados(&self, capacidade: Option<i32>) -> Resultado<DadosEvento> {
        let data = NaiveDate::parse_from_str(self.data.trim(), "%Y-%m-%d")
            .map_err(|_| Erro::validacao("informe a data do evento"))?;
        let horario = NaiveTime::parse_from_str(self.horario.trim(), "%H:%M")
            .map_err(|_| Erro::validacao("informe o horário do evento"))?;
        let capacidade = match capacidade {
            Some(c) => c,
            None => inteiro(&self.capacidade, "capacidade")?,
        };
        Ok(DadosEvento {
            nome: self.nome.clone(),
            descricao: self.descricao.clone(),
            local: self.local.clone(),
            data,
            horario,
            capacidade,
            permite_cancelamento: self.permite_cancelamento.is_some(),
        })
    }
}

fn categoria_do_formulario(l: &LinhaCategoria) -> Resultado<NovaCategoria> {
    let preco = Dinheiro::de_texto(&l.preco)?;
    if l.com_assento {
        NovaCategoria::com_assento(&l.nome, preco, inteiro(&l.fileiras, "fileiras")?, inteiro(&l.por_fileira, "assentos por fileira")?)
    } else {
        NovaCategoria::sem_assento(&l.nome, preco, inteiro(&l.quantidade, "quantidade")?)
    }
}

fn pagina_novo(s: &aplicacao::Sessao, campos: CamposEvento, categorias: Vec<LinhaCategoria>, erros: Vec<String>) -> Response {
    render(PaginaFormEvento {
        layout: Layout::novo("Novo evento", Some(s), None),
        acao: "/organizador/eventos/novo".into(),
        editando: false,
        campos,
        categorias,
        erros,
    })
}

pub async fn form_novo(State(estado): State<Estado>, Logado(s): Logado) -> Response {
    tentar!(estado.app.eventos.organizador_ativo(&s).await, s, "/");
    let campos = CamposEvento { permite_cancelamento: true, vendas_abertas: true, ..Default::default() };
    let categorias = vec![
        LinhaCategoria { nome: "Pista".into(), ..Default::default() },
        LinhaCategoria { nome: "Cadeira".into(), com_assento: true, ..Default::default() },
    ];
    pagina_novo(&s, campos, categorias, vec![])
}

/// Algoritmo 3 — valida os dados, depois cada categoria (laço) e grava tudo junto.
pub async fn criar(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar, Form(f): Form<FormEvento>) -> Response {
    let (campos, linhas) = (f.campos(), f.categorias());
    let mut erros = Vec::new();

    let builder = match f.dados(None) {
        Ok(dados) => estado.app.eventos.iniciar_cadastro(&s, dados).await,
        Err(e) => Err(e),
    };
    let mut builder = match builder {
        Ok(b) => b,
        Err(e) => return pagina_novo(&s, campos, linhas, vec![e.mensagem().to_string()]),
    };
    for (i, linha) in linhas.iter().enumerate() {
        let adicionada = categoria_do_formulario(linha).and_then(|c| builder.adicionar_categoria(c).map(|_| ()));
        if let Err(e) = adicionada {
            erros.push(format!("Categoria {} ({}): {}", i + 1, linha.nome.trim(), e.mensagem()));
        }
    }
    if !erros.is_empty() {
        return pagina_novo(&s, campos, linhas, erros);
    }
    let resultado = match builder.construir() {
        Ok(novo) => estado.app.eventos.cadastrar(&s, &novo).await,
        Err(e) => Err(e),
    };
    match resultado {
        Ok(evento) => redirecionar(
            &estado,
            jar,
            "/organizador/eventos",
            Aviso::sucesso(format!("Evento \"{}\" cadastrado (código {}).", evento.nome(), evento.id())),
        ),
        Err(e) => pagina_novo(&s, campos, linhas, vec![e.mensagem().to_string()]),
    }
}

pub async fn form_editar(State(estado): State<Estado>, Logado(s): Logado, Path(id): Path<i64>) -> Response {
    let e = tentar!(estado.app.eventos.evento_do_organizador(&s, id).await, s, "/organizador/eventos");
    let campos = CamposEvento {
        nome: e.nome().into(),
        descricao: e.descricao().into(),
        local: e.local().into(),
        data: e.data().format("%Y-%m-%d").to_string(),
        horario: e.horario().format("%H:%M").to_string(),
        capacidade: e.capacidade().to_string(),
        permite_cancelamento: e.permite_cancelamento(),
        vendas_abertas: e.status() == StatusEvento::Aberto,
    };
    render(PaginaFormEvento {
        layout: Layout::novo("Alterar evento", Some(&s), None),
        acao: format!("/organizador/eventos/{id}/editar"),
        editando: true,
        campos,
        categorias: vec![],
        erros: vec![],
    })
}

pub async fn editar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormEvento>,
) -> Response {
    let status = if f.vendas_abertas.is_some() { StatusEvento::Aberto } else { StatusEvento::Encerrado };
    let resultado = match f.dados(Some(1)) {
        Ok(dados) => estado.app.eventos.alterar(&s, id, dados, status).await,
        Err(e) => Err(e),
    };
    match resultado {
        Ok(e) => redirecionar(&estado, jar, "/organizador/eventos", Aviso::sucesso(format!("Evento \"{}\" alterado.", e.nome()))),
        Err(e) => render(PaginaFormEvento {
            layout: Layout::novo("Alterar evento", Some(&s), None),
            acao: format!("/organizador/eventos/{id}/editar"),
            editando: true,
            campos: f.campos(),
            categorias: vec![],
            erros: vec![e.mensagem().to_string()],
        }),
    }
}

/// Algoritmo 6 — relatório + mapa de cada setor com assento marcado.
pub async fn relatorio(State(estado): State<Estado>, Logado(s): Logado, Path(id): Path<i64>) -> Response {
    let relatorio = tentar!(estado.app.relatorios.relatorio(&s, id).await, s, "/organizador/eventos");
    let mut mapas = Vec::new();
    for c in relatorio.categorias.iter().filter(|c| c.com_assento()) {
        mapas.push(tentar!(estado.app.eventos.mapa(c).await, s, "/organizador/eventos"));
    }
    render(PaginaRelatorio { layout: Layout::novo("Relatório do evento", Some(&s), None), relatorio, mapas })
}

pub async fn form_checkin(State(estado): State<Estado>, Logado(s): Logado) -> Response {
    tentar!(estado.app.eventos.organizador_ativo(&s).await, s, "/");
    render(PaginaCheckin { layout: Layout::novo("Check-in", Some(&s), None), codigo: String::new(), resultado: None })
}

#[derive(Deserialize)]
pub struct FormCheckin {
    codigo: String,
}

/// Registro de presença: o número do ingresso é o número da compra.
pub async fn checkin(State(estado): State<Estado>, Logado(s): Logado, Form(f): Form<FormCheckin>) -> Response {
    let numero = f.codigo.trim().trim_start_matches('0');
    let resultado = match numero.parse::<i64>() {
        Ok(id) => match estado.app.relatorios.registrar_presenca(&s, id).await {
            Ok(v) => Aviso::sucesso(format!("Entrada liberada — ingresso nº {:06}.", v.id())),
            Err(e) => Aviso::erro(e.mensagem()),
        },
        Err(_) => Aviso::erro("digite o número do ingresso"),
    };
    render(PaginaCheckin { layout: Layout::novo("Check-in", Some(&s), None), codigo: String::new(), resultado: Some(resultado) })
}
