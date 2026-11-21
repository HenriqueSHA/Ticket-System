//! Entidades do sistema (tabela 7.2 do relatório).
//!
//! Em Rust, a "classe" é uma `struct` (atributos) + um bloco `impl` (métodos).
//! Os atributos são **privados**: fora deste módulo só é possível lê-los pelos métodos
//! de acesso e alterá-los pelos métodos que aplicam as regras (encapsulamento).

mod assento;
mod evento;
mod ingresso;
mod organizador;
mod relatorio;
mod usuario;
mod venda;

pub use assento::{Assento, CelulaMapa, LinhaMapa, MapaAssentos, Situacao, indice_fileira, letra_fileira};
pub use evento::{DadosEvento, Evento, EventoBuilder, NovoEvento, StatusEvento};
pub use ingresso::{Ingresso, MAX_ASSENTOS_POR_FILEIRA, MAX_FILEIRAS, NovaCategoria, Setor, total_disponivel};
pub use organizador::{NovoOrganizador, Organizador, StatusOrganizador};
pub use relatorio::RelatorioEvento;
pub use usuario::{Perfil, Usuario};
pub use venda::{FormaPagamento, ItemHistorico, NovaVenda, StatusVenda, TipoIngresso, Venda, VendaDetalhada};

/// Abstração comum aos enums gravados no banco como texto (ex.: `StatusVenda::Confirmada` ↔ "CONFIRMADA").
///
/// Cada enum implementa `codigo()` do seu jeito (polimorfismo); `de_codigo()` é herdado
/// pronto por todos (método padrão do trait).
pub trait Codificavel: Sized + Copy + 'static {
    const TODOS: &'static [Self];

    fn codigo(&self) -> &'static str;

    fn de_codigo(codigo: &str) -> Option<Self> {
        Self::TODOS.iter().copied().find(|v| v.codigo() == codigo)
    }
}
