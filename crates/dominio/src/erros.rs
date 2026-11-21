//! Erros do sistema (seção 14 do relatório).
//!
//! Toda operação que pode falhar devolve `Resultado<T>`: o compilador obriga quem chama a
//! tratar o erro. Cada variante corresponde a um tipo de situação dos fluxogramas.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erro {
    /// Dado de entrada inválido (CPF errado, preço negativo, campo vazio...).
    Validacao(String),
    /// Registro procurado não existe (evento, venda, assento...).
    NaoEncontrado(String),
    /// Operação proibida pelas regras de negócio (assento ocupado, evento lotado...).
    RegraNegocio(String),
    /// Usuário sem permissão (login inválido, perfil errado, organizador bloqueado).
    AcessoNegado(String),
    /// Falha técnica (banco de dados fora do ar, etc.).
    Infraestrutura(String),
}

pub type Resultado<T> = Result<T, Erro>;

impl Erro {
    pub fn validacao(msg: impl Into<String>) -> Self {
        Erro::Validacao(msg.into())
    }
    pub fn nao_encontrado(msg: impl Into<String>) -> Self {
        Erro::NaoEncontrado(msg.into())
    }
    pub fn regra(msg: impl Into<String>) -> Self {
        Erro::RegraNegocio(msg.into())
    }
    pub fn acesso_negado(msg: impl Into<String>) -> Self {
        Erro::AcessoNegado(msg.into())
    }

    /// Mensagem para mostrar ao usuário.
    pub fn mensagem(&self) -> &str {
        match self {
            Erro::Validacao(m)
            | Erro::NaoEncontrado(m)
            | Erro::RegraNegocio(m)
            | Erro::AcessoNegado(m)
            | Erro::Infraestrutura(m) => m,
        }
    }
}

impl fmt::Display for Erro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tipo = match self {
            Erro::Validacao(_) => "Dado inválido",
            Erro::NaoEncontrado(_) => "Não encontrado",
            Erro::RegraNegocio(_) => "Operação não permitida",
            Erro::AcessoNegado(_) => "Acesso negado",
            Erro::Infraestrutura(_) => "Erro interno",
        };
        write!(f, "{tipo}: {}", self.mensagem())
    }
}

impl std::error::Error for Erro {}
