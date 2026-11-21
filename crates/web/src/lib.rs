//! # Interface web
//!
//! Camada de apresentação: recebe as requisições do navegador, chama os casos de uso da
//! [`aplicacao`] e devolve páginas HTML. Não contém regras de negócio.
//!
//! - [`rotas`]: endereços e *handlers* (um módulo por perfil);
//! - [`sessao`]: login por cookie e mensagens de aviso;
//! - [`paginas`]: modelos das páginas (templates em `templates/`).

pub mod paginas;
pub mod rotas;
pub mod sessao;

use std::sync::Arc;

use aplicacao::Aplicacao;
use sessao::ArmazemSessoes;

/// Estado compartilhado por todas as requisições.
#[derive(Clone)]
pub struct Estado {
    pub app: Arc<Aplicacao>,
    pub sessoes: Arc<ArmazemSessoes>,
}

impl Estado {
    pub fn novo(app: Aplicacao) -> Self {
        Estado { app: Arc::new(app), sessoes: Arc::new(ArmazemSessoes::default()) }
    }
}
