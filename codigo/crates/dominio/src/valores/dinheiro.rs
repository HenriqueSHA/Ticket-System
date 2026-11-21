use std::fmt;
use std::iter::Sum;
use std::ops::Add;

use crate::{Erro, Resultado};

/// Valor monetário guardado em **centavos inteiros**.
///
/// Usar inteiros evita os erros de arredondamento do tipo real (0,1 + 0,2 ≠ 0,3 em ponto flutuante).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "i64", into = "i64")] // trafega como centavos; negativo é recusado
pub struct Dinheiro(i64);

impl Dinheiro {
    pub const ZERO: Dinheiro = Dinheiro(0);

    pub fn de_centavos(centavos: i64) -> Resultado<Self> {
        if centavos < 0 {
            Err(Erro::validacao("o valor não pode ser negativo"))
        } else {
            Ok(Dinheiro(centavos))
        }
    }

    /// Converte o texto digitado ("120", "120,50", "1.234,50" ou "120.50").
    pub fn de_texto(texto: &str) -> Resultado<Self> {
        let limpo = texto.trim().trim_start_matches("R$").trim().replace(' ', "");
        let normalizado = if limpo.contains(',') { limpo.replace('.', "").replace(',', ".") } else { limpo };
        let reais: f64 = normalizado
            .parse()
            .map_err(|_| Erro::validacao(format!("valor \"{}\" inválido (exemplo: 120,50)", texto.trim())))?;
        if !reais.is_finite() {
            return Err(Erro::validacao("valor inválido"));
        }
        Self::de_centavos((reais * 100.0).round() as i64)
    }

    pub fn centavos(&self) -> i64 {
        self.0
    }

    /// Aplica um desconto percentual (ex.: 0.5 = 50%), arredondando para o centavo.
    pub fn com_desconto(&self, fracao: f64) -> Dinheiro {
        Dinheiro((self.0 as f64 * (1.0 - fracao)).round() as i64)
    }

    /// Valor sem o "R$" (ex.: "1.234,50"), usado em campos de formulário.
    pub fn sem_simbolo(&self) -> String {
        let inteiro = (self.0 / 100).to_string();
        let mut com_pontos = String::new();
        for (i, c) in inteiro.chars().enumerate() {
            if i > 0 && (inteiro.len() - i).is_multiple_of(3) {
                com_pontos.push('.');
            }
            com_pontos.push(c);
        }
        format!("{com_pontos},{:02}", self.0 % 100)
    }
}

impl TryFrom<i64> for Dinheiro {
    type Error = Erro;
    fn try_from(centavos: i64) -> Resultado<Self> {
        Dinheiro::de_centavos(centavos)
    }
}
impl From<Dinheiro> for i64 {
    fn from(d: Dinheiro) -> i64 {
        d.0
    }
}

impl fmt::Display for Dinheiro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R$ {}", self.sem_simbolo())
    }
}

impl Add for Dinheiro {
    type Output = Dinheiro;
    fn add(self, outro: Dinheiro) -> Dinheiro {
        Dinheiro(self.0 + outro.0)
    }
}

impl Sum for Dinheiro {
    fn sum<I: Iterator<Item = Dinheiro>>(iter: I) -> Dinheiro {
        iter.fold(Dinheiro::ZERO, Add::add)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_texto_em_varios_formatos() {
        assert_eq!(Dinheiro::de_texto("120").unwrap().centavos(), 12000);
        assert_eq!(Dinheiro::de_texto("120,50").unwrap().centavos(), 12050);
        assert_eq!(Dinheiro::de_texto("R$ 1.234,50").unwrap().centavos(), 123450);
        assert_eq!(Dinheiro::de_texto("75.5").unwrap().centavos(), 7550);
        assert!(Dinheiro::de_texto("-1").is_err());
        assert!(Dinheiro::de_texto("abc").is_err());
    }

    #[test]
    fn formata_e_soma() {
        assert_eq!(Dinheiro::de_centavos(123456789).unwrap().to_string(), "R$ 1.234.567,89");
        assert_eq!(Dinheiro::ZERO.to_string(), "R$ 0,00");
        let total: Dinheiro = [18000, 9000].iter().map(|c| Dinheiro::de_centavos(*c).unwrap()).sum();
        assert_eq!(total.to_string(), "R$ 270,00");
    }
}
