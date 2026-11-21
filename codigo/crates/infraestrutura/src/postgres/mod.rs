//! Repositórios PostgreSQL (seção 13 do relatório).

mod eventos;
mod organizadores;
mod usuarios;
mod vendas;

use std::time::Duration;

use dominio::entidades::Codificavel;
use dominio::{Erro, Resultado};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub use eventos::RepositorioEventosPg;
pub use organizadores::RepositorioOrganizadoresPg;
pub use usuarios::RepositorioUsuariosPg;
pub use vendas::RepositorioVendasPg;

/// Conexão com o banco (pool de conexões compartilhado pelos repositórios).
#[derive(Clone)]
pub struct BancoPostgres {
    pool: PgPool,
}

impl BancoPostgres {
    pub async fn conectar(url: &str) -> Resultado<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(url)
            .await
            .map_err(|e| Erro::Infraestrutura(format!("não foi possível conectar ao PostgreSQL ({e})")))?;
        Ok(BancoPostgres { pool })
    }

    /// Cria/atualiza as tabelas executando os arquivos de `migrations/`.
    pub async fn migrar(&self) -> Resultado<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Erro::Infraestrutura(format!("falha ao criar as tabelas: {e}")))
    }

    pub fn usuarios(&self) -> RepositorioUsuariosPg {
        RepositorioUsuariosPg::new(self.pool.clone())
    }
    pub fn organizadores(&self) -> RepositorioOrganizadoresPg {
        RepositorioOrganizadoresPg::new(self.pool.clone())
    }
    pub fn eventos(&self) -> RepositorioEventosPg {
        RepositorioEventosPg::new(self.pool.clone())
    }
    pub fn vendas(&self) -> RepositorioVendasPg {
        RepositorioVendasPg::new(self.pool.clone())
    }
}

/// Converte erros do sqlx em erros do domínio. Restrições violadas no banco viram regra de
/// negócio: UNIQUE (23505 — ex.: assento já vendido), chave estrangeira (23503 — registro com
/// vendas) e CHECK (23514 — ex.: capacidade menor que o já vendido).
pub(crate) fn erro_bd(e: sqlx::Error) -> Erro {
    if let sqlx::Error::Database(db) = &e {
        match db.code().as_deref() {
            Some("23505") => {
                return Erro::regra(match db.constraint() {
                    Some("vendas_um_ativo_por_assento") => "assento já vendido",
                    Some("usuarios_email_key") => "já existe um usuário com este e-mail",
                    Some("usuarios_documento_key") => "já existe um usuário com este CPF/CNPJ",
                    _ => "registro duplicado",
                });
            }
            Some("23503") => return Erro::regra("o registro tem vendas ligadas a ele e não pode ser apagado"),
            Some("23514") => return Erro::regra("valor incompatível com as vendas já realizadas"),
            _ => {}
        }
    }
    Erro::Infraestrutura(format!("erro no banco de dados: {e}"))
}

/// Converte o texto de uma coluna no enum correspondente.
pub(crate) fn decodificar<T: Codificavel>(texto: &str) -> Resultado<T> {
    T::de_codigo(texto).ok_or_else(|| Erro::Infraestrutura(format!("valor inesperado no banco: {texto}")))
}
