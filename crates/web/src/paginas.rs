//! Modelos das páginas. Cada `struct` é ligada a um arquivo HTML da pasta `templates/`
//! (biblioteca Askama: o template é verificado na compilação).

use aplicacao::{ResumoCompra, Sessao};
use askama::Template;
#[allow(unused_imports)] // usado dentro dos templates (ex.: status().codigo())
use dominio::entidades::Codificavel;
use dominio::entidades::*;
use dominio::valores::Dinheiro;

use crate::sessao::Aviso;

/// Filtros usados nos templates: `{{ evento.data()|data }}`.
mod filters {
    use std::borrow::Borrow;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    // `Borrow` aceita tanto o valor quanto a referência (o Askama usa os dois).
    pub fn data(d: impl Borrow<NaiveDate>) -> askama::Result<String> {
        Ok(d.borrow().format("%d/%m/%Y").to_string())
    }
    pub fn hora(h: impl Borrow<NaiveTime>) -> askama::Result<String> {
        Ok(h.borrow().format("%H:%M").to_string())
    }
    pub fn data_hora(dh: impl Borrow<NaiveDateTime>) -> askama::Result<String> {
        Ok(dh.borrow().format("%d/%m/%Y %H:%M").to_string())
    }
}

pub struct ItemMenu {
    pub href: &'static str,
    pub texto: &'static str,
}

/// Partes comuns a todas as páginas (título, menu do perfil e aviso).
pub struct Layout {
    pub titulo: String,
    pub usuario: Option<String>,
    pub perfil: Option<&'static str>,
    pub menu: Vec<ItemMenu>,
    pub aviso: Option<Aviso>,
}

impl Layout {
    pub fn novo(titulo: &str, sessao: Option<&Sessao>, aviso: Option<Aviso>) -> Self {
        let menu = match sessao.map(|s| s.perfil) {
            Some(Perfil::Cliente) => vec![
                ItemMenu { href: "/eventos", texto: "Eventos" },
                ItemMenu { href: "/meus-ingressos", texto: "Meus ingressos" },
            ],
            Some(Perfil::Organizador) => vec![
                ItemMenu { href: "/organizador/eventos", texto: "Meus eventos" },
                ItemMenu { href: "/organizador/eventos/novo", texto: "Novo evento" },
                ItemMenu { href: "/organizador/checkin", texto: "Check-in" },
            ],
            Some(Perfil::SuperAdmin) => vec![
                ItemMenu { href: "/admin/organizadores", texto: "Organizadores" },
                ItemMenu { href: "/admin/organizadores/novo", texto: "Novo organizador" },
            ],
            None => vec![],
        };
        Layout {
            titulo: titulo.to_string(),
            usuario: sessao.map(|s| s.nome.clone()),
            perfil: sessao.map(|s| s.perfil.nome()),
            menu,
            aviso,
        }
    }
}

// ---------------------------- público ----------------------------

#[derive(Template)]
#[template(path = "entrar.html")]
pub struct PaginaEntrar {
    pub layout: Layout,
    pub email: String,
    pub erro: Option<String>,
}

#[derive(Template)]
#[template(path = "cadastro.html")]
pub struct PaginaCadastro {
    pub layout: Layout,
    pub nome: String,
    pub cpf: String,
    pub email: String,
    pub erro: Option<String>,
}

#[derive(Template)]
#[template(path = "cadastro_organizador.html")]
pub struct PaginaCadastroOrganizador {
    pub layout: Layout,
    pub razao_social: String,
    pub cnpj: String,
    pub contato: String,
    pub email: String,
    pub erro: Option<String>,
}

#[derive(Template)]
#[template(path = "erro.html")]
pub struct PaginaErro {
    pub layout: Layout,
    pub mensagem: String,
    pub voltar: String,
}

// ---------------------------- cliente ----------------------------

pub struct CartaoEvento {
    pub evento: Evento,
    pub categorias: Vec<Ingresso>,
    pub disponiveis: i32,
}

#[derive(Template)]
#[template(path = "cliente/eventos.html")]
pub struct PaginaEventos {
    pub layout: Layout,
    pub cartoes: Vec<CartaoEvento>,
}

#[derive(Template)]
#[template(path = "cliente/comprar.html")]
pub struct PaginaComprar {
    pub layout: Layout,
    pub evento: Evento,
    pub categorias: Vec<Ingresso>,
    pub disponiveis: i32,
    pub ingresso: Ingresso,
    pub mapa: Option<MapaAssentos>,
    pub preco_inteira: Dinheiro,
    pub preco_meia: Dinheiro,
}

#[derive(Template)]
#[template(path = "cliente/resumo.html")]
pub struct PaginaResumo {
    pub layout: Layout,
    pub resumo: ResumoCompra,
    pub assento_codigo: String,
}

#[derive(Template)]
#[template(path = "cliente/comprovante.html")]
pub struct PaginaComprovante {
    pub layout: Layout,
    pub item: ItemHistorico,
    pub cliente: String,
}

pub struct LinhaIngresso {
    pub item: ItemHistorico,
    pub pode_cancelar: bool,
    pub passado: bool,
}

#[derive(Template)]
#[template(path = "cliente/meus_ingressos.html")]
pub struct PaginaMeusIngressos {
    pub layout: Layout,
    pub linhas: Vec<LinhaIngresso>,
}

// ---------------------------- organizador ----------------------------

pub struct LinhaEventoOrganizador {
    pub evento: Evento,
    pub realizado: bool,
    pub disponiveis: i32,
}

#[derive(Template)]
#[template(path = "organizador/eventos.html")]
pub struct PaginaEventosOrganizador {
    pub layout: Layout,
    pub eventos: Vec<LinhaEventoOrganizador>,
}

/// Valores digitados no formulário de evento (texto, para poder devolvê-los em caso de erro).
#[derive(Default, Clone)]
pub struct CamposEvento {
    pub nome: String,
    pub descricao: String,
    pub local: String,
    pub data: String,
    pub horario: String,
    pub capacidade: String,
    pub permite_cancelamento: bool,
    pub vendas_abertas: bool,
}

#[derive(Default, Clone)]
pub struct LinhaCategoria {
    pub nome: String,
    pub preco: String,
    pub com_assento: bool,
    pub quantidade: String,
    pub fileiras: String,
    pub por_fileira: String,
}

#[derive(Template)]
#[template(path = "organizador/form_evento.html")]
pub struct PaginaFormEvento {
    pub layout: Layout,
    pub acao: String,
    pub editando: bool,
    pub campos: CamposEvento,
    pub categorias: Vec<LinhaCategoria>,
    pub erros: Vec<String>,
    /// Na alteração: a capacidade não pode ficar abaixo da soma das categorias.
    pub capacidade_minima: i32,
    pub voltar: String,
}

#[derive(Template)]
#[template(path = "organizador/gerenciar.html")]
pub struct PaginaGerenciarEvento {
    pub layout: Layout,
    pub evento: Evento,
    pub categorias: Vec<Ingresso>,
    /// Capacidade ainda não distribuída entre categorias.
    pub vagas_para_categorias: i32,
    /// Ingressos que ainda podem ser vendidos (soma das categorias).
    pub disponiveis: i32,
    pub realizado: bool,
    /// Valores do formulário "Adicionar categoria" (devolvidos em caso de erro).
    pub nova: LinhaCategoria,
    pub erro_categoria: Option<String>,
}

#[derive(Template)]
#[template(path = "organizador/form_categoria.html")]
pub struct PaginaFormCategoria {
    pub layout: Layout,
    pub evento: Evento,
    pub ingresso: Ingresso,
    pub nome: String,
    pub preco: String,
    pub quantidade: String,
    pub erro: Option<String>,
}

#[derive(Template)]
#[template(path = "organizador/relatorio.html")]
pub struct PaginaRelatorio {
    pub layout: Layout,
    pub relatorio: RelatorioEvento,
    pub mapas: Vec<MapaAssentos>,
    pub vendas: Vec<VendaDetalhada>,
}

#[derive(Template)]
#[template(path = "organizador/checkin.html")]
pub struct PaginaCheckin {
    pub layout: Layout,
    pub codigo: String,
    pub resultado: Option<Aviso>,
}

// ---------------------------- superadmin ----------------------------

#[derive(Template)]
#[template(path = "admin/organizadores.html")]
pub struct PaginaOrganizadores {
    pub layout: Layout,
    pub organizadores: Vec<Organizador>,
}

#[derive(Template)]
#[template(path = "admin/form_organizador.html")]
pub struct PaginaFormOrganizador {
    pub layout: Layout,
    pub acao: String,
    pub editando: bool,
    pub razao_social: String,
    pub cnpj: String,
    pub contato: String,
    pub email: String,
    pub erro: Option<String>,
}
