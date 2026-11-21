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
}
