use crate::valores::{Dinheiro, texto_obrigatorio};
use crate::{Erro, Resultado};

/// Limite de fileiras (A a Z).
pub const MAX_FILEIRAS: i32 = 26;
/// Limite de assentos por fileira (para o mapa caber na tela).
pub const MAX_ASSENTOS_POR_FILEIRA: i32 = 30;

/// Dimensões da matriz de assentos de uma categoria com assento marcado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Setor {
    pub fileiras: i32,
    pub assentos_por_fileira: i32,
}

/// Categoria informada pelo organizador ao cadastrar o evento (Algoritmo 3).
#[derive(Debug, Clone)]
pub struct NovaCategoria {
    nome: String,
    preco: Dinheiro,
    quantidade: i32,
    setor: Option<Setor>,
}

impl NovaCategoria {
    /// Categoria sem assento marcado (ex.: Pista).
    pub fn sem_assento(nome: &str, preco: Dinheiro, quantidade: i32) -> Resultado<Self> {
        let nome = texto_obrigatorio(nome, "nome da categoria")?;
        if quantidade <= 0 {
            return Err(Erro::validacao("a quantidade deve ser maior que zero"));
        }
        Ok(NovaCategoria { nome, preco, quantidade, setor: None })
    }

    /// Categoria com assento marcado: a quantidade é fileiras × assentos por fileira.
    pub fn com_assento(nome: &str, preco: Dinheiro, fileiras: i32, assentos_por_fileira: i32) -> Resultado<Self> {
        let nome = texto_obrigatorio(nome, "nome da categoria")?;
        if !(1..=MAX_FILEIRAS).contains(&fileiras) {
            return Err(Erro::validacao(format!("o número de fileiras deve ser de 1 a {MAX_FILEIRAS}")));
        }
        if !(1..=MAX_ASSENTOS_POR_FILEIRA).contains(&assentos_por_fileira) {
            return Err(Erro::validacao(format!(
                "o número de assentos por fileira deve ser de 1 a {MAX_ASSENTOS_POR_FILEIRA}"
            )));
        }
        Ok(NovaCategoria {
            nome,
            preco,
            quantidade: fileiras * assentos_por_fileira,
            setor: Some(Setor { fileiras, assentos_por_fileira }),
        })
    }

    pub fn nome(&self) -> &str {
        &self.nome
    }
    pub fn preco(&self) -> Dinheiro {
        self.preco
    }
    pub fn quantidade(&self) -> i32 {
        self.quantidade
    }
    pub fn setor(&self) -> Option<Setor> {
        self.setor
    }
}

/// Ingressos que ainda podem ser vendidos no evento (soma do disponível de cada categoria).
pub fn total_disponivel(categorias: &[Ingresso]) -> i32 {
    categorias.iter().map(Ingresso::disponivel).sum()
}

/// Categoria de ingresso já cadastrada em um evento (tabela `ingressos`).
#[derive(Debug, Clone)]
pub struct Ingresso {
    id: i64,
    evento_id: i64,
    categoria: String,
    preco: Dinheiro,
    quantidade: i32,
    disponivel: i32,
    setor: Option<Setor>,
}

impl Ingresso {
    pub fn reconstruir(
        id: i64,
        evento_id: i64,
        categoria: String,
        preco: Dinheiro,
        quantidade: i32,
        disponivel: i32,
        setor: Option<Setor>,
    ) -> Self {
        Ingresso { id, evento_id, categoria, preco, quantidade, disponivel, setor }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn evento_id(&self) -> i64 {
        self.evento_id
    }
    pub fn categoria(&self) -> &str {
        &self.categoria
    }
    pub fn preco(&self) -> Dinheiro {
        self.preco
    }
    pub fn quantidade(&self) -> i32 {
        self.quantidade
    }
    pub fn disponivel(&self) -> i32 {
        self.disponivel
    }
    pub fn setor(&self) -> Option<Setor> {
        self.setor
    }
    pub fn com_assento(&self) -> bool {
        self.setor.is_some()
    }
    pub fn vendidos(&self) -> i32 {
        self.quantidade - self.disponivel
    }
    pub fn esgotado(&self) -> bool {
        self.disponivel <= 0
    }

    // ---- comportamentos (gestão do evento pelo organizador) ----

    /// Altera nome e preço. O novo preço vale para as próximas vendas.
    pub fn alterar(&mut self, nome: &str, preco: Dinheiro) -> Resultado<()> {
        self.categoria = texto_obrigatorio(nome, "nome da categoria")?;
        self.preco = preco;
        Ok(())
    }

    /// Nova quantidade total (só categorias sem assento marcado). Não pode ficar abaixo do
    /// que já foi vendido; a quantidade disponível é recalculada.
    pub fn alterar_quantidade(&mut self, nova: i32) -> Resultado<()> {
        if self.com_assento() {
            return Err(Erro::regra(
                "a quantidade de uma categoria com assento marcado é definida pelo mapa (fileiras × assentos)",
            ));
        }
        if nova <= 0 {
            return Err(Erro::validacao("a quantidade deve ser maior que zero"));
        }
        if nova < self.vendidos() {
            return Err(Erro::regra(format!(
                "já foram vendidos {} ingressos desta categoria; a quantidade não pode ser menor",
                self.vendidos()
            )));
        }
        self.disponivel = nova - self.vendidos();
        self.quantidade = nova;
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn pista(quantidade: i32, disponivel: i32) -> Ingresso {
        Ingresso::reconstruir(1, 1, "Pista".into(), Dinheiro::ZERO, quantidade, disponivel, None)
    }

    #[test]
    fn quantidade_nao_fica_abaixo_do_vendido() {
        let mut i = pista(100, 70); // 30 vendidos
        assert!(matches!(i.alterar_quantidade(20), Err(Erro::RegraNegocio(_))));
        i.alterar_quantidade(50).unwrap();
        assert_eq!((i.quantidade(), i.disponivel(), i.vendidos()), (50, 20, 30));
        assert!(i.alterar_quantidade(0).is_err());
    }

    #[test]
    fn categoria_com_assento_nao_muda_quantidade() {
        let setor = Some(Setor { fileiras: 2, assentos_por_fileira: 5 });
        let mut i = Ingresso::reconstruir(1, 1, "Cadeira".into(), Dinheiro::ZERO, 10, 10, setor);
        assert!(i.alterar_quantidade(12).is_err());
        i.alterar("Cadeira VIP", Dinheiro::de_centavos(9900).unwrap()).unwrap();
        assert_eq!(i.categoria(), "Cadeira VIP");
        assert!(i.alterar("  ", Dinheiro::ZERO).is_err());
    }
}
