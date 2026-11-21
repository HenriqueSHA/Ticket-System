//! Repositórios em memória (listas protegidas por `Mutex`).
//!
//! Implementam exatamente os mesmos `trait`s e regras dos repositórios PostgreSQL
//! (unicidade, operações atômicas), por isso podem substituí-los nos testes automatizados
//! — Princípio da Substituição de Liskov.

use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use chrono::NaiveDate;
use dominio::entidades::*;
use dominio::portas::*;
use dominio::valores::Email;
use dominio::{Erro, Resultado};

#[derive(Default)]
struct Tabelas {
    proximo_id: i64,
    usuarios: Vec<Usuario>,
    organizadores: Vec<Organizador>,
    eventos: Vec<Evento>,
    ingressos: Vec<Ingresso>,
    assentos: Vec<Assento>,
    vendas: Vec<Venda>,
}

impl Tabelas {
    fn novo_id(&mut self) -> i64 {
        self.proximo_id += 1;
        self.proximo_id
    }

    fn inserir_usuario(&mut self, u: &Usuario) -> Resultado<Usuario> {
        if self.usuarios.iter().any(|x| x.email() == u.email()) {
            return Err(Erro::regra("já existe um usuário com este e-mail"));
        }
        if self.usuarios.iter().any(|x| x.documento() == u.documento()) {
            return Err(Erro::regra("já existe um usuário com este CPF/CNPJ"));
        }
        let novo = u.clone().com_id(self.novo_id());
        self.usuarios.push(novo.clone());
        Ok(novo)
    }

    fn evento_mut(&mut self, id: i64) -> Option<&mut Evento> {
        self.eventos.iter_mut().find(|e| e.id() == id)
    }

    /// Grava a categoria e gera a matriz de assentos (dois laços aninhados).
    fn criar_categoria(&mut self, evento_id: i64, cat: &NovaCategoria) -> Ingresso {
        let ingresso = Ingresso::reconstruir(
            self.novo_id(),
            evento_id,
            cat.nome().to_string(),
            cat.preco(),
            cat.quantidade(),
            cat.quantidade(),
            cat.setor(),
        );
        if let Some(s) = cat.setor() {
            for f in 0..s.fileiras {
                for n in 1..=s.assentos_por_fileira {
                    let id = self.novo_id();
                    let fileira = letra_fileira(f as usize);
                    self.assentos.push(Assento::reconstruir(id, ingresso.id(), cat.nome().into(), fileira, n, Situacao::Livre));
                }
            }
        }
        self.ingressos.push(ingresso.clone());
        ingresso
    }

    fn evento_possui_vendas(&self, evento_id: i64) -> bool {
        self.vendas.iter().any(|v| v.evento_id() == evento_id)
    }
}

/// Banco em memória: uma única instância implementa os quatro repositórios.
#[derive(Clone, Default)]
pub struct RepositoriosMemoria {
    tabelas: Arc<Mutex<Tabelas>>,
}

impl RepositoriosMemoria {
    pub fn novo() -> Self {
        Self::default()
    }

    fn t(&self) -> MutexGuard<'_, Tabelas> {
        self.tabelas.lock().expect("repositório em memória")
    }
}

fn com_total(e: &Evento, delta: i32) -> Evento {
    Evento::reconstruir(e.id(), e.organizador_id(), e.dados().clone(), e.total_vendido() + delta, e.status())
}

fn com_disponivel(i: &Ingresso, delta: i32) -> Ingresso {
    Ingresso::reconstruir(
        i.id(),
        i.evento_id(),
        i.categoria().to_string(),
        i.preco(),
        i.quantidade(),
        i.disponivel() + delta,
        i.setor(),
    )
}

fn com_situacao(a: &Assento, situacao: Situacao) -> Assento {
    Assento::reconstruir(a.id(), a.ingresso_id(), a.setor().to_string(), a.fileira().to_string(), a.numero(), situacao)
}

#[async_trait]
impl RepositorioUsuarios for RepositoriosMemoria {
    async fn inserir(&self, usuario: &Usuario) -> Resultado<Usuario> {
        self.t().inserir_usuario(usuario)
    }
    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Usuario>> {
        Ok(self.t().usuarios.iter().find(|u| u.id() == id).cloned())
    }
    async fn buscar_por_email(&self, email: &Email) -> Resultado<Option<Usuario>> {
        Ok(self.t().usuarios.iter().find(|u| u.email() == email).cloned())
    }
    async fn buscar_por_documento(&self, documento: &str) -> Resultado<Option<Usuario>> {
        Ok(self.t().usuarios.iter().find(|u| u.documento() == documento).cloned())
    }
    async fn contar(&self) -> Resultado<i64> {
        Ok(self.t().usuarios.len() as i64)
    }
}

#[async_trait]
impl RepositorioOrganizadores for RepositoriosMemoria {
    async fn inserir(&self, usuario: &Usuario, dados: &NovoOrganizador) -> Resultado<Organizador> {
        let mut t = self.t();
        let login = t.inserir_usuario(usuario)?;
        let org = Organizador::reconstruir(
            t.novo_id(),
            login.id(),
            dados.razao_social().to_string(),
            dados.cnpj().clone(),
            dados.contato().to_string(),
            StatusOrganizador::Ativo,
        );
        t.organizadores.push(org.clone());
        Ok(org)
    }
    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Organizador>> {
        Ok(self.t().organizadores.iter().find(|o| o.id() == id).cloned())
    }
    async fn buscar_por_usuario(&self, usuario_id: i64) -> Resultado<Option<Organizador>> {
        Ok(self.t().organizadores.iter().find(|o| o.usuario_id() == usuario_id).cloned())
    }
    async fn listar(&self) -> Resultado<Vec<Organizador>> {
        let mut lista = self.t().organizadores.clone();
        lista.sort_by(|a, b| a.razao_social().cmp(b.razao_social()));
        Ok(lista)
    }
    async fn atualizar(&self, org: &Organizador) -> Resultado<()> {
        let mut t = self.t();
        if let Some(o) = t.organizadores.iter_mut().find(|o| o.id() == org.id()) {
            *o = org.clone();
        }
        Ok(())
    }
    async fn possui_vendas(&self, organizador_id: i64) -> Resultado<bool> {
        let t = self.t();
        Ok(t.vendas.iter().any(|v| t.eventos.iter().any(|e| e.id() == v.evento_id() && e.pertence_a(organizador_id))))
    }
    async fn remover(&self, org: &Organizador) -> Resultado<()> {
        let mut t = self.t();
        let eventos: Vec<i64> = t.eventos.iter().filter(|e| e.pertence_a(org.id())).map(Evento::id).collect();
        let ingressos: Vec<i64> =
            t.ingressos.iter().filter(|i| eventos.contains(&i.evento_id())).map(Ingresso::id).collect();
        t.assentos.retain(|a| !ingressos.contains(&a.ingresso_id()));
        t.ingressos.retain(|i| !eventos.contains(&i.evento_id()));
        t.eventos.retain(|e| !e.pertence_a(org.id()));
        t.organizadores.retain(|o| o.id() != org.id());
        t.usuarios.retain(|u| u.id() != org.usuario_id());
        Ok(())
    }
}

#[async_trait]
impl RepositorioEventos for RepositoriosMemoria {
    async fn inserir(&self, organizador_id: i64, novo: &NovoEvento) -> Resultado<Evento> {
        let mut t = self.t();
        let evento = Evento::reconstruir(t.novo_id(), organizador_id, novo.dados().clone(), 0, StatusEvento::Aberto);
        for cat in novo.categorias() {
            t.criar_categoria(evento.id(), cat);
        }
        t.eventos.push(evento.clone());
        Ok(evento)
    }
    async fn buscar(&self, id: i64) -> Resultado<Option<Evento>> {
        Ok(self.t().eventos.iter().find(|e| e.id() == id).cloned())
    }
    async fn listar_disponiveis(&self, hoje: NaiveDate) -> Resultado<Vec<Evento>> {
        let t = self.t();
        let mut lista: Vec<Evento> = t
            .eventos
            .iter()
            .filter(|e| e.status() == StatusEvento::Aberto && e.data() > hoje && !e.esta_lotado())
            .filter(|e| t.ingressos.iter().any(|i| i.evento_id() == e.id() && !i.esgotado()))
            .cloned()
            .collect();
        lista.sort_by_key(|e| (e.data(), e.horario()));
        Ok(lista)
    }
    async fn listar_do_organizador(&self, organizador_id: i64) -> Resultado<Vec<Evento>> {
        let mut lista: Vec<Evento> = self.t().eventos.iter().filter(|e| e.pertence_a(organizador_id)).cloned().collect();
        lista.sort_by_key(|e| (e.data(), e.horario()));
        Ok(lista)
    }
    /// Como o UPDATE do PostgreSQL: grava dados, capacidade e status, mas o total vendido
    /// continua o atual (pode ter havido vendas desde que o evento foi lido).
    async fn atualizar(&self, evento: &Evento) -> Resultado<()> {
        let mut t = self.t();
        if let Some(e) = t.evento_mut(evento.id()) {
            *e = Evento::reconstruir(evento.id(), e.organizador_id(), evento.dados().clone(), e.total_vendido(), evento.status());
        }
        Ok(())
    }
    async fn possui_vendas(&self, evento_id: i64) -> Resultado<bool> {
        Ok(self.t().evento_possui_vendas(evento_id))
    }
    async fn remover(&self, evento_id: i64) -> Resultado<()> {
        let mut t = self.t();
        if t.evento_possui_vendas(evento_id) {
            return Err(Erro::regra("o evento já tem vendas e não pode ser excluído"));
        }
        let ingressos: Vec<i64> = t.ingressos.iter().filter(|i| i.evento_id() == evento_id).map(Ingresso::id).collect();
        t.assentos.retain(|a| !ingressos.contains(&a.ingresso_id()));
        t.ingressos.retain(|i| i.evento_id() != evento_id);
        t.eventos.retain(|e| e.id() != evento_id);
        Ok(())
    }
    async fn adicionar_categoria(&self, evento_id: i64, categoria: &NovaCategoria) -> Resultado<Ingresso> {
        Ok(self.t().criar_categoria(evento_id, categoria))
    }
    async fn atualizar_categoria(&self, novo: &Ingresso) -> Resultado<()> {
        let mut t = self.t();
        let pos = t
            .ingressos
            .iter()
            .position(|i| i.id() == novo.id())
            .ok_or_else(|| Erro::nao_encontrado("categoria não encontrada"))?;
        let vendidos = t.ingressos[pos].vendidos(); // valor atual, como no UPDATE do PostgreSQL
        if novo.quantidade() < vendidos {
            return Err(Erro::regra("a quantidade não pode ser menor que a de ingressos já vendidos"));
        }
        t.ingressos[pos] = Ingresso::reconstruir(
            novo.id(),
            novo.evento_id(),
            novo.categoria().to_string(),
            novo.preco(),
            novo.quantidade(),
            novo.quantidade() - vendidos,
            novo.setor(),
        );
        Ok(())
    }
    async fn remover_categoria(&self, ingresso_id: i64) -> Resultado<()> {
        let mut t = self.t();
        if t.vendas.iter().any(|v| v.ingresso_id() == ingresso_id) {
            return Err(Erro::regra("esta categoria já tem vendas e não pode ser removida"));
        }
        t.assentos.retain(|a| a.ingresso_id() != ingresso_id);
        t.ingressos.retain(|i| i.id() != ingresso_id);
        Ok(())
    }
    async fn listar_categorias(&self, evento_id: i64) -> Resultado<Vec<Ingresso>> {
        Ok(self.t().ingressos.iter().filter(|i| i.evento_id() == evento_id).cloned().collect())
    }
    async fn listar_categorias_de(&self, eventos: &[i64]) -> Resultado<Vec<Ingresso>> {
        Ok(self.t().ingressos.iter().filter(|i| eventos.contains(&i.evento_id())).cloned().collect())
    }
    async fn buscar_categoria(&self, ingresso_id: i64) -> Resultado<Option<Ingresso>> {
        Ok(self.t().ingressos.iter().find(|i| i.id() == ingresso_id).cloned())
    }
    async fn listar_assentos(&self, ingresso_id: i64) -> Resultado<Vec<Assento>> {
        Ok(self.t().assentos.iter().filter(|a| a.ingresso_id() == ingresso_id).cloned().collect())
    }
    async fn buscar_assento(&self, ingresso_id: i64, fileira: &str, numero: i32) -> Resultado<Option<Assento>> {
        Ok(self
            .t()
            .assentos
            .iter()
            .find(|a| a.ingresso_id() == ingresso_id && a.fileira() == fileira && a.numero() == numero)
            .cloned())
    }
}

#[async_trait]
impl RepositorioVendas for RepositoriosMemoria {
    /// Atômico: o `Mutex` fica travado do início ao fim, e todas as condições são
    /// verificadas antes de qualquer alteração.
    async fn registrar(&self, nova: &NovaVenda) -> Resultado<Venda> {
        let mut t = self.t();
        let pos_assento = match nova.assento_id {
            Some(id) => match t.assentos.iter().position(|a| a.id() == id) {
                Some(p) if t.assentos[p].livre() => Some(p),
                _ => return Err(Erro::regra("assento já vendido")),
            },
            None => None,
        };
        let pos_ingresso = t
            .ingressos
            .iter()
            .position(|i| i.id() == nova.ingresso_id && !i.esgotado())
            .ok_or_else(|| Erro::regra("categoria esgotada"))?;
        let pos_evento = t
            .eventos
            .iter()
            .position(|e| e.id() == nova.evento_id && !e.esta_lotado())
            .ok_or_else(|| Erro::regra("evento lotado"))?;

        if let Some(p) = pos_assento {
            t.assentos[p] = com_situacao(&t.assentos[p], Situacao::Ocupado);
        }
        t.ingressos[pos_ingresso] = com_disponivel(&t.ingressos[pos_ingresso], -1);
        t.eventos[pos_evento] = com_total(&t.eventos[pos_evento], 1);
        let venda = Venda::registrada(t.novo_id(), nova);
        t.vendas.push(venda.clone());
        Ok(venda)
    }

    async fn cancelar(&self, venda: &Venda) -> Resultado<()> {
        let mut t = self.t();
        let pos = t
            .vendas
            .iter()
            .position(|v| v.id() == venda.id() && v.status() == StatusVenda::Confirmada)
            .ok_or_else(|| Erro::regra("a compra não está mais confirmada"))?;
        t.vendas[pos].marcar_cancelada();
        if let Some(a) = t.assentos.iter_mut().find(|a| Some(a.id()) == venda.assento_id()) {
            *a = com_situacao(a, Situacao::Livre);
        }
        if let Some(i) = t.ingressos.iter_mut().find(|i| i.id() == venda.ingresso_id()) {
            *i = com_disponivel(i, 1);
        }
        if let Some(e) = t.evento_mut(venda.evento_id()) {
            *e = com_total(e, -1);
        }
        Ok(())
    }

    async fn marcar_utilizada(&self, venda_id: i64) -> Resultado<bool> {
        let mut t = self.t();
        match t.vendas.iter_mut().find(|v| v.id() == venda_id && v.status() == StatusVenda::Confirmada) {
            Some(v) => {
                v.marcar_utilizada();
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn buscar(&self, id: i64) -> Resultado<Option<Venda>> {
        Ok(self.t().vendas.iter().find(|v| v.id() == id).cloned())
    }

    async fn listar_do_evento(&self, evento_id: i64) -> Resultado<Vec<Venda>> {
        Ok(self.t().vendas.iter().filter(|v| v.evento_id() == evento_id).cloned().collect())
    }

    async fn listar_detalhadas_do_evento(&self, evento_id: i64) -> Resultado<Vec<VendaDetalhada>> {
        let t = self.t();
        let mut lista: Vec<VendaDetalhada> = t
            .vendas
            .iter()
            .filter(|v| v.evento_id() == evento_id)
            .map(|v| VendaDetalhada {
                venda: v.clone(),
                cliente: t.usuarios.iter().find(|u| u.id() == v.cliente_id()).map_or_else(String::new, |u| u.nome().to_string()),
                categoria: t.ingressos.iter().find(|i| i.id() == v.ingresso_id()).map_or_else(String::new, |i| i.categoria().to_string()),
                assento: t.assentos.iter().find(|a| Some(a.id()) == v.assento_id()).map(Assento::rotulo),
            })
            .collect();
        lista.sort_by_key(|d| std::cmp::Reverse(d.venda.id()));
        Ok(lista)
    }

    async fn historico_do_cliente(&self, cliente_id: i64) -> Resultado<Vec<ItemHistorico>> {
        let t = self.t();
        let mut itens = Vec::new();
        for v in t.vendas.iter().filter(|v| v.cliente_id() == cliente_id) {
            let (Some(e), Some(i)) = (
                t.eventos.iter().find(|e| e.id() == v.evento_id()),
                t.ingressos.iter().find(|i| i.id() == v.ingresso_id()),
            ) else {
                continue;
            };
            itens.push(ItemHistorico {
                venda: v.clone(),
                evento: e.nome().to_string(),
                data_evento: e.data(),
                horario: e.horario(),
                local: e.local().to_string(),
                categoria: i.categoria().to_string(),
                assento: t.assentos.iter().find(|a| Some(a.id()) == v.assento_id()).map(Assento::rotulo),
                permite_cancelamento: e.permite_cancelamento(),
            });
        }
        itens.sort_by(|a, b| b.data_evento.cmp(&a.data_evento).then(b.venda.id().cmp(&a.venda.id())));
        Ok(itens)
    }
}

#[cfg(test)]
mod testes {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use dominio::valores::Dinheiro;

    use super::*;

    fn agora() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().and_hms_opt(12, 0, 0).unwrap()
    }

    /// Mesmo comportamento do UPDATE do PostgreSQL: atualizar o evento com uma cópia lida antes
    /// de uma venda não pode apagar essa venda do total vendido.
    #[tokio::test]
    async fn atualizar_evento_preserva_total_vendido() {
        let repo = RepositoriosMemoria::novo();
        let dados = DadosEvento {
            nome: "Show".into(),
            descricao: String::new(),
            local: "Arena".into(),
            data: NaiveDate::from_ymd_opt(2026, 12, 20).unwrap(),
            horario: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            capacidade: 10,
            permite_cancelamento: true,
        };
        let mut b = EventoBuilder::novo(dados, agora().date()).unwrap();
        b.adicionar_categoria(NovaCategoria::sem_assento("Pista", Dinheiro::ZERO, 10).unwrap()).unwrap();
        let evento = RepositorioEventos::inserir(&repo, 1, &b.construir().unwrap()).await.unwrap();
        let copia_antiga = evento.clone();
        let cat = repo.listar_categorias(evento.id()).await.unwrap().remove(0);
        let nova = NovaVenda {
            data_compra: agora(),
            valor: Dinheiro::ZERO,
            tipo: TipoIngresso::Inteira,
            forma_pagamento: FormaPagamento::Pix,
            cliente_id: 1,
            evento_id: evento.id(),
            ingresso_id: cat.id(),
            assento_id: None,
        };
        repo.registrar(&nova).await.unwrap();

        RepositorioEventos::atualizar(&repo, &copia_antiga).await.unwrap();
        assert_eq!(RepositorioEventos::buscar(&repo, evento.id()).await.unwrap().unwrap().total_vendido(), 1);
    }
}
