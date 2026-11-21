use std::sync::Arc;

use dominio::entidades::*;
use dominio::portas::{Relogio, RepositorioEventos, RepositorioOrganizadores};
use dominio::valores::Dinheiro;
use dominio::{Erro, Resultado};

use crate::sessao::Sessao;

/// Soma das quantidades das categorias (não pode passar da capacidade do evento).
fn soma_quantidades(categorias: &[Ingresso]) -> i32 {
    categorias.iter().map(Ingresso::quantidade).sum()
}

/// Algoritmo 3 (cadastro de evento), gestão do evento pelo organizador e consultas
/// de eventos, categorias e assentos.
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

    /// Altera dados, capacidade e status. A capacidade não pode ficar abaixo da soma das categorias.
    pub async fn alterar(
        &self,
        sessao: &Sessao,
        evento_id: i64,
        dados: DadosEvento,
        status: StatusEvento,
    ) -> Resultado<Evento> {
        let mut evento = self.evento_do_organizador(sessao, evento_id).await?;
        let soma = soma_quantidades(&self.eventos.listar_categorias(evento_id).await?);
        evento.alterar(dados, status, self.relogio.hoje(), soma)?;
        self.eventos.atualizar(&evento).await?;
        Ok(evento)
    }

    /// Evento do organizador com suas categorias (tela "Gerenciar evento").
    pub async fn gerenciar(&self, sessao: &Sessao, evento_id: i64) -> Resultado<(Evento, Vec<Ingresso>)> {
        let evento = self.evento_do_organizador(sessao, evento_id).await?;
        let categorias = self.eventos.listar_categorias(evento_id).await?;
        Ok((evento, categorias))
    }

    /// Nova categoria em um evento já cadastrado (mesmas regras do laço do Algoritmo 3).
    pub async fn adicionar_categoria(&self, sessao: &Sessao, evento_id: i64, categoria: NovaCategoria) -> Resultado<Ingresso> {
        let evento = self.evento_do_organizador(sessao, evento_id).await?;
        evento.verificar_gerenciavel(self.relogio.hoje())?;
        let soma = soma_quantidades(&self.eventos.listar_categorias(evento_id).await?);
        evento.verificar_capacidade_para(soma, categoria.quantidade())?;
        self.eventos.adicionar_categoria(evento_id, &categoria).await
    }

    /// Categoria do evento do organizador (para o formulário de edição).
    pub async fn categoria_do_organizador(&self, sessao: &Sessao, evento_id: i64, ingresso_id: i64) -> Resultado<(Evento, Ingresso)> {
        let (evento, categorias) = self.gerenciar(sessao, evento_id).await?;
        let ingresso = categorias
            .into_iter()
            .find(|c| c.id() == ingresso_id)
            .ok_or_else(|| Erro::nao_encontrado("categoria não encontrada neste evento"))?;
        Ok((evento, ingresso))
    }

    /// Altera nome, preço e — nas categorias sem assento — a quantidade de ingressos.
    pub async fn alterar_categoria(
        &self,
        sessao: &Sessao,
        evento_id: i64,
        ingresso_id: i64,
        nome: &str,
        preco: Dinheiro,
        quantidade: Option<i32>,
    ) -> Resultado<Ingresso> {
        let (evento, categorias) = self.gerenciar(sessao, evento_id).await?;
        evento.verificar_gerenciavel(self.relogio.hoje())?;
        let mut ingresso = categorias
            .iter()
            .find(|c| c.id() == ingresso_id)
            .cloned()
            .ok_or_else(|| Erro::nao_encontrado("categoria não encontrada neste evento"))?;
        ingresso.alterar(nome, preco)?;
        if let Some(nova) = quantidade.filter(|q| *q != ingresso.quantidade()) {
            ingresso.alterar_quantidade(nova)?;
            let soma_outras = soma_quantidades(&categorias) - categorias.iter().find(|c| c.id() == ingresso_id).map_or(0, Ingresso::quantidade);
            evento.verificar_capacidade_para(soma_outras, nova)?;
        }
        self.eventos.atualizar_categoria(&ingresso).await?;
        Ok(ingresso)
    }

    /// Remove uma categoria sem vendas. O evento precisa continuar com pelo menos uma.
    pub async fn remover_categoria(&self, sessao: &Sessao, evento_id: i64, ingresso_id: i64) -> Resultado<()> {
        let (evento, categorias) = self.gerenciar(sessao, evento_id).await?;
        evento.verificar_gerenciavel(self.relogio.hoje())?;
        let ingresso = categorias
            .iter()
            .find(|c| c.id() == ingresso_id)
            .ok_or_else(|| Erro::nao_encontrado("categoria não encontrada neste evento"))?;
        if categorias.len() == 1 {
            return Err(Erro::regra("o evento precisa ter pelo menos uma categoria de ingresso"));
        }
        if ingresso.vendidos() > 0 {
            return Err(Erro::regra("esta categoria já tem ingressos vendidos e não pode ser removida"));
        }
        self.eventos.remover_categoria(ingresso_id).await
    }

    /// Exclui o evento. Se já houve vendas, o histórico dos clientes é preservado:
    /// o organizador deve encerrar as vendas em vez de excluir.
    pub async fn remover(&self, sessao: &Sessao, evento_id: i64) -> Resultado<()> {
        let evento = self.evento_do_organizador(sessao, evento_id).await?;
        if self.eventos.possui_vendas(evento.id()).await? {
            return Err(Erro::regra("o evento já tem vendas e não pode ser excluído. Encerre as vendas em vez disso"));
        }
        self.eventos.remover(evento.id()).await
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
