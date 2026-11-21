//! Páginas sem login: entrada, criação de conta e saída (Algoritmos 1 e 2).

use aplicacao::CadastroCliente;
use axum::Form;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use dominio::entidades::Perfil;
use serde::Deserialize;

use super::{redirecionar, render};
use crate::Estado;
use crate::paginas::{Layout, PaginaCadastro, PaginaEntrar};
use crate::sessao::Aviso;

/// Página inicial de cada perfil.
fn inicio_do_perfil(perfil: Perfil) -> &'static str {
    match perfil {
        Perfil::Cliente => "/eventos",
        Perfil::Organizador => "/organizador/eventos",
        Perfil::SuperAdmin => "/admin/organizadores",
    }
}

pub async fn inicio(State(estado): State<Estado>, jar: CookieJar) -> Redirect {
    match estado.sessoes.atual(&jar) {
        Some(s) => Redirect::to(inicio_do_perfil(s.perfil)),
        None => Redirect::to("/entrar"),
    }
}

pub async fn form_entrar(State(estado): State<Estado>, jar: CookieJar) -> Response {
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let pagina = PaginaEntrar { layout: Layout::novo("Entrar", None, aviso), email: String::new(), erro: None };
    (jar, render(pagina)).into_response()
}

#[derive(Deserialize)]
pub struct FormEntrar {
    email: String,
    senha: String,
}

/// Algoritmo 1: confere o login e direciona conforme o perfil.
pub async fn entrar(State(estado): State<Estado>, jar: CookieJar, Form(f): Form<FormEntrar>) -> Response {
    match estado.app.autenticacao.entrar(&f.email, &f.senha).await {
        Ok(sessao) => {
            let destino = inicio_do_perfil(sessao.perfil);
            let aviso = Aviso::sucesso(format!("Bem-vindo(a), {}!", sessao.nome));
            let jar = estado.sessoes.abrir(jar, sessao);
            redirecionar(&estado, jar, destino, aviso)
        }
        Err(e) => render(PaginaEntrar {
            layout: Layout::novo("Entrar", None, None),
            email: f.email,
            erro: Some(e.mensagem().to_string()),
        }),
    }
}

pub async fn form_cadastro() -> Response {
    render(PaginaCadastro {
        layout: Layout::novo("Criar conta", None, None),
        nome: String::new(),
        cpf: String::new(),
        email: String::new(),
        erro: None,
    })
}

#[derive(Deserialize)]
pub struct FormCadastro {
    nome: String,
    cpf: String,
    email: String,
    senha: String,
}

/// Algoritmo 2: se algum dado for inválido, o formulário volta preenchido com a mensagem.
pub async fn cadastrar(State(estado): State<Estado>, jar: CookieJar, Form(f): Form<FormCadastro>) -> Response {
    let dados = CadastroCliente { nome: f.nome, cpf: f.cpf, email: f.email, senha: f.senha };
    match estado.app.autenticacao.cadastrar_cliente(&dados).await {
        Ok(u) => {
            let aviso = Aviso::sucesso(format!("Conta criada! Entre com o e-mail {}.", u.email()));
            redirecionar(&estado, jar, "/entrar", aviso)
        }
        Err(e) => render(PaginaCadastro {
            layout: Layout::novo("Criar conta", None, None),
            nome: dados.nome,
            cpf: dados.cpf,
            email: dados.email,
            erro: Some(e.mensagem().to_string()),
        }),
    }
}

pub async fn sair(State(estado): State<Estado>, jar: CookieJar) -> Response {
    let jar = estado.sessoes.fechar(jar);
    redirecionar(&estado, jar, "/entrar", Aviso::sucesso("Você saiu do sistema."))
}
