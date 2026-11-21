//! Padrão **Strategy** para o cálculo do valor do ingresso.
//!
//! Cada política implementa o mesmo `trait`. Para criar um novo tipo de ingresso
//! (ex.: "Idoso", "Cortesia"), basta criar uma nova política — o código de venda não
//! muda (princípio Aberto/Fechado do SOLID).

use crate::valores::Dinheiro;

/// Meia-entrada: 50% de desconto (constante do Algoritmo 4).
pub const DESCONTO_MEIA: f64 = 0.50;

pub trait PoliticaPreco {
    fn calcular(&self, preco: Dinheiro) -> Dinheiro;
    fn descricao(&self) -> &'static str;
}

pub struct PrecoInteira;

impl PoliticaPreco for PrecoInteira {
    fn calcular(&self, preco: Dinheiro) -> Dinheiro {
        preco
    }
    fn descricao(&self) -> &'static str {
        "Inteira"
    }
}

pub struct PrecoMeiaEntrada;

impl PoliticaPreco for PrecoMeiaEntrada {
    fn calcular(&self, preco: Dinheiro) -> Dinheiro {
        preco.com_desconto(DESCONTO_MEIA)
    }
    fn descricao(&self) -> &'static str {
        "Meia-entrada"
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::entidades::TipoIngresso;

    #[test]
    fn meia_entrada_tem_50_por_cento() {
        let preco = Dinheiro::de_centavos(12000).unwrap();
        assert_eq!(TipoIngresso::Inteira.politica().calcular(preco).centavos(), 12000);
        assert_eq!(TipoIngresso::Meia.politica().calcular(preco).centavos(), 6000);
        let impar = Dinheiro::de_centavos(7551).unwrap();
        assert_eq!(PrecoMeiaEntrada.calcular(impar).centavos(), 3776); // arredonda para o centavo
    }
}
