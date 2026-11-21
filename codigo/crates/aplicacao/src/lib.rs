//! # Aplicação — casos de uso do sistema
//!
//! Cada serviço implementa um grupo de algoritmos da seção 8 do relatório:
//!
//! | Serviço                   | Algoritmos                                   |
//! |---------------------------|----------------------------------------------|
//! | [`ServicoAutenticacao`]   | 1 (login e perfil), 2 (cadastro de cliente) e cadastro de organizador |
//! | [`ServicoEventos`]        | 3 (cadastro de evento) e mapa de assentos    |
//! | [`ServicoVendas`]         | 4 (compra) e 5 (cancelamento)                |
//! | [`ServicoRelatorios`]     | 6 (relatório e check-in)                     |
//! | [`ServicoAdministracao`]  | 7 (gestão de organizadores)                  |
//!
//! Os serviços recebem as dependências por **injeção** (construtor) e só conhecem os
//! `trait`s do domínio — nunca o PostgreSQL (Inversão de Dependência).

mod cadastro_organizador;
pub mod dados_exemplo;
mod fabrica;
mod servicos;
mod sessao;

use std::sync::Arc;

use dominio::portas::*;

pub use cadastro_organizador::CadastroOrganizador;
pub use fabrica::FabricaUsuarios;
pub use servicos::administracao::ServicoAdministracao;
pub use servicos::autenticacao::{CadastroCliente, ServicoAutenticacao};
pub use servicos::eventos::ServicoEventos;
pub use servicos::relatorios::ServicoRelatorios;
pub use servicos::vendas::{Pedido, ResumoCompra, ServicoVendas};
pub use sessao::Sessao;

/// Dependências externas que a aplicação precisa (implementações das portas).
pub struct Dependencias {
    pub usuarios: Arc<dyn RepositorioUsuarios>,
    pub organizadores: Arc<dyn RepositorioOrganizadores>,
    pub eventos: Arc<dyn RepositorioEventos>,
    pub vendas: Arc<dyn RepositorioVendas>,
    pub hash: Arc<dyn HashSenha>,
    pub relogio: Arc<dyn Relogio>,
}

/// Padrão **Facade**: ponto único de acesso aos casos de uso para a interface.
pub struct Aplicacao {
    pub autenticacao: ServicoAutenticacao,
    pub eventos: ServicoEventos,
    pub vendas: ServicoVendas,
    pub relatorios: ServicoRelatorios,
    pub administracao: ServicoAdministracao,
    usuarios: Arc<dyn RepositorioUsuarios>,
    fabrica: FabricaUsuarios,
    relogio: Arc<dyn Relogio>,
}

impl Aplicacao {
    /// Monta todos os serviços, entregando a cada um só as dependências de que precisa.
    pub fn nova(d: Dependencias) -> Self {
        let fabrica = FabricaUsuarios::new(d.hash.clone());
        Aplicacao {
            autenticacao: ServicoAutenticacao::new(d.usuarios.clone(), d.organizadores.clone(), fabrica.clone()),
            eventos: ServicoEventos::new(d.eventos.clone(), d.organizadores.clone(), d.relogio.clone()),
            vendas: ServicoVendas::new(d.eventos.clone(), d.vendas.clone(), d.relogio.clone()),
            relatorios: ServicoRelatorios::new(d.eventos.clone(), d.vendas.clone(), d.organizadores.clone()),
            administracao: ServicoAdministracao::new(d.usuarios.clone(), d.organizadores.clone(), fabrica.clone()),
            usuarios: d.usuarios,
            fabrica,
            relogio: d.relogio,
        }
    }

    pub fn hoje(&self) -> chrono::NaiveDate {
        self.relogio.hoje()
    }
}
