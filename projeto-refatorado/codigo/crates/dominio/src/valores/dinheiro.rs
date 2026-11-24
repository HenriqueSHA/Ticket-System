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
        if centavos < 0 { Err(Erro::validacao("o valor não pode ser negativo")) } else { Ok(Dinheiro(centavos)) }
    }

    /// Converte o texto digitado ("120", "120,50", "1.234,50" ou "120.50") só com aritmética
    /// inteira: separa reais e centavos e calcula `reais × 100 + centavos`.
    pub fn de_texto(texto: &str) -> Resultado<Self> {
        let invalido = || Erro::validacao(format!("valor \"{}\" inválido (exemplo: 120,50)", texto.trim()));
        let limpo = texto.trim().trim_start_matches("R$").trim().replace(' ', "");
        if limpo.starts_with('-') {
            return Err(Erro::validacao("o valor não pode ser negativo"));
        }
        // com vírgula, o ponto é separador de milhar ("1.234,50"); sem vírgula, é o decimal ("120.50")
        let normalizado = if limpo.contains(',') { limpo.replace('.', "").replace(',', ".") } else { limpo };
        let (reais, centavos) = normalizado.split_once('.').unwrap_or((normalizado.as_str(), ""));
        let so_digitos = |t: &str| t.chars().all(|c| c.is_ascii_digit());
        if (reais.is_empty() && centavos.is_empty())
            || centavos.len() > 2
            || !so_digitos(reais)
            || !so_digitos(centavos)
        {
            return Err(invalido());
        }
        let reais: i64 = if reais.is_empty() { 0 } else { reais.parse().map_err(|_| invalido())? };
        let centavos: i64 = format!("{centavos:0<2}").parse().map_err(|_| invalido())?;
        let total = reais.checked_mul(100).and_then(|v| v.checked_add(centavos)).ok_or_else(invalido)?;
        Self::de_centavos(total)
    }

    pub fn centavos(&self) -> i64 {
        self.0
    }

    /// Aplica um desconto percentual (ex.: 50 = 50%), arredondando meio centavo para cima.
    #[must_use]
    pub fn com_desconto(&self, percentual: i64) -> Dinheiro {
        Dinheiro((self.0 * (100 - percentual.clamp(0, 100)) + 50) / 100)
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
        assert_eq!(Dinheiro::de_texto("120").unwrap().centavos(), 12_000);
        assert_eq!(Dinheiro::de_texto("120,50").unwrap().centavos(), 12_050);
        assert_eq!(Dinheiro::de_texto("R$ 1.234,50").unwrap().centavos(), 123_450);
        assert_eq!(Dinheiro::de_texto("75.5").unwrap().centavos(), 7550);
        assert!(Dinheiro::de_texto("-1").is_err());
        assert!(Dinheiro::de_texto("abc").is_err());
        assert!(Dinheiro::de_texto("1.234").is_err(), "três casas decimais é ambíguo");
        assert!(Dinheiro::de_texto("99999999999999999999").is_err(), "estouro");
        assert_eq!(Dinheiro::de_texto(",5").unwrap().centavos(), 50);
        assert_eq!(Dinheiro::de_texto("0,1").unwrap().centavos(), 10);
    }

    #[test]
    fn formata_e_soma() {
        assert_eq!(Dinheiro::de_centavos(123_456_789).unwrap().to_string(), "R$ 1.234.567,89");
        assert_eq!(Dinheiro::ZERO.to_string(), "R$ 0,00");
        let total: Dinheiro = [18000, 9000].iter().map(|c| Dinheiro::de_centavos(*c).unwrap()).sum();
        assert_eq!(total.to_string(), "R$ 270,00");
        assert_eq!(Dinheiro::de_centavos(12_345).unwrap().com_desconto(50).centavos(), 6_173);
    }
}
