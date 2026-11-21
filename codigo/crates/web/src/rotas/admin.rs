//! Páginas do SuperAdmin: gestão de organizadores (Algoritmo 7).

use axum::Form;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use contrato::CadastroOrganizador;
use dominio::Erro;
use serde::Deserialize;

use super::{redirecionar, render, tentar};
use crate::Estado;
use crate::paginas::*;
use crate::sessao::{Aviso, Logado};

const LISTA: &str = "/admin/organizadores";

pub async fn lista(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar) -> Response {
    let organizadores = tentar!(estado.back.administracao.listar(&s).await, s, "/");
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    (jar, render(PaginaOrganizadores { layout: Layout::novo("Organizadores", Some(&s), aviso), organizadores }))
        .into_response()
}

pub async fn form_novo(Logado(s): Logado) -> Response {
    render(PaginaFormOrganizador {
        layout: Layout::novo("Novo organizador", Some(&s), None),
        acao: "/admin/organizadores/novo".into(),
        editando: false,
        razao_social: String::new(),
        cnpj: String::new(),
        contato: String::new(),
        email: String::new(),
        erro: None,
    })
}

#[derive(Deserialize)]
pub struct FormOrganizador {
    razao_social: String,
    #[serde(default)]
    cnpj: String,
    contato: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    senha: String,
}

pub async fn criar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Form(f): Form<FormOrganizador>,
) -> Response {
    let dados = CadastroOrganizador {
        razao_social: f.razao_social,
        cnpj: f.cnpj,
        contato: f.contato,
        email: f.email,
        senha: f.senha,
    };
    match estado.back.administracao.cadastrar_organizador(&s, &dados).await {
        Ok(org) => redirecionar(
            &estado,
            jar,
            LISTA,
            Aviso::sucesso(format!("Organizador \"{}\" cadastrado.", org.razao_social())),
        ),
        Err(e) => render(PaginaFormOrganizador {
            layout: Layout::novo("Novo organizador", Some(&s), None),
            acao: "/admin/organizadores/novo".into(),
            editando: false,
            razao_social: dados.razao_social,
            cnpj: dados.cnpj,
            contato: dados.contato,
            email: dados.email,
            erro: Some(e.mensagem().to_string()),
        }),
    }
}

pub async fn form_editar(State(estado): State<Estado>, Logado(s): Logado, Path(id): Path<i64>) -> Response {
    let org = tentar!(estado.back.administracao.buscar(&s, id).await, s, LISTA);
    render(PaginaFormOrganizador {
        layout: Layout::novo("Editar organizador", Some(&s), None),
        acao: format!("/admin/organizadores/{id}/editar"),
        editando: true,
        razao_social: org.razao_social().into(),
        cnpj: org.cnpj().to_string(),
        contato: org.contato().into(),
        email: String::new(),
        erro: None,
    })
}

pub async fn editar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormOrganizador>,
) -> Response {
    match estado.back.administracao.editar(&s, id, &f.razao_social, &f.contato).await {
        Ok(org) => redirecionar(
            &estado,
            jar,
            LISTA,
            Aviso::sucesso(format!("Dados de \"{}\" atualizados.", org.razao_social())),
        ),
        Err(e) => render(PaginaFormOrganizador {
            layout: Layout::novo("Editar organizador", Some(&s), None),
            acao: format!("/admin/organizadores/{id}/editar"),
            editando: true,
            razao_social: f.razao_social,
            cnpj: f.cnpj,
            contato: f.contato,
            email: String::new(),
            erro: Some(e.mensagem().to_string()),
        }),
    }
}

/// Ativar, bloquear ou remover (escolha do fluxograma 7).
pub async fn acao(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path((id, acao)): Path<(i64, String)>,
) -> Response {
    let adm = &estado.back.administracao;
    let resultado = match acao.as_str() {
        "ativar" => adm.ativar(&s, id).await.map(|o| format!("\"{}\" ativado.", o.razao_social())),
        "bloquear" => adm
            .bloquear(&s, id)
            .await
            .map(|o| format!("\"{}\" bloqueado. Ele não poderá mais acessar o sistema.", o.razao_social())),
        "remover" => adm.remover(&s, id).await.map(|_| "Organizador removido.".to_string()),
        _ => Err(Erro::validacao("opção inválida")),
    };
    let aviso = match resultado {
        Ok(msg) => Aviso::sucesso(msg),
        Err(e) => Aviso::erro(e.mensagem()),
    };
    redirecionar(&estado, jar, LISTA, aviso)
}
