use super::{Evento, Ingresso, StatusVenda, TipoIngresso, Venda};
use crate::valores::Dinheiro;

/// Relatório de um evento para o organizador (Algoritmo 6).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelatorioEvento {
    pub evento: Evento,
    pub vendidos: i32,
    pub cancelados: i32,
    pub presentes: i32,
    pub inteiras: i32,
    pub meias: i32,
    pub receita: Dinheiro,
    /// Percentual de ocupação (0 a 100).
    pub ocupacao: f64,
    pub categorias: Vec<Ingresso>,
}

impl RelatorioEvento {
    /// Percorre as vendas do evento acumulando contadores — o laço do fluxograma 6.
    pub fn calcular(evento: Evento, vendas: &[Venda], categorias: Vec<Ingresso>) -> Self {
        let (mut vendidos, mut cancelados, mut presentes, mut inteiras, mut meias) = (0, 0, 0, 0, 0);
        let mut receita = Dinheiro::ZERO;

        for v in vendas {
            if v.status() == StatusVenda::Cancelada {
                cancelados += 1;
                continue;
            }
            vendidos += 1;
            receita = receita + v.valor();
            match v.tipo() {
                TipoIngresso::Inteira => inteiras += 1,
                TipoIngresso::Meia => meias += 1,
            }
            if v.status() == StatusVenda::Utilizada {
                presentes += 1;
            }
        }

        let ocupacao = f64::from(vendidos) / f64::from(evento.capacidade()) * 100.0;
        RelatorioEvento { evento, vendidos, cancelados, presentes, inteiras, meias, receita, ocupacao, categorias }
    }

    /// Ocupação formatada com vírgula (ex.: "2,5").
    pub fn ocupacao_formatada(&self) -> String {
        format!("{:.1}", self.ocupacao).replace('.', ",")
    }
}
