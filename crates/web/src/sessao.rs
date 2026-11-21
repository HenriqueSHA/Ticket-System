//! Sessões de login (cookie com um código aleatório) e avisos de uma única exibição.

use std::collections::HashMap;
use std::sync::RwLock;

use aplicacao::Sessao;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::Estado;

pub const COOKIE_SESSAO: &str = "sessao";
pub const COOKIE_AVISO: &str = "aviso";

/// Mensagem mostrada no topo da próxima página (ex.: "Compra realizada!").
#[derive(Debug, Clone)]
pub struct Aviso {
    pub sucesso: bool,
    pub texto: String,
}

impl Aviso {
    pub fn sucesso(texto: impl Into<String>) -> Self {
        Aviso { sucesso: true, texto: texto.into() }
    }
    pub fn erro(texto: impl Into<String>) -> Self {
        Aviso { sucesso: false, texto: texto.into() }
    }
}

/// Guarda as sessões abertas e os avisos pendentes, indexados pelo código do cookie.
#[derive(Default)]
pub struct ArmazemSessoes {
    sessoes: RwLock<HashMap<String, Sessao>>,
    avisos: RwLock<HashMap<String, Aviso>>,
}

fn novo_codigo() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn cookie(nome: &'static str, valor: String) -> Cookie<'static> {
    Cookie::build((nome, valor)).path("/").http_only(true).same_site(SameSite::Lax).build()
}

impl ArmazemSessoes {
    pub fn abrir(&self, jar: CookieJar, sessao: Sessao) -> CookieJar {
        let codigo = novo_codigo();
        self.sessoes.write().expect("sessões").insert(codigo.clone(), sessao);
        jar.add(cookie(COOKIE_SESSAO, codigo))
    }

    pub fn fechar(&self, jar: CookieJar) -> CookieJar {
        if let Some(c) = jar.get(COOKIE_SESSAO) {
            self.sessoes.write().expect("sessões").remove(c.value());
        }
        jar.remove(Cookie::build(COOKIE_SESSAO).path("/"))
    }

    pub fn atual(&self, jar: &CookieJar) -> Option<Sessao> {
        let codigo = jar.get(COOKIE_SESSAO)?;
        self.sessoes.read().expect("sessões").get(codigo.value()).cloned()
    }

    /// Guarda um aviso para ser mostrado na próxima página.
    pub fn avisar(&self, jar: CookieJar, aviso: Aviso) -> CookieJar {
        let codigo = novo_codigo();
        self.avisos.write().expect("avisos").insert(codigo.clone(), aviso);
        jar.add(cookie(COOKIE_AVISO, codigo))
    }

    /// Retira o aviso pendente (ele só aparece uma vez).
    pub fn retirar_aviso(&self, jar: CookieJar) -> (CookieJar, Option<Aviso>) {
        match jar.get(COOKIE_AVISO).map(|c| c.value().to_string()) {
            Some(codigo) => {
                let aviso = self.avisos.write().expect("avisos").remove(&codigo);
                (jar.remove(Cookie::build(COOKIE_AVISO).path("/")), aviso)
            }
            None => (jar, None),
        }
    }
}

/// Extrator: só deixa o *handler* rodar se houver login; senão manda para a tela de entrada.
pub struct Logado(pub Sessao);

impl FromRequestParts<Estado> for Logado {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, estado: &Estado) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        estado.sessoes.atual(&jar).map(Logado).ok_or_else(|| Redirect::to("/entrar").into_response())
    }
}
