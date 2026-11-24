use super::{Codificavel, Ingresso};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Situacao {
    Livre,
    Ocupado,
}

impl Codificavel for Situacao {
    const TODOS: &'static [Self] = &[Situacao::Livre, Situacao::Ocupado];

    fn codigo(&self) -> &'static str {
        match self {
            Situacao::Livre => "LIVRE",
            Situacao::Ocupado => "OCUPADO",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Assento {
    id: i64,
    ingresso_id: i64,
    setor: String,
    fileira: String,
    numero: i32,
    situacao: Situacao,
}

impl Assento {
    pub fn reconstruir(
        id: i64,
        ingresso_id: i64,
        setor: String,
        fileira: String,
        numero: i32,
        situacao: Situacao,
    ) -> Self {
        Assento { id, ingresso_id, setor, fileira, numero, situacao }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn ingresso_id(&self) -> i64 {
        self.ingresso_id
    }
    pub fn setor(&self) -> &str {
        &self.setor
    }
    pub fn fileira(&self) -> &str {
        &self.fileira
    }
    pub fn numero(&self) -> i32 {
        self.numero
    }
    pub fn situacao(&self) -> Situacao {
        self.situacao
    }
    pub fn livre(&self) -> bool {
        self.situacao == Situacao::Livre
    }
    /// Identificação legível, ex.: "B7".
    pub fn rotulo(&self) -> String {
        format!("{}{}", self.fileira, self.numero)
    }
}

/// Converte o índice da fileira (0, 1, 2...) na letra ("A", "B", "C"...).
pub fn letra_fileira(indice: usize) -> String {
    match u8::try_from(indice) {
        Ok(i) if i < 26 => char::from(b'A' + i).to_string(),
        _ => "?".to_string(),
    }
}

/// Converte a letra da fileira no índice (0 = "A"). Aceita minúscula.
pub fn indice_fileira(letra: &str) -> Option<usize> {
    let mut chars = letra.trim().chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Some(usize::from(c.to_ascii_uppercase() as u8 - b'A')),
        _ => None,
    }
}

/// Um assento no mapa (para desenhar a tela).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CelulaMapa {
    pub numero: i32,
    pub rotulo: String,
    pub ocupado: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinhaMapa {
    pub letra: String,
    pub assentos: Vec<CelulaMapa>,
}

/// Mapa de assentos de um setor, guardado como **matriz** `matriz[fileira][número]`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MapaAssentos {
    setor: String,
    matriz: Vec<Vec<Situacao>>,
}

impl MapaAssentos {
    /// Monta a matriz a partir da categoria e da lista de assentos gravada.
    pub fn montar(ingresso: &Ingresso, assentos: &[Assento]) -> Self {
        let (fileiras, por_fileira) =
            ingresso.setor().map_or((0, 0), |s| (tamanho(s.fileiras), tamanho(s.assentos_por_fileira)));
        let mut matriz = vec![vec![Situacao::Livre; por_fileira]; fileiras];
        for a in assentos {
            if let Some(f) = indice_fileira(a.fileira()).filter(|f| *f < fileiras)
                && let Some(n) = usize::try_from(a.numero() - 1).ok().filter(|n| *n < por_fileira)
            {
                matriz[f][n] = a.situacao();
            }
        }
        MapaAssentos { setor: ingresso.categoria().to_string(), matriz }
    }

    pub fn setor(&self) -> &str {
        &self.setor
    }

    pub fn situacao(&self, fileira: usize, numero: usize) -> Option<Situacao> {
        self.matriz.get(fileira)?.get(numero).copied()
    }

    pub fn total(&self) -> usize {
        self.matriz.iter().map(Vec::len).sum()
    }

    pub fn livres(&self) -> usize {
        self.matriz.iter().flatten().filter(|s| **s == Situacao::Livre).count()
    }

    /// Números das colunas (1..=assentos por fileira), para o cabeçalho do mapa.
    pub fn numeros(&self) -> Vec<i32> {
        (1..=self.matriz.first().map_or(0, Vec::len)).map(numero_i32).collect()
    }

    /// Percorre a matriz com dois laços aninhados (fileiras × números) — Algoritmo "exibir mapa".
    pub fn linhas(&self) -> Vec<LinhaMapa> {
        let mut linhas = Vec::new();
        for (f, fileira) in self.matriz.iter().enumerate() {
            let letra = letra_fileira(f);
            let mut assentos = Vec::new();
            for (n, situacao) in fileira.iter().enumerate() {
                assentos.push(CelulaMapa {
                    numero: numero_i32(n + 1),
                    rotulo: format!("{letra}{}", n + 1),
                    ocupado: *situacao == Situacao::Ocupado,
                });
            }
            linhas.push(LinhaMapa { letra, assentos });
        }
        linhas
    }
}

/// Quantidade (i32 do banco) como tamanho de vetor; negativos viram 0.
fn tamanho(n: i32) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// Número do assento para exibição (o mapa tem no máximo 26 × 30 posições).
fn numero_i32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::entidades::Setor;
    use crate::valores::Dinheiro;

    #[test]
    fn letras_e_indices() {
        assert_eq!(letra_fileira(0), "A");
        assert_eq!(indice_fileira("c"), Some(2));
        assert_eq!(indice_fileira("AB"), None);
        assert_eq!(indice_fileira("1"), None);
    }

    #[test]
    fn monta_matriz() {
        let setor = Setor { fileiras: 2, assentos_por_fileira: 3 };
        let ingresso = Ingresso::reconstruir(1, 1, "Cadeira".into(), Dinheiro::ZERO, 6, 5, Some(setor));
        let assentos = [Assento::reconstruir(9, 1, "Cadeira".into(), "B".into(), 2, Situacao::Ocupado)];
        let mapa = MapaAssentos::montar(&ingresso, &assentos);
        assert_eq!(mapa.situacao(1, 1), Some(Situacao::Ocupado));
        assert_eq!(mapa.livres(), 5);
        assert_eq!(mapa.linhas()[1].assentos[1].rotulo, "B2");
    }
}
