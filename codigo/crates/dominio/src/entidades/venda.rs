use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use super::{Codificavel, Evento};
use crate::precos::{PoliticaPreco, PrecoInteira, PrecoMeiaEntrada};
use crate::valores::Dinheiro;
use crate::{Erro, Resultado};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TipoIngresso {
    Inteira,
    Meia,
}

impl TipoIngresso {
    /// Padrão **Strategy**: cada tipo de ingresso tem sua política de preço.
    pub fn politica(&self) -> Box<dyn PoliticaPreco> {
        match self {
            TipoIngresso::Inteira => Box::new(PrecoInteira),
            TipoIngresso::Meia => Box::new(PrecoMeiaEntrada),
        }
    }

    pub fn nome(&self) -> &'static str {
        self.politica().descricao()
    }
}

impl Codificavel for TipoIngresso {
    const TODOS: &'static [Self] = &[TipoIngresso::Inteira, TipoIngresso::Meia];

    fn codigo(&self) -> &'static str {
        match self {
            TipoIngresso::Inteira => "INTEIRA",
            TipoIngresso::Meia => "MEIA",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FormaPagamento {
    Pix,
    Credito,
    Debito,
}

impl FormaPagamento {
    pub fn nome(&self) -> &'static str {
        match self {
            FormaPagamento::Pix => "PIX",
            FormaPagamento::Credito => "Cartão de crédito",
            FormaPagamento::Debito => "Cartão de débito",
        }
    }
}

impl Codificavel for FormaPagamento {
    const TODOS: &'static [Self] = &[FormaPagamento::Pix, FormaPagamento::Credito, FormaPagamento::Debito];

    fn codigo(&self) -> &'static str {
        match self {
            FormaPagamento::Pix => "PIX",
            FormaPagamento::Credito => "CREDITO",
            FormaPagamento::Debito => "DEBITO",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StatusVenda {
    Confirmada,
    Cancelada,
    /// Ingresso já usado na entrada (check-in).
    Utilizada,
}

impl StatusVenda {
    pub fn nome(&self) -> &'static str {
        match self {
            StatusVenda::Confirmada => "Confirmada",
            StatusVenda::Cancelada => "Cancelada",
            StatusVenda::Utilizada => "Utilizada",
        }
    }
}

impl Codificavel for StatusVenda {
    const TODOS: &'static [Self] = &[StatusVenda::Confirmada, StatusVenda::Cancelada, StatusVenda::Utilizada];

    fn codigo(&self) -> &'static str {
        match self {
            StatusVenda::Confirmada => "CONFIRMADA",
            StatusVenda::Cancelada => "CANCELADA",
            StatusVenda::Utilizada => "UTILIZADA",
        }
    }
}

/// Venda a ser registrada (montada pelo caso de uso depois de todas as verificações).
#[derive(Debug, Clone)]
pub struct NovaVenda {
    pub data_compra: NaiveDateTime,
    pub valor: Dinheiro,
    pub tipo: TipoIngresso,
    pub forma_pagamento: FormaPagamento,
    pub cliente_id: i64,
    pub evento_id: i64,
    pub ingresso_id: i64,
    /// `None` para categorias sem assento marcado (ex.: Pista).
    pub assento_id: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Venda {
    id: i64,
    data_compra: NaiveDateTime,
    valor: Dinheiro,
    tipo: TipoIngresso,
    forma_pagamento: FormaPagamento,
    status: StatusVenda,
    cliente_id: i64,
    evento_id: i64,
    ingresso_id: i64,
    assento_id: Option<i64>,
}

impl Venda {
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruir(
        id: i64,
        data_compra: NaiveDateTime,
        valor: Dinheiro,
        tipo: TipoIngresso,
        forma_pagamento: FormaPagamento,
        status: StatusVenda,
        cliente_id: i64,
        evento_id: i64,
        ingresso_id: i64,
        assento_id: Option<i64>,
    ) -> Self {
        Venda { id, data_compra, valor, tipo, forma_pagamento, status, cliente_id, evento_id, ingresso_id, assento_id }
    }

    /// Venda recém-gravada a partir da `NovaVenda`.
    pub fn registrada(id: i64, nova: &NovaVenda) -> Self {
        Venda {
            id,
            data_compra: nova.data_compra,
            valor: nova.valor,
            tipo: nova.tipo,
            forma_pagamento: nova.forma_pagamento,
            status: StatusVenda::Confirmada,
            cliente_id: nova.cliente_id,
            evento_id: nova.evento_id,
            ingresso_id: nova.ingresso_id,
            assento_id: nova.assento_id,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn data_compra(&self) -> NaiveDateTime {
        self.data_compra
    }
    pub fn valor(&self) -> Dinheiro {
        self.valor
    }
    pub fn tipo(&self) -> TipoIngresso {
        self.tipo
    }
    pub fn forma_pagamento(&self) -> FormaPagamento {
        self.forma_pagamento
    }
    pub fn status(&self) -> StatusVenda {
        self.status
    }
    pub fn cliente_id(&self) -> i64 {
        self.cliente_id
    }
    pub fn evento_id(&self) -> i64 {
        self.evento_id
    }
    pub fn ingresso_id(&self) -> i64 {
        self.ingresso_id
    }
    pub fn assento_id(&self) -> Option<i64> {
        self.assento_id
    }

    /// Algoritmo 5 — condições para o cliente cancelar a compra.
    pub fn verificar_cancelamento(&self, cliente_id: i64, evento: &Evento, hoje: NaiveDate) -> Resultado<()> {
        if self.cliente_id != cliente_id {
            return Err(Erro::nao_encontrado("compra não encontrada"));
        }
        if self.status != StatusVenda::Confirmada {
            return Err(Erro::regra("esta compra já foi cancelada ou utilizada"));
        }
        if !evento.permite_cancelamento() {
            return Err(Erro::regra("este evento não permite cancelamento"));
        }
        if evento.ja_ocorreu(hoje) {
            return Err(Erro::regra("não é possível cancelar após a data do evento"));
        }
        Ok(())
    }

    /// Algoritmo 6 — check-in: só ingresso confirmado pode entrar.
    pub fn verificar_check_in(&self) -> Resultado<()> {
        match self.status {
            StatusVenda::Confirmada => Ok(()),
            StatusVenda::Cancelada => Err(Erro::regra("acesso negado: ingresso cancelado")),
            StatusVenda::Utilizada => Err(Erro::regra("acesso negado: ingresso já utilizado")),
        }
    }

    pub fn marcar_cancelada(&mut self) {
        self.status = StatusVenda::Cancelada;
    }

    pub fn marcar_utilizada(&mut self) {
        self.status = StatusVenda::Utilizada;
    }
}

/// Linha do histórico de compras do cliente (consulta com dados do evento).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ItemHistorico {
    pub venda: Venda,
    pub evento: String,
    pub data_evento: NaiveDate,
    pub horario: NaiveTime,
    pub local: String,
    pub categoria: String,
    pub assento: Option<String>,
    pub permite_cancelamento: bool,
}

/// Venda de um evento com o nome do cliente (lista de vendas do organizador).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VendaDetalhada {
    pub venda: Venda,
    pub cliente: String,
    pub categoria: String,
    pub assento: Option<String>,
}
