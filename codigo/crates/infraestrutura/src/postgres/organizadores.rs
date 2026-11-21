use async_trait::async_trait;
use dominio::Resultado;
use dominio::entidades::{Codificavel, NovoOrganizador, Organizador, StatusOrganizador, Usuario};
use dominio::portas::RepositorioOrganizadores;
use dominio::valores::Cnpj;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use super::{decodificar, erro_bd};

const COLUNAS: &str = "id, usuario_id, razao_social, cnpj, contato, status";

fn ler_organizador(r: &PgRow) -> Resultado<Organizador> {
    let cnpj: String = r.try_get("cnpj").map_err(erro_bd)?;
    let status: String = r.try_get("status").map_err(erro_bd)?;
    Ok(Organizador::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("usuario_id").map_err(erro_bd)?,
        r.try_get("razao_social").map_err(erro_bd)?,
        Cnpj::novo(&cnpj)?,
        r.try_get("contato").map_err(erro_bd)?,
        decodificar(&status)?,
    ))
}

pub struct RepositorioOrganizadoresPg {
    pool: PgPool,
}

impl RepositorioOrganizadoresPg {
    pub fn new(pool: PgPool) -> Self {
        RepositorioOrganizadoresPg { pool }
    }

    async fn buscar_onde(&self, coluna: &str, valor: i64) -> Resultado<Option<Organizador>> {
        let sql = format!("SELECT {COLUNAS} FROM organizadores WHERE {coluna} = $1");
        let linha = sqlx::query(&sql).bind(valor).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_organizador).transpose()
    }
}

#[async_trait]
impl RepositorioOrganizadores for RepositorioOrganizadoresPg {
    async fn inserir(&self, u: &Usuario, dados: &NovoOrganizador) -> Resultado<Organizador> {
        // transação: usuário de login + organizador, tudo ou nada
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        let usuario_id: i64 = sqlx::query_scalar(
            "INSERT INTO usuarios (nome, documento, email, senha_hash, perfil) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(u.nome())
        .bind(u.documento())
        .bind(u.email().como_texto())
        .bind(u.senha_hash())
        .bind(u.perfil().codigo())
        .fetch_one(&mut *tx)
        .await
        .map_err(erro_bd)?;
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO organizadores (usuario_id, razao_social, cnpj, contato, status)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(usuario_id)
        .bind(dados.razao_social())
        .bind(dados.cnpj().digitos())
        .bind(dados.contato())
        .bind(StatusOrganizador::Ativo.codigo())
        .fetch_one(&mut *tx)
        .await
        .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)?;
        Ok(Organizador::reconstruir(
            id,
            usuario_id,
            dados.razao_social().to_string(),
            dados.cnpj().clone(),
            dados.contato().to_string(),
            StatusOrganizador::Ativo,
        ))
    }

    async fn buscar_por_id(&self, id: i64) -> Resultado<Option<Organizador>> {
        self.buscar_onde("id", id).await
    }

    async fn buscar_por_usuario(&self, usuario_id: i64) -> Resultado<Option<Organizador>> {
        self.buscar_onde("usuario_id", usuario_id).await
    }

    async fn listar(&self) -> Resultado<Vec<Organizador>> {
        let sql = format!("SELECT {COLUNAS} FROM organizadores ORDER BY razao_social");
        let linhas = sqlx::query(&sql).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_organizador).collect()
    }

    async fn atualizar(&self, o: &Organizador) -> Resultado<()> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        sqlx::query("UPDATE organizadores SET razao_social = $1, contato = $2, status = $3 WHERE id = $4")
            .bind(o.razao_social())
            .bind(o.contato())
            .bind(o.status().codigo())
            .bind(o.id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        sqlx::query("UPDATE usuarios SET nome = $1 WHERE id = $2")
            .bind(o.razao_social())
            .bind(o.usuario_id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)
    }

    async fn possui_vendas(&self, organizador_id: i64) -> Resultado<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM vendas v JOIN ingressos i ON i.id = v.ingresso_id
                            JOIN eventos e ON e.id = i.evento_id WHERE e.organizador_id = $1)",
        )
        .bind(organizador_id)
        .fetch_one(&self.pool)
        .await
        .map_err(erro_bd)
    }

    async fn remover(&self, o: &Organizador) -> Resultado<()> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        for sql in [
            "DELETE FROM eventos WHERE organizador_id = $1",
            "DELETE FROM organizadores WHERE id = $1",
        ] {
            sqlx::query(sql).bind(o.id()).execute(&mut *tx).await.map_err(erro_bd)?;
        }
        sqlx::query("DELETE FROM usuarios WHERE id = $1")
            .bind(o.usuario_id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)
    }
}
