//! CPF e CNPJ com validação dos dígitos verificadores (Algoritmo 2 do relatório).

use std::fmt;

use crate::{Erro, Resultado};

/// Remove pontuação: "529.982.247-25" -> "52998224725".
pub fn somente_digitos(texto: &str) -> String {
    texto.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Converte o texto em vetor de dígitos se tiver exatamente `tamanho` dígitos
/// e nenhum caractere além de dígitos e pontuação comum.
fn vetor_de_digitos(texto: &str, tamanho: usize) -> Option<Vec<u32>> {
    let texto = texto.trim();
    if texto.chars().any(|c| !(c.is_ascii_digit() || ".-/ ".contains(c))) {
        return None;
    }
    let d: Vec<u32> = texto.chars().filter_map(|c| c.to_digit(10)).collect();
    (d.len() == tamanho).then_some(d)
}

fn todos_iguais(d: &[u32]) -> bool {
    d.iter().all(|&x| x == d[0])
}

/// CPF válido, guardado somente com os 11 dígitos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpf(String);

impl Cpf {
    pub fn novo(texto: &str) -> Resultado<Self> {
        if Self::valido(texto) {
            Ok(Cpf(somente_digitos(texto)))
        } else {
            Err(Erro::validacao("CPF inválido (confira os 11 dígitos)"))
        }
    }

    /// Algoritmo 2 — validação pelos dígitos verificadores.
    pub fn valido(texto: &str) -> bool {
        let Some(d) = vetor_de_digitos(texto, 11) else { return false };
        if todos_iguais(&d) {
            return false; // ex.: 111.111.111-11 passa no cálculo, mas é inválido
        }
        // dígito = (soma dos dígitos × pesos decrescentes) × 10 mod 11; 10 vira 0
        let digito = |n: usize| {
            let soma: u32 = (0..n).map(|i| d[i] * (n as u32 + 1 - i as u32)).sum();
            let resto = (soma * 10) % 11;
            if resto == 10 { 0 } else { resto }
        };
        digito(9) == d[9] && digito(10) == d[10]
    }

    pub fn digitos(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cpf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.0;
        write!(f, "{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11])
    }
}

/// CNPJ válido, guardado somente com os 14 dígitos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnpj(String);

impl Cnpj {
    pub fn novo(texto: &str) -> Resultado<Self> {
        if Self::valido(texto) {
            Ok(Cnpj(somente_digitos(texto)))
        } else {
            Err(Erro::validacao("CNPJ inválido (confira os 14 dígitos)"))
        }
    }

    pub fn valido(texto: &str) -> bool {
        let Some(d) = vetor_de_digitos(texto, 14) else { return false };
        if todos_iguais(&d) {
            return false;
        }
        let digito = |pesos: &[u32]| {
            let soma: u32 = d.iter().zip(pesos).map(|(x, p)| x * p).sum();
            let resto = soma % 11;
            if resto < 2 { 0 } else { 11 - resto }
        };
        digito(&[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]) == d[12]
            && digito(&[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]) == d[13]
    }

    pub fn digitos(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Cnpj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = &self.0;
        write!(f, "{}.{}.{}/{}-{}", &d[0..2], &d[2..5], &d[5..8], &d[8..12], &d[12..14])
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cpf_valido_com_e_sem_pontuacao() {
        assert!(Cpf::valido("52998224725"));
        assert_eq!(Cpf::novo("529.982.247-25").unwrap().digitos(), "52998224725");
        assert_eq!(Cpf::novo("52998224725").unwrap().to_string(), "529.982.247-25");
    }

    #[test]
    fn cpf_invalido() {
        assert!(!Cpf::valido("52998224724")); // dígito verificador errado
        assert!(!Cpf::valido("11111111111")); // todos iguais
        assert!(!Cpf::valido("1234567890")); // 10 dígitos
        assert!(!Cpf::valido("12345678909 01")); // dígitos a mais
        assert!(!Cpf::valido("abc.def.ghi-jk"));
        assert!(Cpf::novo("98998998943").is_err());
    }

    #[test]
    fn cnpj() {
        assert!(Cnpj::valido("11.222.333/0001-81"));
        assert!(!Cnpj::valido("11.222.333/0001-82"));
        assert!(!Cnpj::valido("00000000000000"));
        assert_eq!(Cnpj::novo("11222333000181").unwrap().to_string(), "11.222.333/0001-81");
    }
}
