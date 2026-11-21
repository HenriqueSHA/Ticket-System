use std::fmt;

use crate::{Erro, Resultado};

/// E-mail válido, sempre em minúsculas (assim "Ana@Email.com" e "ana@email.com" são o mesmo login).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")] // ao ler JSON, passa pela validação
pub struct Email(String);

impl Email {
    pub fn novo(texto: &str) -> Resultado<Self> {
        let email = texto.trim().to_lowercase();
        let valido = match email.split_once('@') {
            Some((usuario, dominio)) => {
                !usuario.is_empty()
                    && !email.contains(char::is_whitespace)
                    && dominio.contains('.')
                    && !dominio.starts_with('.')
                    && !dominio.ends_with('.')
                    && !dominio.contains('@')
            }
            None => false,
        };
        if valido { Ok(Email(email)) } else { Err(Erro::validacao("e-mail inválido (exemplo: nome@dominio.com)")) }
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Email {
    type Error = Erro;
    fn try_from(texto: String) -> Resultado<Self> {
        Email::novo(&texto)
    }
}
impl From<Email> for String {
    fn from(email: Email) -> String {
        email.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn valida_e_normaliza() {
        assert_eq!(Email::novo("  Ana@Email.COM ").unwrap().como_texto(), "ana@email.com");
        assert!(Email::novo("ana.email.com").is_err());
        assert!(Email::novo("@email.com").is_err());
        assert!(Email::novo("ana@email").is_err());
        assert!(Email::novo("hne enrique@gmail.com").is_err());
    }
}
