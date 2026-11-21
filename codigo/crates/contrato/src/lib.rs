//! # Contrato da API (front ↔ back)
//!
//! Tipos que trafegam em JSON entre o contêiner **front** (`web`) e o contêiner **back** (`api`).
//! Os dois lados compilam contra este mesmo crate, então uma mudança no formato quebra a
//! compilação em vez de quebrar em produção.
//!
//! As entidades do domínio (`Evento`, `Ingresso`, `Venda`...) e os dados de entrada da aplicação
//! (`Pedido`, `CadastroCliente`...) já são serializáveis e são reaproveitados aqui.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub use aplicacao::{CadastroCliente, CadastroOrganizador, Pedido, ResumoCompra, Sessao};
pub use dominio::entidades::{
    DadosEvento, Evento, Ingresso, ItemHistorico, MapaAssentos, Organizador, RelatorioEvento, Setor, StatusEvento,
    Venda, VendaDetalhada,
};
pub use dominio::valores::Dinheiro;
pub use dominio::{Erro, Resultado};

/// Cabeçalho que leva o token de login: `Authorization: Bearer <token>`.
pub const PREFIXO_TOKEN: &str = "Bearer ";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedidoLogin {
    pub email: String,
    pub senha: String,
}

/// Resposta do login: o token identifica a sessão nas próximas requisições.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespostaLogin {
    pub token: String,
    pub sessao: Sessao,
}

/// Usuário recém-cadastrado (sem dados sensíveis — o hash da senha nunca sai do back).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsuarioCriado {
    pub id: i64,
    pub nome: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoComCategorias {
    pub evento: Evento,
    pub categorias: Vec<Ingresso>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriaDoEvento {
    pub evento: Evento,
    pub ingresso: Ingresso,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompraConfirmada {
    pub venda: Venda,
    pub resumo: ResumoCompra,
}

/// Categoria enviada pelo organizador. O back a transforma em `NovaCategoria`
/// pelos construtores do domínio, que fazem todas as validações.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriaNova {
    pub nome: String,
    pub preco: Dinheiro,
    /// Usada quando `setor` é `None` (categoria sem assento marcado).
    pub quantidade: i32,
    /// `Some` = categoria com mapa de assentos (fileiras × assentos por fileira).
    pub setor: Option<Setor>,
}

/// Cadastro de evento (Algoritmo 3): dados + todas as categorias de uma vez.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoNovo {
    pub dados: DadosEvento,
    pub categorias: Vec<CategoriaNova>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoAlterado {
    pub dados: DadosEvento,
    pub status: StatusEvento,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriaAlterada {
    pub nome: String,
    pub preco: Dinheiro,
    /// `None` mantém a quantidade (e é o único valor aceito para categorias com assento).
    pub quantidade: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizadorEditado {
    pub razao_social: String,
    pub contato: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hoje {
    pub hoje: NaiveDate,
}

/// Separador usado quando o back devolve várias mensagens de validação em um só erro
/// (ex.: uma por categoria inválida no cadastro do evento).
pub const SEPARADOR_MENSAGENS: char = '\n';
