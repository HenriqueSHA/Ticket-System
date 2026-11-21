//! **Portas**: interfaces que o domínio e a aplicação usam, mas que são implementadas
//! fora daqui (na infraestrutura).
//!
//! Princípios SOLID aplicados:
//! - **Inversão de Dependência (D)**: os casos de uso dependem destes `trait`s, nunca do PostgreSQL;
//! - **Segregação de Interface (I)**: um repositório por agregado, cada um só com o que precisa;
//! - **Substituição de Liskov (L)**: qualquer implementação (PostgreSQL ou em memória, nos testes)
//!   pode ser usada no lugar da outra sem mudar o comportamento esperado.

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};

use crate::Resultado;
use crate::entidades::*;
use crate::valores::Email;

#[async_trait]
pub trait RepositorioUsuarios: Send + Sync {
    /// Grava o usuário e devolve-o com o `id` gerado.
    async fn inserir(&self, usuario: &Usuario) -> Resultado<Usuario>;
    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Usuario>>;
    async fn buscar_por_email(&self, email: &Email) -> Resultado<Option<Usuario>>;
    async fn buscar_por_documento(&self, documento: &str) -> Resultado<Option<Usuario>>;
    async fn contar(&self) -> Resultado<i64>;
}

#[async_trait]
pub trait RepositorioOrganizadores: Send + Sync {
    /// Grava o usuário de login e o organizador juntos (tudo ou nada).
    async fn inserir(&self, usuario: &Usuario, dados: &NovoOrganizador) -> Resultado<Organizador>;
    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Organizador>>;
    async fn buscar_por_usuario(&self, usuario_id: i64) -> Resultado<Option<Organizador>>;
    async fn listar(&self) -> Resultado<Vec<Organizador>>;
    async fn atualizar(&self, organizador: &Organizador) -> Resultado<()>;
    async fn possui_vendas(&self, organizador_id: i64) -> Resultado<bool>;
    /// Remove o organizador, seus eventos e o usuário de login.
    async fn remover(&self, organizador: &Organizador) -> Resultado<()>;
}

/// O evento é a raiz do agregado: categorias (ingressos) e assentos são acessados por ele.
#[async_trait]
pub trait RepositorioEventos: Send + Sync {
    /// Grava evento, categorias e a matriz de assentos em uma única transação.
    async fn inserir(&self, organizador_id: i64, novo: &NovoEvento) -> Resultado<Evento>;
    async fn buscar(&self, id: i64) -> Resultado<Option<Evento>>;
    /// Abertos, com data posterior a `hoje`, com vagas e com alguma categoria não esgotada.
    async fn listar_disponiveis(&self, hoje: NaiveDate) -> Resultado<Vec<Evento>>;
    async fn listar_do_organizador(&self, organizador_id: i64) -> Resultado<Vec<Evento>>;
    /// Grava dados, capacidade e status do evento.
    async fn atualizar(&self, evento: &Evento) -> Resultado<()>;
    /// `true` se o evento tem qualquer venda (inclusive cancelada — é histórico).
    async fn possui_vendas(&self, evento_id: i64) -> Resultado<bool>;
    /// Exclui evento, categorias e assentos. Recusa se houver vendas.
    async fn remover(&self, evento_id: i64) -> Resultado<()>;
    /// Nova categoria em um evento já cadastrado (gera os assentos, se houver).
    async fn adicionar_categoria(&self, evento_id: i64, categoria: &NovaCategoria) -> Resultado<Ingresso>;
    /// Grava nome, preço e quantidade. A quantidade disponível é recalculada no banco a partir
    /// do que já foi vendido naquele instante; recusa se a nova quantidade ficar abaixo dele.
    async fn atualizar_categoria(&self, ingresso: &Ingresso) -> Resultado<()>;
    /// Remove a categoria e seus assentos. Recusa se houver vendas.
    async fn remover_categoria(&self, ingresso_id: i64) -> Resultado<()>;
    async fn listar_categorias(&self, evento_id: i64) -> Resultado<Vec<Ingresso>>;
    /// Categorias de vários eventos de uma vez (evita uma consulta por evento nas listagens).
    async fn listar_categorias_de(&self, eventos: &[i64]) -> Resultado<Vec<Ingresso>>;
    async fn buscar_categoria(&self, ingresso_id: i64) -> Resultado<Option<Ingresso>>;
    async fn listar_assentos(&self, ingresso_id: i64) -> Resultado<Vec<Assento>>;
    async fn buscar_assento(&self, ingresso_id: i64, fileira: &str, numero: i32) -> Resultado<Option<Assento>>;
}

#[async_trait]
pub trait RepositorioVendas: Send + Sync {
    /// Registra a venda de forma **atômica**: ocupa o assento (se houver, e só se estiver LIVRE),
    /// baixa a quantidade disponível, soma no total vendido do evento e grava a venda.
    /// Se qualquer condição falhar no momento da gravação, nada é alterado.
    async fn registrar(&self, nova: &NovaVenda) -> Resultado<Venda>;
    /// Cancela de forma atômica, devolvendo assento, categoria e vaga do evento.
    async fn cancelar(&self, venda: &Venda) -> Resultado<()>;
    /// Marca como UTILIZADA (check-in). Devolve `false` se a venda não estava confirmada.
    async fn marcar_utilizada(&self, venda_id: i64) -> Resultado<bool>;
    async fn buscar(&self, id: i64) -> Resultado<Option<Venda>>;
    async fn listar_do_evento(&self, evento_id: i64) -> Resultado<Vec<Venda>>;
    /// Vendas do evento com nome do cliente, categoria e assento (mais recentes primeiro).
    async fn listar_detalhadas_do_evento(&self, evento_id: i64) -> Resultado<Vec<VendaDetalhada>>;
    async fn historico_do_cliente(&self, cliente_id: i64) -> Resultado<Vec<ItemHistorico>>;
}

/// Geração e verificação de hash de senha (implementada com Argon2 na infraestrutura).
pub trait HashSenha: Send + Sync {
    fn gerar(&self, senha: &str) -> Resultado<String>;
    fn verificar(&self, senha: &str, hash: &str) -> bool;
}

/// Fonte da data/hora atual. Nos testes usa-se um relógio fixo para resultados reproduzíveis.
pub trait Relogio: Send + Sync {
    fn agora(&self) -> NaiveDateTime;

    fn hoje(&self) -> NaiveDate {
        self.agora().date()
    }
}
