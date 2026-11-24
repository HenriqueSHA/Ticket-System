use async_trait::async_trait;
use dominio::entidades::*;
use dominio::portas::RepositorioVendas;
use dominio::valores::Dinheiro;
use dominio::{Erro, Resultado};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use super::{decodificar, erro_bd};

/// Toda consulta de venda junta com `ingressos` para saber o evento.
const SELECT_VENDA: &str = "SELECT v.id, v.data_compra, v.valor_centavos, v.tipo, v.forma_pagamento, v.status,
                                   v.cliente_id, i.evento_id, v.ingresso_id, v.assento_id
                            FROM vendas v JOIN ingressos i ON i.id = v.ingresso_id";

fn ler_venda(r: &PgRow) -> Resultado<Venda> {
    let tipo: String = r.try_get("tipo").map_err(erro_bd)?;
    let forma: String = r.try_get("forma_pagamento").map_err(erro_bd)?;
    let status: String = r.try_get("status").map_err(erro_bd)?;
    Ok(Venda::reconstruir(
        r.try_get("id").map_err(erro_bd)?,
        r.try_get("data_compra").map_err(erro_bd)?,
        Dinheiro::de_centavos(r.try_get("valor_centavos").map_err(erro_bd)?)?,
        decodificar(&tipo)?,
        decodificar(&forma)?,
        decodificar(&status)?,
        r.try_get("cliente_id").map_err(erro_bd)?,
        r.try_get("evento_id").map_err(erro_bd)?,
        r.try_get("ingresso_id").map_err(erro_bd)?,
        r.try_get("assento_id").map_err(erro_bd)?,
    ))
}

pub struct RepositorioVendasPg {
    pool: PgPool,
}

impl RepositorioVendasPg {
    pub fn new(pool: PgPool) -> Self {
        RepositorioVendasPg { pool }
    }
}

#[async_trait]
impl RepositorioVendas for RepositorioVendasPg {
    /// Transação: cada UPDATE só altera a linha se a condição ainda for verdadeira no momento
    /// da gravação. Se alguma não alterar nada, a função retorna o erro e a transação é
    /// desfeita automaticamente (rollback) — é isso que impede a venda duplicada.
    async fn registrar(&self, nova: &NovaVenda) -> Resultado<Venda> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;

        if let Some(assento_id) = nova.assento_id {
            let r = sqlx::query("UPDATE assentos SET situacao = 'OCUPADO' WHERE id = $1 AND situacao = 'LIVRE'")
                .bind(assento_id)
                .execute(&mut *tx)
                .await
                .map_err(erro_bd)?;
            if r.rows_affected() == 0 {
                return Err(Erro::regra("assento já vendido"));
            }
        }
        let r = sqlx::query("UPDATE ingressos SET disponivel = disponivel - 1 WHERE id = $1 AND disponivel > 0")
            .bind(nova.ingresso_id)
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("categoria esgotada"));
        }
        let r = sqlx::query(
            "UPDATE eventos SET total_vendido = total_vendido + 1 WHERE id = $1 AND total_vendido < capacidade",
        )
        .bind(nova.evento_id)
        .execute(&mut *tx)
        .await
        .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("evento lotado"));
        }

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO vendas (data_compra, valor_centavos, tipo, forma_pagamento, status, cliente_id,
                                 ingresso_id, assento_id)
             VALUES ($1, $2, $3, $4, 'CONFIRMADA', $5, $6, $7) RETURNING id",
        )
        .bind(nova.data_compra)
        .bind(nova.valor.centavos())
        .bind(nova.tipo.codigo())
        .bind(nova.forma_pagamento.codigo())
        .bind(nova.cliente_id)
        .bind(nova.ingresso_id)
        .bind(nova.assento_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(erro_bd)?;

        tx.commit().await.map_err(erro_bd)?;
        Ok(Venda::registrada(id, nova))
    }

    async fn cancelar(&self, venda: &Venda) -> Resultado<()> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        let r = sqlx::query("UPDATE vendas SET status = 'CANCELADA' WHERE id = $1 AND status = 'CONFIRMADA'")
            .bind(venda.id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        if r.rows_affected() == 0 {
            return Err(Erro::regra("a compra não está mais confirmada"));
        }
        if let Some(assento_id) = venda.assento_id() {
            sqlx::query("UPDATE assentos SET situacao = 'LIVRE' WHERE id = $1")
                .bind(assento_id)
                .execute(&mut *tx)
                .await
                .map_err(erro_bd)?;
        }
        sqlx::query("UPDATE ingressos SET disponivel = disponivel + 1 WHERE id = $1")
            .bind(venda.ingresso_id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        sqlx::query("UPDATE eventos SET total_vendido = total_vendido - 1 WHERE id = $1")
            .bind(venda.evento_id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)
    }

    async fn remarcar(&self, venda: &Venda, novo_assento_id: i64) -> Resultado<()> {
        let mut tx = self.pool.begin().await.map_err(erro_bd)?;
        let ocupou = sqlx::query("UPDATE assentos SET situacao = 'OCUPADO' WHERE id = $1 AND situacao = 'LIVRE'")
            .bind(novo_assento_id)
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        if ocupou.rows_affected() == 0 {
            return Err(Erro::regra("assento já vendido. Escolha outro"));
        }
        let trocou = sqlx::query(
            "UPDATE vendas SET assento_id = $2 WHERE id = $1 AND status = 'CONFIRMADA' AND assento_id = $3",
        )
        .bind(venda.id())
        .bind(novo_assento_id)
        .bind(venda.assento_id())
        .execute(&mut *tx)
        .await
        .map_err(erro_bd)?;
        if trocou.rows_affected() == 0 {
            return Err(Erro::regra("a compra não está mais confirmada"));
        }
        sqlx::query("UPDATE assentos SET situacao = 'LIVRE' WHERE id = $1")
            .bind(venda.assento_id())
            .execute(&mut *tx)
            .await
            .map_err(erro_bd)?;
        tx.commit().await.map_err(erro_bd)
    }

    async fn marcar_utilizada(&self, venda_id: i64) -> Resultado<bool> {
        let r = sqlx::query("UPDATE vendas SET status = 'UTILIZADA' WHERE id = $1 AND status = 'CONFIRMADA'")
            .bind(venda_id)
            .execute(&self.pool)
            .await
            .map_err(erro_bd)?;
        Ok(r.rows_affected() == 1)
    }

    async fn buscar(&self, id: i64) -> Resultado<Option<Venda>> {
        let sql = format!("{SELECT_VENDA} WHERE v.id = $1");
        let linha = sqlx::query(&sql).bind(id).fetch_optional(&self.pool).await.map_err(erro_bd)?;
        linha.as_ref().map(ler_venda).transpose()
    }

    async fn listar_do_evento(&self, evento_id: i64) -> Resultado<Vec<Venda>> {
        let sql = format!("{SELECT_VENDA} WHERE i.evento_id = $1 ORDER BY v.id");
        let linhas = sqlx::query(&sql).bind(evento_id).fetch_all(&self.pool).await.map_err(erro_bd)?;
        linhas.iter().map(ler_venda).collect()
    }

    async fn listar_detalhadas_do_evento(&self, evento_id: i64) -> Resultado<Vec<VendaDetalhada>> {
        let linhas = sqlx::query(
            "SELECT v.id, v.data_compra, v.valor_centavos, v.tipo, v.forma_pagamento, v.status,
                    v.cliente_id, i.evento_id, v.ingresso_id, v.assento_id,
                    u.nome AS cliente, i.categoria, a.fileira, a.numero
             FROM vendas v
             JOIN ingressos i ON i.id = v.ingresso_id
             JOIN usuarios u ON u.id = v.cliente_id
             LEFT JOIN assentos a ON a.id = v.assento_id
             WHERE i.evento_id = $1
             ORDER BY v.id DESC",
        )
        .bind(evento_id)
        .fetch_all(&self.pool)
        .await
        .map_err(erro_bd)?;
        linhas
            .iter()
            .map(|r| {
                let fileira: Option<String> = r.try_get("fileira").map_err(erro_bd)?;
                let numero: Option<i32> = r.try_get("numero").map_err(erro_bd)?;
                Ok(VendaDetalhada {
                    venda: ler_venda(r)?,
                    cliente: r.try_get("cliente").map_err(erro_bd)?,
                    categoria: r.try_get("categoria").map_err(erro_bd)?,
                    assento: fileira.zip(numero).map(|(f, n)| format!("{f}{n}")),
                })
            })
            .collect()
    }

    async fn historico_do_cliente(&self, cliente_id: i64) -> Resultado<Vec<ItemHistorico>> {
        let linhas = sqlx::query(
            "SELECT v.id, v.data_compra, v.valor_centavos, v.tipo, v.forma_pagamento, v.status,
                    v.cliente_id, i.evento_id, v.ingresso_id, v.assento_id,
                    e.nome AS evento, e.data AS data_evento, e.horario, e.local, e.permite_cancelamento,
                    i.categoria, a.fileira, a.numero
             FROM vendas v
             JOIN ingressos i ON i.id = v.ingresso_id
             JOIN eventos e ON e.id = i.evento_id
             LEFT JOIN assentos a ON a.id = v.assento_id
             WHERE v.cliente_id = $1
             ORDER BY e.data DESC, v.id DESC",
        )
        .bind(cliente_id)
        .fetch_all(&self.pool)
        .await
        .map_err(erro_bd)?;

        linhas
            .iter()
            .map(|r| {
                let fileira: Option<String> = r.try_get("fileira").map_err(erro_bd)?;
                let numero: Option<i32> = r.try_get("numero").map_err(erro_bd)?;
                Ok(ItemHistorico {
                    venda: ler_venda(r)?,
                    evento: r.try_get("evento").map_err(erro_bd)?,
                    data_evento: r.try_get("data_evento").map_err(erro_bd)?,
                    horario: r.try_get("horario").map_err(erro_bd)?,
                    local: r.try_get("local").map_err(erro_bd)?,
                    categoria: r.try_get("categoria").map_err(erro_bd)?,
                    assento: fileira.zip(numero).map(|(f, n)| format!("{f}{n}")),
                    permite_cancelamento: r.try_get("permite_cancelamento").map_err(erro_bd)?,
                })
            })
            .collect()
    }
}
