use std::sync::Arc;

use dominio::entidades::*;
use dominio::portas::{Relogio, RepositorioEventos, RepositorioOrganizadores};
use dominio::{Erro, Resultado};

use crate::sessao::Sessao;

/// Algoritmo 3 (cadastro de evento) e consultas de eventos, categorias e assentos.
pub struct ServicoEventos {
    eventos: Arc<dyn RepositorioEventos>,
    organizadores: Arc<dyn RepositorioOrganizadores>,
    relogio: Arc<dyn Relogio>,
}

impl ServicoEventos {
    pub fn new(
        eventos: Arc<dyn RepositorioEventos>,
        organizadores: Arc<dyn RepositorioOrganizadores>,
        relogio: Arc<dyn Relogio>,
    ) -> Self {
        ServicoEventos { eventos, organizadores, relogio }
    }

    /// Organizador da sessão, relido do banco (pode ter sido bloqueado depois do login).
    pub async fn organizador_ativo(&self, sessao: &Sessao) -> Resultado<Organizador> {
        sessao.exigir(Perfil::Organizador)?;
        let org = self
            .organizadores
            .buscar_por_usuario(sessao.usuario_id)
            .await?
            .ok_or_else(|| Erro::nao_encontrado("organizador da sessão"))?;
        if !org.esta_ativo() {
            return Err(Erro::acesso_negado("apenas organizadores ativos podem gerenciar eventos"));
        }
        Ok(org)
    }

    /// Início do Algoritmo 3: valida os dados do evento e devolve o *builder* para as categorias.
    pub async fn iniciar_cadastro(&self, sessao: &Sessao, dados: DadosEvento) -> Resultado<EventoBuilder> {
        self.organizador_ativo(sessao).await?;
        EventoBuilder::novo(dados, self.relogio.hoje())
    }

    /// Fim do Algoritmo 3: grava evento, categorias e assentos.
    pub async fn cadastrar(&self, sessao: &Sessao, novo: &NovoEvento) -> Resultado<Evento> {
        let org = self.organizador_ativo(sessao).await?;
        novo.dados().validar(self.relogio.hoje())?;
        self.eventos.inserir(org.id(), novo).await
    }

    /// Evento do organizador logado (cada organizador só acessa os seus — regra 7.3).
    pub async fn evento_do_organizador(&self, sessao: &Sessao, evento_id: i64) -> Resultado<Evento> {
        let org = self.organizador_ativo(sessao).await?;
        match self.eventos.buscar(evento_id).await? {
            Some(e) if e.pertence_a(org.id()) => Ok(e),
            _ => Err(Erro::nao_encontrado("evento não encontrado entre os seus eventos")),
        }
    }

    pub async fn alterar(
        &self,
        sessao: &Sessao,
        evento_id: i64,
        dados: DadosEvento,
        status: StatusEvento,
    ) -> Resultado<Evento> {
        let mut evento = self.evento_do_organizador(sessao, evento_id).await?;
        evento.alterar(dados, status, self.relogio.hoje())?;
        self.eventos.atualizar(&evento).await?;
        Ok(evento)
    }

    pub async fn listar_do_organizador(&self, sessao: &Sessao) -> Resultado<Vec<Evento>> {
        let org = self.organizador_ativo(sessao).await?;
        self.eventos.listar_do_organizador(org.id()).await
    }

    /// Eventos que o cliente pode comprar (abertos, futuros e com vagas).
    pub async fn listar_disponiveis(&self) -> Resultado<Vec<(Evento, Vec<Ingresso>)>> {
        let mut lista = Vec::new();
        for evento in self.eventos.listar_disponiveis(self.relogio.hoje()).await? {
            let categorias = self.eventos.listar_categorias(evento.id()).await?;
            lista.push((evento, categorias));
        }
        Ok(lista)
    }

    pub async fn categorias(&self, evento_id: i64) -> Resultado<Vec<Ingresso>> {
        self.eventos.listar_categorias(evento_id).await
    }

    pub async fn mapa(&self, ingresso: &Ingresso) -> Resultado<MapaAssentos> {
        let assentos = self.eventos.listar_assentos(ingresso.id()).await?;
        Ok(MapaAssentos::montar(ingresso, &assentos))
    }
}
