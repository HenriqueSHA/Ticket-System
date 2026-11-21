use async_trait::async_trait;
use chrono::NaiveDate;
use dominio::{Erro, Resultado};
use dominio::entidades::*;
use dominio::portas::RepositorioEventos;
use dominio::valores::Dinheiro;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Postgres, Row, Transaction};

use super::{decodificar, erro_bd};

const COLUNAS_EVENTO: &str = "id, organizador_id, nome, descricao, local, data, horario, capacidade, \
                              total_vendido, permite_cancelamento, status";
const COLUNAS_INGRESSO: &str =
    "id, evento_id, categoria, preco_centavos, quantidade, disponivel, com_assento, fileiras, assentos_por_fileira";
const COLUNAS_ASSENTO: &str = "id, ingresso_id, setor, fileira, numero, situacao";

fn ler_evento(r: &PgRow) -> Resultado<Evento> {
    let status: String = r.try_get("status").map_err(erro_bd)?;
    let dados = DadosEvento {
        nome: r.try_get("nome").map_err(erro_bd)?,
        descricao: r.try_get("descricao").map_err(erro_bd)?,
        local: r.try_get("local").map_err(erro_bd)?,
        data: r.try_get("data").map_err(erro_bd)?,
        horario: r.try_get("horario").map_err(erro_bd)?,
        capacidade: r.try_get("capacidade").map_err(erro_bd)?,
        permite_cancelamento: r.try_get("permite_cancelamento").map_err(erro_bd)?,
    };
    Ok(Evento::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("organizador_id").map_err(erro_bd)?,
        dados,
        r.try_get("total_vendido").map_err(erro_bd)?,
        decodificar(&status)?,
    ))
}

fn ler_ingresso(r: &PgRow) -> Resultado<Ingresso> {
    let com_assento: bool = r.try_get("com_assento").map_err(erro_bd)?;
    let setor = com_assento
        .then(|| -> Resultado<Setor> {
            Ok(Setor {
                fileiras: r.try_get("fileiras").map_err(erro_bd)?,
                assentos_por_fileira: r.try_get("assentos_por_fileira").map_err(erro_bd)?,
            })
        })
        .transpose()?;
    Ok(Ingresso::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("evento_id").map_err(erro_bd)?,
        r.try_get("categoria").map_err(erro_bd)?,
        Dinheiro::de_centavos(r.try_get("preco_centavos").map_err(erro_bd)?)?,
        r.try_get("quantidade").map_err(erro_bd)?,
        r.try_get("disponivel").map_err(erro_bd)?,
        setor,
    ))
}

fn ler_assento(r: &PgRow) -> Resultado<Assento> {
    let situacao: String = r.try_get("situacao").map_err(erro_bd)?;
    Ok(Assento::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("ingresso_id").map_err(erro_bd)?,
        r.try_get("setor").map_err(erro_bd)?,
        r.try_get("fileira").map_err(erro_bd)?,
        r.try_get("numero").map_err(erro_bd)?,
        decodificar(&situacao)?,
    ))
}

/// Grava uma categoria e gera a matriz de assentos (gerarAssentos: dois laços aninhados).
async fn inserir_categoria(tx: &mut Transaction<'_, Postgres>, evento_id: i64, cat: &NovaCategoria) -> Resultado<Ingresso> {
    let setor = cat.setor();
    let ingresso_id: i64 = sqlx::query_scalar(
        "INSERT INTO ingressos (evento_id, categoria, preco_centavos, quantidade, disponivel, com_assento,
                                fileiras, assentos_por_fileira)
         VALUES ($1, $2, $3, $4, $4, $5, $6, $7) RETURNING id",
    )
    .bind(evento_id)
    .bind(cat.nome())
    .bind(cat.preco().centavos())
    .bind(cat.quantidade())
    .bind(setor.is_some())
    .bind(setor.map_or(0, |s| s.fileiras))
    .bind(setor.map_or(0, |s| s.assentos_por_fileira))
    .fetch_one(&mut **tx)
    .await
    .map_err(erro_bd)?;

    if let Some(s) = setor {
        for f in 0..s.fileiras {
            for n in 1..=s.assentos_por_fileira {
                sqlx::query(
                    "INSERT INTO assentos (evento_id, ingresso_id, setor, fileira, numero, situacao)
                     VALUES ($1, $2, $3, $4, $5, 'LIVRE')",
                )
                .bind(evento_id)
                .bind(ingresso_id)
                .bind(cat.nome())
                .bind(letra_fileira(f as usize))
                .bind(n)
                .execute(&mut **tx)
                .await
                .map_err(erro_bd)?;
            }
        }
    }
    Ok(Ingresso::reconstruir(
        ingresso_id,
        evento_id,
        cat.nome().to_string(),
        cat.preco(),
        cat.quantidade(),
        cat.quantidade(),
        setor,
    ))
}

pub struct RepositorioEventosPg {
    pool: PgPool,
}

impl RepositorioEventosPg {
    pub fn new(pool: PgPool) -> Self {
        RepositorioEventosPg { pool }
    }
}

#[async_trait]
impl RepositorioEventos for RepositorioEventosPg {
    async fn inserir(&self, organizador_id: i64, novo: &NovoEvento) -> Resultado<Evento> {
        let d = novo.dados();
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        let evento_id: i64 = sqlx::query_scalar(
            "INSERT INTO eventos (organizador_id, nome, descricao, local, data, horario, capacidade,
                                  total_vendido, permite_cancelamento, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8, $9) RETURNING id",
        )
        .bind(organizador_id)
        .bind(&d.nome)
        .bind(&d.descricao)
        .bind(&d.local)
        .bind(d.data)
        .bind(d.horario)
        .bind(d.capacidade)
        .bind(d.permite_cancelamento)
        .bind(StatusEvento::Aberto.codigo())
        .fetch_one(&mut *tx)
        .await
        .map_err(erro_bd)?;

        for cat in novo.categorias() {
            inserir_categoria(&mut tx, evento_id, cat).await?;
        }
        tx.commit().await.map_err(erro_bd)?;
        Ok(Evento::reconstruir(evento_id, organizador_id, d.clone(), 0, StatusEvento::Aberto))
    }

    async fn buscar(&self, id: i64) -> Resultado<Option<Evento>> {
        let sql = format!("SELECT {COLUNAS_EVENTO} FROM eventos WHERE id = $1");
        let linha = sqlx::query(&sql).bind(id).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_evento).transpose()
    }

    async fn listar_disponiveis(&self, hoje: NaiveDate) -> Resultado<Vec<Evento>> {
        let sql = format!(
            "SELECT {COLUNAS_EVENTO} FROM eventos
             WHERE status = 'ABERTO' AND data > $1 AND total_vendido < capacidade
               AND EXISTS (SELECT 1 FROM ingressos i WHERE i.evento_id = eventos.id AND i.disponivel > 0)
             ORDER BY data, horario"
        );
        let linhas = sqlx::query(&sql).bind(hoje).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_evento).collect()
    }

    async fn listar_do_organizador(&self, organizador_id: i64) -> Resultado<Vec<Evento>> {
        let sql = format!("SELECT {COLUNAS_EVENTO} FROM eventos WHERE organizador_id = $1 ORDER BY data, horario");
        let linhas = sqlx::query(&sql).bind(organizador_id).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_evento).collect()
    }

    async fn atualizar(&self, e: &Evento) -> Resultado<()> {
        sqlx::query(
            "UPDATE eventos SET nome = $1, descricao = $2, local = $3, data = $4, horario = $5,
                                permite_cancelamento = $6, status = $7, capacidade = $8 WHERE id = $9",
        )
        .bind(e.nome())
        .bind(e.descricao())
        .bind(e.local())
        .bind(e.data())
        .bind(e.horario())
        .bind(e.permite_cancelamento())
        .bind(e.status().codigo())
        .bind(e.capacidade())
        .bind(e.id())
        .execute(&self.pool)
        .await
        .map_err(erro_bd)?;
        Ok(())
    }

    async fn possui_vendas(&self, evento_id: i64) -> Resultado<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM vendas v JOIN ingressos i ON i.id = v.ingresso_id WHERE i.evento_id = $1)",
        )
        .bind(evento_id)
        .fetch_one(&self.pool)
        .await
        .map_err(erro_bd)
    }

    async fn remover(&self, evento_id: i64) -> Resultado<()> {
        // ingressos e assentos são apagados em cascata; vendas impedem a exclusão (chave estrangeira)
        let r = sqlx::query(
            "DELETE FROM eventos e WHERE e.id = $1 AND NOT EXISTS
               (SELECT 1 FROM vendas v JOIN ingressos i ON i.id = v.ingresso_id WHERE i.evento_id = e.id)",
        )
        .bind(evento_id)
        .execute(&self.pool)
        .await
        .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("o evento já tem vendas e não pode ser excluído"));
        }
        Ok(())
    }

    async fn adicionar_categoria(&self, evento_id: i64, categoria: &NovaCategoria) -> Resultado<Ingresso> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        let ingresso = inserir_categoria(&mut tx, evento_id, categoria).await?;
        tx.commit().await.map_err(erro_bd)?;
        Ok(ingresso)
    }

    async fn atualizar_categoria(&self, i: &Ingresso) -> Resultado<()> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        // vendidos = quantidade - disponivel, lidos no próprio UPDATE (valores atuais da linha)
        let r = sqlx::query(
            "UPDATE ingressos SET categoria = $1, preco_centavos = $2, quantidade = $3,
                                  disponivel = $3 - (quantidade - disponivel)
             WHERE id = $4 AND $3 >= quantidade - disponivel",
        )
        .bind(i.categoria())
        .bind(i.preco().centavos())
        .bind(i.quantidade())
        .bind(i.id())
        .execute(&mut *tx)
        .await
        .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("a quantidade não pode ser menor que a de ingressos já vendidos"));
        }
        sqlx::query("UPDATE assentos SET setor = $1 WHERE ingresso_id = $2")
            .bind(i.categoria())
            .bind(i.id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)
    }

    async fn remover_categoria(&self, ingresso_id: i64) -> Resultado<()> {
        let r = sqlx::query("DELETE FROM ingressos WHERE id = $1 AND NOT EXISTS (SELECT 1 FROM vendas WHERE ingresso_id = $1)")
            .bind(ingresso_id)
            .execute(&self.pool)
            .await
            .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("esta categoria já tem vendas e não pode ser removida"));
        }
        Ok(())
    }

    async fn listar_categorias(&self, evento_id: i64) -> Resultado<Vec<Ingresso>> {
        let sql = format!("SELECT {COLUNAS_INGRESSO} FROM ingressos WHERE evento_id = $1 ORDER BY id");
        let linhas = sqlx::query(&sql).bind(evento_id).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_ingresso).collect()
    }

    async fn buscar_categoria(&self, ingresso_id: i64) -> Resultado<Option<Ingresso>> {
        let sql = format!("SELECT {COLUNAS_INGRESSO} FROM ingressos WHERE id = $1");
        let linha = sqlx::query(&sql).bind(ingresso_id).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_ingresso).transpose()
    }

    async fn listar_assentos(&self, ingresso_id: i64) -> Resultado<Vec<Assento>> {
        let sql = format!("SELECT {COLUNAS_ASSENTO} FROM assentos WHERE ingresso_id = $1 ORDER BY fileira, numero");
        let linhas = sqlx::query(&sql).bind(ingresso_id).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_assento).collect()
    }

    async fn buscar_assento(&self, ingresso_id: i64, fileira: &str, numero: i32) -> Resultado<Option<Assento>> {
        let sql = format!("SELECT {COLUNAS_ASSENTO} FROM assentos WHERE ingresso_id = $1 AND fileira = $2 AND numero = $3");
        let linha = sqlx::query(&sql)
            .bind(ingresso_id)
            .bind(fileira)
            .bind(numero)
            .fetch_optional(&self.pool)
            .await
            .map_err(erro_bd)?;
        linha.as_ref().map(ler_assento).transpose()
    }
}
