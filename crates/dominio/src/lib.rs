//! # Domínio — Sistema de Cadastro e Venda de Ingressos (Equipe 11)
//!
//! Camada mais interna da arquitetura. Contém **somente regras de negócio**:
//! não conhece banco de dados, web ou terminal (princípio da Inversão de Dependência —
//! as outras camadas dependem desta, nunca o contrário).
//!
//! - [`valores`]: objetos de valor (CPF, CNPJ, e-mail, dinheiro) que se validam na criação;
//! - [`entidades`]: as entidades da seção 7.2 do relatório, com atributos privados e métodos;
//! - [`precos`]: padrão *Strategy* para o cálculo de inteira/meia-entrada;
//! - [`portas`]: interfaces (traits) que a infraestrutura implementa (repositórios, hash, relógio).

pub mod entidades;
pub mod erros;
pub mod portas;
pub mod precos;
pub mod valores;

pub use erros::{Erro, Resultado};
