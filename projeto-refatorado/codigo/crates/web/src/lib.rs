//! # Front-end — interface web
//!
//! Camada de apresentação (contêiner `front`): recebe as requisições do navegador, chama o
//! **back** pela API HTTP/JSON e devolve páginas HTML. Não contém regras de negócio nem acessa
//! o banco de dados.
//!
//! - [`backend`]: cliente HTTP da API (mesmas operações dos casos de uso);
//! - [`rotas`]: endereços e *handlers* (um módulo por perfil);
//! - [`sessao`]: login por cookie e mensagens de aviso;
//! - [`paginas`]: modelos das páginas (templates em `templates/`).

pub mod backend;
pub mod paginas;
pub mod rotas;
pub mod sessao;

use std::sync::Arc;

use backend::Backend;
use sessao::ArmazemSessoes;

/// Estado compartilhado por todas as requisições.
#[derive(Clone)]
pub struct Estado {
    pub back: Arc<Backend>,
    pub sessoes: Arc<ArmazemSessoes>,
}

impl Estado {
    /// `url_back`: endereço da API, ex.: `http://back:8080`.
    pub fn novo(url_back: &str, cookies_seguros: bool) -> Self {
        Estado { back: Arc::new(Backend::novo(url_back)), sessoes: Arc::new(ArmazemSessoes::novo(cookies_seguros)) }
    }
}
