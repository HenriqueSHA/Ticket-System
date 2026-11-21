//! Sessão de login no front e avisos de uma única exibição.
//!
//! O **back** emite o token no login; o front o guarda num cookie `HttpOnly` e mantém um cache
//! token → sessão para não consultar o back a cada página. Se o front reiniciar, a sessão é
//! recuperada no back pelo token.

use std::collections::HashMap;
use std::sync::RwLock;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};

use contrato::Sessao as SessaoApi;
use dominio::entidades::Perfil;

use crate::Estado;

/// Usuário logado no front: os dados da sessão do back + o token para chamar a API.
#[derive(Debug, Clone)]
pub struct Sessao {
    pub token: String,
    pub usuario_id: i64,
    pub nome: String,
    pub perfil: Perfil,
    pub organizador_id: Option<i64>,
}

impl Sessao {
    pub fn nova(token: String, s: SessaoApi) -> Self {
        Sessao { token, usuario_id: s.usuario_id, nome: s.nome, perfil: s.perfil, organizador_id: s.organizador_id }
    }
}

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
    /// `true` atrás de HTTPS: o navegador só envia o cookie por conexão criptografada.
    cookies_seguros: bool,
}

fn novo_codigo() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

impl ArmazemSessoes {
    pub fn novo(cookies_seguros: bool) -> Self {
        ArmazemSessoes { cookies_seguros, ..Default::default() }
    }

    fn cookie(&self, nome: &'static str, valor: String) -> Cookie<'static> {
        Cookie::build((nome, valor))
            .path("/")
            .http_only(true)
            .secure(self.cookies_seguros)
            .same_site(SameSite::Lax)
            .build()
    }

    /// Guarda a sessão no cache e o token (emitido pelo back) no cookie.
    pub fn abrir(&self, jar: CookieJar, sessao: Sessao) -> CookieJar {
        let token = sessao.token.clone();
        self.sessoes.write().expect("sessões").insert(token.clone(), sessao);
        jar.add(self.cookie(COOKIE_SESSAO, token))
    }

    fn lembrar(&self, sessao: &Sessao) {
        self.sessoes.write().expect("sessões").insert(sessao.token.clone(), sessao.clone());
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
        jar.add(self.cookie(COOKIE_AVISO, codigo))
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
        if let Some(sessao) = estado.sessoes.atual(&jar) {
            return Ok(Logado(sessao));
        }
        // cookie com token, mas fora do cache (front reiniciou): pergunta ao back
        if let Some(token) = jar.get(COOKIE_SESSAO).map(|c| c.value().to_string())
            && let Ok(sessao) = estado.back.autenticacao.sessao(&token).await
        {
            estado.sessoes.lembrar(&sessao);
            return Ok(Logado(sessao));
        }
        Err(Redirect::to("/entrar").into_response())
    }
}
