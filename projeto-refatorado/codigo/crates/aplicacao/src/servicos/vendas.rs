use std::sync::Arc;

use dominio::entidades::*;
use dominio::portas::{Relogio, RepositorioEventos, RepositorioVendas};
use dominio::valores::Dinheiro;
use dominio::{Erro, Resultado};

use crate::sessao::Sessao;

/// Escolhas do cliente durante a compra.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pedido {
    pub evento_id: i64,
    pub ingresso_id: i64,
    pub tipo: TipoIngresso,
    /// (fileira, número) — obrigatório só para categorias com assento marcado.
    pub assento: Option<(String, i32)>,
    pub forma_pagamento: FormaPagamento,
}

/// Resumo mostrado antes da confirmação.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResumoCompra {
    pub evento: Evento,
    pub ingresso: Ingresso,
    pub assento: Option<Assento>,
    pub tipo: TipoIngresso,
    pub forma_pagamento: FormaPagamento,
    pub valor: Dinheiro,
}

/// Algoritmos 4 (comprar), 5 (cancelar) e 5b (remarcar), mais o histórico do cliente.
pub struct ServicoVendas {
    eventos: Arc<dyn RepositorioEventos>,
    vendas: Arc<dyn RepositorioVendas>,
    relogio: Arc<dyn Relogio>,
}

impl ServicoVendas {
    pub fn new(
        eventos: Arc<dyn RepositorioEventos>,
        vendas: Arc<dyn RepositorioVendas>,
        relogio: Arc<dyn Relogio>,
    ) -> Self {
        ServicoVendas { eventos, vendas, relogio }
    }

    /// Passo 1 do fluxograma 4: evento existe, está ABERTO, tem data futura e vagas?
    pub async fn evento_para_venda(&self, evento_id: i64) -> Resultado<Evento> {
        let evento =
            self.eventos.buscar(evento_id).await?.ok_or_else(|| Erro::nao_encontrado("evento não encontrado"))?;
        evento.verificar_disponivel_para_venda(self.relogio.hoje())?;
        Ok(evento)
    }

    /// Passo 2: a categoria é deste evento e ainda tem ingressos?
    pub async fn ingresso_para_venda(&self, evento: &Evento, ingresso_id: i64) -> Resultado<Ingresso> {
        let ingresso = match self.eventos.buscar_categoria(ingresso_id).await? {
            Some(i) if i.evento_id() == evento.id() => i,
            _ => return Err(Erro::nao_encontrado("categoria inexistente neste evento")),
        };
        if ingresso.esgotado() {
            return Err(Erro::regra("categoria esgotada"));
        }
        Ok(ingresso)
    }

    /// Passo 3: o assento existe e está LIVRE? (evita venda duplicada)
    pub async fn assento_para_venda(&self, ingresso: &Ingresso, fileira: &str, numero: i32) -> Resultado<Assento> {
        if indice_fileira(fileira).is_none() {
            return Err(Erro::validacao("fileira deve ser uma letra (A, B, C...)"));
        }
        let fileira = fileira.trim().to_uppercase();
        let assento = self
            .eventos
            .buscar_assento(ingresso.id(), &fileira, numero)
            .await?
            .ok_or_else(|| Erro::nao_encontrado(format!("assento {fileira}{numero} inexistente")))?;
        if !assento.livre() {
            return Err(Erro::regra(format!("assento {} já vendido. Escolha outro", assento.rotulo())));
        }
        Ok(assento)
    }

    /// Todas as verificações do fluxograma 4 + cálculo do valor (Strategy), sem gravar nada.
    pub async fn preparar(&self, sessao: &Sessao, pedido: &Pedido) -> Resultado<ResumoCompra> {
        sessao.exigir(Perfil::Cliente)?;
        let evento = self.evento_para_venda(pedido.evento_id).await?;
        let ingresso = self.ingresso_para_venda(&evento, pedido.ingresso_id).await?;
        let assento = match (&pedido.assento, ingresso.com_assento()) {
            (Some((fileira, numero)), true) => Some(self.assento_para_venda(&ingresso, fileira, *numero).await?),
            (None, true) => return Err(Erro::validacao("escolha um assento no mapa")),
            (_, false) => None,
        };
        let valor = pedido.tipo.politica().calcular(ingresso.preco());
        Ok(ResumoCompra {
            evento,
            ingresso,
            assento,
            tipo: pedido.tipo,
            forma_pagamento: pedido.forma_pagamento,
            valor,
        })
    }

    /// Algoritmo 4 — confirma: verifica tudo de novo e grava de forma atômica.
    pub async fn confirmar(&self, sessao: &Sessao, pedido: &Pedido) -> Resultado<(Venda, ResumoCompra)> {
        let resumo = self.preparar(sessao, pedido).await?;
        let nova = NovaVenda {
            data_compra: self.relogio.agora(),
            valor: resumo.valor,
            tipo: resumo.tipo,
            forma_pagamento: resumo.forma_pagamento,
            cliente_id: sessao.usuario_id,
            evento_id: resumo.evento.id(),
            ingresso_id: resumo.ingresso.id(),
            assento_id: resumo.assento.as_ref().map(Assento::id),
        };
        let venda = self.vendas.registrar(&nova).await?;
        Ok((venda, resumo))
    }

    /// Algoritmo 5 — cancelar compra.
    pub async fn cancelar(&self, sessao: &Sessao, venda_id: i64) -> Resultado<Venda> {
        sessao.exigir(Perfil::Cliente)?;
        let mut venda =
            self.vendas.buscar(venda_id).await?.ok_or_else(|| Erro::nao_encontrado("compra não encontrada"))?;
        let evento =
            self.eventos.buscar(venda.evento_id()).await?.ok_or_else(|| Erro::nao_encontrado("evento da compra"))?;
        venda.verificar_cancelamento(sessao.usuario_id, &evento, self.relogio.hoje())?;
        self.vendas.cancelar(&venda).await?;
        venda.marcar_cancelada();
        Ok(venda)
    }

    /// Algoritmo 5b — remarcação: troca o assento por outro livre da mesma categoria.
    pub async fn remarcar(&self, sessao: &Sessao, venda_id: i64, fileira: &str, numero: i32) -> Resultado<Venda> {
        sessao.exigir(Perfil::Cliente)?;
        let mut venda =
            self.vendas.buscar(venda_id).await?.ok_or_else(|| Erro::nao_encontrado("compra não encontrada"))?;
        let evento =
            self.eventos.buscar(venda.evento_id()).await?.ok_or_else(|| Erro::nao_encontrado("evento da compra"))?;
        venda.verificar_remarcacao(sessao.usuario_id, &evento, self.relogio.hoje())?;
        let ingresso = self
            .eventos
            .buscar_categoria(venda.ingresso_id())
            .await?
            .ok_or_else(|| Erro::nao_encontrado("categoria da compra"))?;
        let novo = self.assento_para_venda(&ingresso, fileira, numero).await?;
        self.vendas.remarcar(&venda, novo.id()).await?;
        venda.trocar_assento(novo.id());
        Ok(venda)
    }

    /// Histórico de compras do cliente (eventos passados e futuros).
    pub async fn historico(&self, sessao: &Sessao) -> Resultado<Vec<ItemHistorico>> {
        sessao.exigir(Perfil::Cliente)?;
        self.vendas.historico_do_cliente(sessao.usuario_id).await
    }

    /// Dados de uma compra do cliente para o comprovante.
    pub async fn comprovante(&self, sessao: &Sessao, venda_id: i64) -> Resultado<ItemHistorico> {
        self.historico(sessao)
            .await?
            .into_iter()
            .find(|item| item.venda.id() == venda_id)
            .ok_or_else(|| Erro::nao_encontrado("compra não encontrada"))
    }
}
