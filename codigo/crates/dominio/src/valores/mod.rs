//! Objetos de valor: tipos pequenos que **só podem ser criados válidos**.
//!
//! Ex.: não existe um `Cpf` com dígito verificador errado — o construtor recusa.
//! Assim, qualquer função que recebe um `Cpf` não precisa validá-lo de novo.

mod dinheiro;
mod documento;
mod email;

pub use dinheiro::Dinheiro;
pub use documento::{Cnpj, Cpf, somente_digitos};
pub use email::Email;

use crate::{Erro, Resultado};

/// Garante que um texto obrigatório foi preenchido e remove espaços das pontas.
pub fn texto_obrigatorio(valor: &str, campo: &str) -> Resultado<String> {
    let valor = valor.trim();
    if valor.is_empty() {
        Err(Erro::validacao(format!("o campo \"{campo}\" é obrigatório")))
    } else {
        Ok(valor.to_string())
    }
}

/// Tamanho mínimo de senha aceito no cadastro.
pub const TAMANHO_MINIMO_SENHA: usize = 6;

pub fn validar_senha(senha: &str) -> Resultado<()> {
    if senha.chars().count() < TAMANHO_MINIMO_SENHA {
        Err(Erro::validacao(format!("a senha deve ter pelo menos {TAMANHO_MINIMO_SENHA} caracteres")))
    } else {
        Ok(())
    }
}
