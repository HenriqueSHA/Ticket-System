use chrono::{NaiveDate, NaiveTime};

use super::{Codificavel, NovaCategoria};
use crate::valores::texto_obrigatorio;
use crate::{Erro, Resultado};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEvento {
    /// Vendas abertas.
    Aberto,
    /// Vendas encerradas pelo organizador.
    Encerrado,
}

impl StatusEvento {
    pub fn nome(&self) -> &'static str {
        match self {
            StatusEvento::Aberto => "Vendas abertas",
            StatusEvento::Encerrado => "Vendas encerradas",
        }
    }
}

impl Codificavel for StatusEvento {
    const TODOS: &'static [Self] = &[StatusEvento::Aberto, StatusEvento::Encerrado];

    fn codigo(&self) -> &'static str {
        match self {
            StatusEvento::Aberto => "ABERTO",
            StatusEvento::Encerrado => "ENCERRADO",
        }
    }
}

/// Dados informados pelo organizador (formulário de cadastro/alteração).
#[derive(Debug, Clone, PartialEq)]
pub struct DadosEvento {
    pub nome: String,
    pub descricao: String,
    pub local: String,
    pub data: NaiveDate,
    pub horario: NaiveTime,
    pub capacidade: i32,
    pub permite_cancelamento: bool,
}

impl DadosEvento {
    /// Algoritmo 3, primeira parte: nome e local obrigatórios, data futura, capacidade > 0.
    pub fn validar(&self, hoje: NaiveDate) -> Resultado<()> {
        texto_obrigatorio(&self.nome, "nome")?;
        texto_obrigatorio(&self.local, "local")?;
        if self.data <= hoje {
            return Err(Erro::validacao("a data do evento deve ser futura"));
        }
        if self.capacidade <= 0 {
            return Err(Erro::validacao("a capacidade deve ser maior que zero"));
        }
        Ok(())
    }

    fn normalizado(mut self) -> Self {
        self.nome = self.nome.trim().to_string();
        self.descricao = self.descricao.trim().to_string();
        self.local = self.local.trim().to_string();
        self
    }
}

/// Evento validado, pronto para ser gravado. Só pode ser criado pelo [`EventoBuilder`].
#[derive(Debug, Clone)]
pub struct NovoEvento {
    dados: DadosEvento,
    categorias: Vec<NovaCategoria>,
}

impl NovoEvento {
    pub fn dados(&self) -> &DadosEvento {
        &self.dados
    }
    pub fn categorias(&self) -> &[NovaCategoria] {
        &self.categorias
    }
}

/// Padrão **Builder**: monta o evento passo a passo, validando cada categoria
/// contra a capacidade (laço de categorias do Algoritmo 3).
pub struct EventoBuilder {
    dados: DadosEvento,
    categorias: Vec<NovaCategoria>,
    total: i32,
}

impl EventoBuilder {
    pub fn novo(dados: DadosEvento, hoje: NaiveDate) -> Resultado<Self> {
        dados.validar(hoje)?;
        Ok(EventoBuilder { dados: dados.normalizado(), categorias: Vec::new(), total: 0 })
    }

    /// Adiciona uma categoria; recusa se a soma ultrapassar a capacidade do evento.
    pub fn adicionar_categoria(&mut self, categoria: NovaCategoria) -> Resultado<&mut Self> {
        if self.total + categoria.quantidade() > self.dados.capacidade {
            return Err(Erro::regra(format!(
                "a categoria \"{}\" ultrapassa a capacidade do evento ({} vagas restantes)",
                categoria.nome(),
                self.vagas_restantes()
            )));
        }
        self.total += categoria.quantidade();
        self.categorias.push(categoria);
        Ok(self)
    }

    pub fn vagas_restantes(&self) -> i32 {
        self.dados.capacidade - self.total
    }

    pub fn construir(self) -> Resultado<NovoEvento> {
        if self.categorias.is_empty() {
            return Err(Erro::validacao("o evento precisa de pelo menos uma categoria de ingresso"));
        }
        Ok(NovoEvento { dados: self.dados, categorias: self.categorias })
    }
}

#[derive(Debug, Clone)]
pub struct Evento {
    id: i64,
    organizador_id: i64,
    dados: DadosEvento,
    total_vendido: i32,
    status: StatusEvento,
}

impl Evento {
    pub fn reconstruir(id: i64, organizador_id: i64, dados: DadosEvento, total_vendido: i32, status: StatusEvento) -> Self {
        Evento { id, organizador_id, dados, total_vendido, status }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn organizador_id(&self) -> i64 {
        self.organizador_id
    }
    pub fn dados(&self) -> &DadosEvento {
        &self.dados
    }
    pub fn nome(&self) -> &str {
        &self.dados.nome
    }
    pub fn descricao(&self) -> &str {
        &self.dados.descricao
    }
    pub fn local(&self) -> &str {
        &self.dados.local
    }
    pub fn data(&self) -> NaiveDate {
        self.dados.data
    }
    pub fn horario(&self) -> NaiveTime {
        self.dados.horario
    }
    pub fn capacidade(&self) -> i32 {
        self.dados.capacidade
    }
    pub fn permite_cancelamento(&self) -> bool {
        self.dados.permite_cancelamento
    }
    pub fn total_vendido(&self) -> i32 {
        self.total_vendido
    }
    pub fn status(&self) -> StatusEvento {
        self.status
    }

    pub fn vagas_restantes(&self) -> i32 {
        self.dados.capacidade - self.total_vendido
    }
    pub fn esta_lotado(&self) -> bool {
        self.total_vendido >= self.dados.capacidade
    }
    pub fn ja_ocorreu(&self, hoje: NaiveDate) -> bool {
        self.dados.data <= hoje
    }
    pub fn pertence_a(&self, organizador_id: i64) -> bool {
        self.organizador_id == organizador_id
    }

    /// Algoritmo 4, primeira verificação: evento ABERTO, com data futura e com vagas.
    pub fn verificar_disponivel_para_venda(&self, hoje: NaiveDate) -> Resultado<()> {
        if self.status != StatusEvento::Aberto || self.ja_ocorreu(hoje) {
            return Err(Erro::regra("evento indisponível para venda"));
        }
        if self.esta_lotado() {
            return Err(Erro::regra("evento lotado"));
        }
        Ok(())
    }

    /// Mudanças na estrutura do evento (categorias, exclusão) só antes de ele acontecer.
    pub fn verificar_gerenciavel(&self, hoje: NaiveDate) -> Resultado<()> {
        if self.ja_ocorreu(hoje) {
            return Err(Erro::regra("o evento já foi realizado e não pode mais ser alterado"));
        }
        Ok(())
    }

    /// A soma das quantidades das categorias não pode passar da capacidade.
    /// `soma_outras` = soma das demais categorias; `quantidade` = a nova/alterada.
    pub fn verificar_capacidade_para(&self, soma_outras: i32, quantidade: i32) -> Resultado<()> {
        if soma_outras + quantidade > self.dados.capacidade {
            return Err(Erro::regra(format!(
                "ultrapassa a capacidade do evento ({} vagas livres para categorias)",
                (self.dados.capacidade - soma_outras).max(0)
            )));
        }
        Ok(())
    }

    /// Altera os dados do evento. A nova capacidade não pode ser menor que a soma das
    /// quantidades das categorias já cadastradas (`soma_categorias`).
    pub fn alterar(&mut self, dados: DadosEvento, status: StatusEvento, hoje: NaiveDate, soma_categorias: i32) -> Resultado<()> {
        dados.validar(hoje)?;
        if dados.capacidade < soma_categorias {
            return Err(Erro::regra(format!(
                "a capacidade não pode ser menor que a soma das categorias ({soma_categorias})"
            )));
        }
        self.dados = dados.normalizado();
        self.status = status;
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::valores::Dinheiro;

    fn hoje() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 21).unwrap()
    }

    fn dados(capacidade: i32) -> DadosEvento {
        DadosEvento {
            nome: "Show".into(),
            descricao: String::new(),
            local: "Arena".into(),
            data: NaiveDate::from_ymd_opt(2026, 12, 20).unwrap(),
            horario: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            capacidade,
            permite_cancelamento: true,
        }
    }

    #[test]
    fn builder_respeita_capacidade() {
        let preco = Dinheiro::de_centavos(5000).unwrap();
        let mut b = EventoBuilder::novo(dados(100), hoje()).unwrap();
        b.adicionar_categoria(NovaCategoria::sem_assento("Pista", preco, 60).unwrap()).unwrap();
        assert_eq!(b.vagas_restantes(), 40);
        let excede = NovaCategoria::com_assento("Cadeira", preco, 5, 10).unwrap(); // 50 > 40
        assert!(matches!(b.adicionar_categoria(excede), Err(Erro::RegraNegocio(_))));
        b.adicionar_categoria(NovaCategoria::com_assento("Cadeira", preco, 4, 10).unwrap()).unwrap();
        assert_eq!(b.construir().unwrap().categorias().len(), 2);
    }

    #[test]
    fn capacidade_e_soma_das_categorias() {
        let mut e = Evento::reconstruir(1, 1, dados(100), 10, StatusEvento::Aberto);
        assert!(e.verificar_capacidade_para(60, 40).is_ok());
        assert!(matches!(e.verificar_capacidade_para(60, 41), Err(Erro::RegraNegocio(_))));
        assert!(e.alterar(dados(80), StatusEvento::Aberto, hoje(), 90).is_err()); // menor que as categorias
        e.alterar(dados(150), StatusEvento::Encerrado, hoje(), 90).unwrap();
        assert_eq!((e.capacidade(), e.status()), (150, StatusEvento::Encerrado));
        let realizado = Evento::reconstruir(2, 1, DadosEvento { data: hoje(), ..dados(10) }, 0, StatusEvento::Aberto);
        assert!(realizado.verificar_gerenciavel(hoje()).is_err());
    }

    #[test]
    fn builder_recusa_dados_invalidos() {
        let passado = DadosEvento { data: hoje(), ..dados(10) };
        assert!(EventoBuilder::novo(passado, hoje()).is_err());
        assert!(EventoBuilder::novo(dados(0), hoje()).is_err());
        assert!(EventoBuilder::novo(dados(10), hoje()).unwrap().construir().is_err()); // sem categorias
    }
}
