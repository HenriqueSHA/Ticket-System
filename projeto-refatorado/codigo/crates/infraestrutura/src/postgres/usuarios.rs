use async_trait::async_trait;
use dominio::Resultado;
use dominio::entidades::{Codificavel, Usuario};
use dominio::portas::RepositorioUsuarios;
use dominio::valores::Email;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use super::{decodificar, erro_bd};

pub(crate) const COLUNAS_USUARIO: &str = "id, nome, documento, email, perfil, senha_hash";

pub(crate) fn ler_usuario(r: &PgRow) -> Resultado<Usuario> {
    let email: String = r.try_get("email").map_err(erro_bd)?;
    let perfil: String = r.try_get("perfil").map_err(erro_bd)?;
    Ok(Usuario::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("nome").map_err(erro_bd)?,
        r.try_get("documento").map_err(erro_bd)?,
        Email::novo(&email)?,
        decodificar(&perfil)?,
        r.try_get("senha_hash").map_err(erro_bd)?,
    ))
}

pub struct RepositorioUsuariosPg {
    pool: PgPool,
}

impl RepositorioUsuariosPg {
    pub fn new(pool: PgPool) -> Self {
        RepositorioUsuariosPg { pool }
    }

    async fn buscar_onde(&self, coluna: &str, valor: &str) -> Resultado<Option<Usuario>> {
        let sql = format!("SELECT {COLUNAS_USUARIO} FROM usuarios WHERE {coluna} = $1");
        let linha = sqlx::query(&sql).bind(valor).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_usuario).transpose()
    }
}

#[async_trait]
impl RepositorioUsuarios for RepositorioUsuariosPg {
    async fn inserir(&self, u: &Usuario) -> Resultado<Usuario> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO usuarios (nome, documento, email, senha_hash, perfil) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(u.nome())
        .bind(u.documento())
        .bind(u.email().como_texto())
        .bind(u.senha_hash())
        .bind(u.perfil().codigo())
        .fetch_one(&self.pool)
        .await
        .map_err(erro_bd)?;
        Ok(u.clone().com_id(id))
    }

    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Usuario>> {
        let sql = format!("SELECT {COLUNAS_USUARIO} FROM usuarios WHERE id = $1");
        let linha = sqlx::query(&sql).bind(id).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_usuario).transpose()
    }

    async fn buscar_por_email(&self, email: &Email) -> Resultado<Option<Usuario>> {
        self.buscar_onde("email", email.como_texto()).await
    }

    async fn buscar_por_documento(&self, documento: &str) -> Resultado<Option<Usuario>> {
        self.buscar_onde("documento", documento).await
    }

    async fn contar(&self) -> Resultado<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM usuarios").fetch_one(&self.pool).await.map_err(erro_bd)
    }
}
