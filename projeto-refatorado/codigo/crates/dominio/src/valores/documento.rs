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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")] // ao ler JSON, passa pela validação
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

/// CNPJ válido, guardado só com os 14 caracteres (sem pontuação, letras em maiúsculas).
///
/// Aceita o formato numérico tradicional e o **CNPJ alfanumérico** da Receita Federal
/// (emitido a partir de julho de 2026): os 12 primeiros caracteres podem ser letras ou números
/// e os 2 últimos são os dígitos verificadores. No cálculo, cada caractere vale o seu código
/// ASCII menos 48 ('0' = 0 … '9' = 9, 'A' = 17 … 'Z' = 42) — para CNPJs só com números o
/// resultado é o mesmo do cálculo antigo.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")] // ao ler JSON, passa pela validação
pub struct Cnpj(String);

impl Cnpj {
    pub fn novo(texto: &str) -> Resultado<Self> {
        match Self::normalizar(texto) {
            Some(c) if Self::digitos_conferem(&c) => Ok(Cnpj(c)),
            Some(_) => Err(Erro::validacao("CNPJ inválido: os dígitos verificadores (os 2 últimos) não conferem")),
            None => Err(Erro::validacao(
                "CNPJ inválido: informe 14 caracteres (ex.: 11.222.333/0001-81 ou 12.ABC.345/01DE-35)",
            )),
        }
    }

    pub fn valido(texto: &str) -> bool {
        Self::normalizar(texto).is_some_and(|c| Self::digitos_conferem(&c))
    }

    /// Remove pontuação e espaços e põe as letras em maiúsculas. `None` se não sobrarem
    /// exatamente 12 caracteres alfanuméricos + 2 dígitos.
    fn normalizar(texto: &str) -> Option<String> {
        let texto = texto.trim();
        if texto.chars().any(|c| !(c.is_ascii_alphanumeric() || ".-/ ".contains(c))) {
            return None;
        }
        let c: String = texto.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_uppercase()).collect();
        let valido = c.len() == 14 && c[12..].chars().all(|d| d.is_ascii_digit());
        valido.then_some(c)
    }

    fn digitos_conferem(c: &str) -> bool {
        let v: Vec<u32> = c.bytes().map(|b| u32::from(b) - 48).collect();
        if c.bytes().all(|b| b == c.as_bytes()[0]) {
            return false; // ex.: 00.000.000/0000-00
        }
        let digito = |pesos: &[u32]| {
            let soma: u32 = v.iter().zip(pesos).map(|(x, p)| x * p).sum();
            let resto = soma % 11;
            if resto < 2 { 0 } else { 11 - resto }
        };
        digito(&[5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]) == v[12]
            && digito(&[6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]) == v[13]
    }

    pub fn digitos(&self) -> &str {
        &self.0
    }
}

// Conversões usadas na (de)serialização: JSON → validação pelo construtor.
impl TryFrom<String> for Cpf {
    type Error = Erro;
    fn try_from(texto: String) -> Resultado<Self> {
        Cpf::novo(&texto)
    }
}
impl From<Cpf> for String {
    fn from(cpf: Cpf) -> String {
        cpf.0
    }
}
impl TryFrom<String> for Cnpj {
    type Error = Erro;
    fn try_from(texto: String) -> Resultado<Self> {
        Cnpj::novo(&texto)
    }
}
impl From<Cnpj> for String {
    fn from(cnpj: Cnpj) -> String {
        cnpj.0
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
    fn cnpj_alfanumerico() {
        // exemplo oficial da Receita Federal
        assert!(Cnpj::valido("12.ABC.345/01DE-35"));
        assert!(Cnpj::valido("12abc34501de35")); // minúsculas e sem pontuação
        assert_eq!(Cnpj::novo("12.abc.345/01de-35").unwrap().digitos(), "12ABC34501DE35");
        assert_eq!(Cnpj::novo("12ABC34501DE35").unwrap().to_string(), "12.ABC.345/01DE-35");
        assert!(!Cnpj::valido("12.ABC.345/01DE-36")); // dígito errado
        assert!(!Cnpj::valido("12.ABC.345/01DE-3X")); // verificador tem que ser número
        assert!(matches!(Cnpj::novo("12.345"), Err(Erro::Validacao(ref m)) if m.contains("14 caracteres")));
        assert!(matches!(Cnpj::novo("11.222.333/0001-82"), Err(Erro::Validacao(ref m)) if m.contains("verificadores")));
    }

    #[test]
    fn cnpj_reais() {
        assert!(Cnpj::valido("00.000.000/0001-91")); // Banco do Brasil
        assert!(Cnpj::valido("33.000.167/0001-01")); // Petrobras
    }

    #[test]
    fn cnpj() {
        assert!(Cnpj::valido("11.222.333/0001-81"));
        assert!(!Cnpj::valido("11.222.333/0001-82"));
        assert!(!Cnpj::valido("00000000000000"));
        assert_eq!(Cnpj::novo("11222333000181").unwrap().to_string(), "11.222.333/0001-81");
    }
}
