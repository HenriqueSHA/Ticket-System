use std::sync::Arc;

use dominio::entidades::{Perfil, RelatorioEvento, Venda};
use dominio::portas::{RepositorioEventos, RepositorioOrganizadores, RepositorioVendas};
use dominio::{Erro, Resultado};

use crate::sessao::Sessao;

/// Algoritmo 6 — relatório do evento e registro de presença (check-in).
pub struct ServicoRelatorios {
    eventos: Arc<dyn RepositorioEventos>,
    vendas: Arc<dyn RepositorioVendas>,
    organizadores: Arc<dyn RepositorioOrganizadores>,
}

impl ServicoRelatorios {
    pub fn new(
        eventos: Arc<dyn RepositorioEventos>,
        vendas: Arc<dyn RepositorioVendas>,
        organizadores: Arc<dyn RepositorioOrganizadores>,
    ) -> Self {
        ServicoRelatorios { eventos, vendas, organizadores }
    }

    async fn organizador_ativo_id(&self, sessao: &Sessao) -> Resultado<i64> {
        sessao.exigir(Perfil::Organizador)?;
        match self.organizadores.buscar_por_usuario(sessao.usuario_id).await? {
            Some(org) if org.esta_ativo() => Ok(org.id()),
            _ => Err(Erro::acesso_negado("apenas organizadores ativos")),
        }
    }

    pub async fn relatorio(&self, sessao: &Sessao, evento_id: i64) -> Resultado<RelatorioEvento> {
        let org_id = self.organizador_ativo_id(sessao).await?;
        let evento = match self.eventos.buscar(evento_id).await? {
            Some(e) if e.pertence_a(org_id) => e,
            _ => return Err(Erro::nao_encontrado("evento não encontrado entre os seus eventos")),
        };
        let vendas = self.vendas.listar_do_evento(evento.id()).await?;
        let categorias = self.eventos.listar_categorias(evento.id()).await?;
        Ok(RelatorioEvento::calcular(evento, &vendas, categorias))
    }

    /// Check-in: marca o ingresso como utilizado na entrada do evento.
    pub async fn registrar_presenca(&self, sessao: &Sessao, venda_id: i64) -> Resultado<Venda> {
        let org_id = self.organizador_ativo_id(sessao).await?;
        let nao_encontrado = || Erro::nao_encontrado("ingresso não encontrado para os seus eventos");
        let mut venda = self.vendas.buscar(venda_id).await?.ok_or_else(nao_encontrado)?;
        let evento = self.eventos.buscar(venda.evento_id()).await?.ok_or_else(nao_encontrado)?;
        if !evento.pertence_a(org_id) {
            return Err(nao_encontrado());
        }
        venda.verificar_check_in()?;
        if !self.vendas.marcar_utilizada(venda.id()).await? {
            return Err(Erro::regra("acesso negado: ingresso já utilizado"));
        }
        venda.marcar_utilizada();
        Ok(venda)
    }
}
