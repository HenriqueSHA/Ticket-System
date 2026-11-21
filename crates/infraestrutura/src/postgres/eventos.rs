use async_trait::async_trait;
use chrono::NaiveDate;
use dominio::Resultado;
use dominio::entidades::*;
use dominio::portas::RepositorioEventos;
use dominio::valores::Dinheiro;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

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
            .fetch_one(&mut *tx)
            .await
            .map_err(erro_bd)?;

            // gerarAssentos: dois laços aninhados percorrendo a matriz fileira × número
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
                        .execute(&mut *tx)
                        .await
                        .map_err(erro_bd)?;
                    }
                }
            }
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
                                permite_cancelamento = $6, status = $7 WHERE id = $8",
        )
        .bind(e.nome())
        .bind(e.descricao())
        .bind(e.local())
        .bind(e.data())
        .bind(e.horario())
        .bind(e.permite_cancelamento())
        .bind(e.status().codigo())
        .bind(e.id())
        .execute(&self.pool)
        .await
        .map_err(erro_bd)?;
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
