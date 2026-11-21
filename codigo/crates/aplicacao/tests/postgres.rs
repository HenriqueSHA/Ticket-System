//! Teste de integração com o **PostgreSQL real**.
//!
//! Cria um banco temporário, aplica as migrações, roda os cenários principais pelos
//! repositórios PostgreSQL e apaga o banco no final. Precisa do container rodando
//! (`docker compose up -d`), por isso só executa com:
//!
//! ```bash
//! cargo test -- --ignored
//! ```

use std::sync::Arc;

use aplicacao::*;
use chrono::NaiveDate;
use dominio::Erro;
use dominio::entidades::*;
use infraestrutura::postgres::BancoPostgres;
use infraestrutura::relogio::RelogioFixo;
use infraestrutura::seguranca::HashArgon2;
use sqlx::Connection;

const SERVIDOR: &str = "postgres://ingressos:ingressos@localhost:5433";

#[tokio::test]
#[ignore = "precisa do PostgreSQL (docker compose up -d)"]
async fn cenarios_no_postgres() {
    let servidor = std::env::var("TESTE_POSTGRES").unwrap_or_else(|_| SERVIDOR.to_string());
    let nome_banco = format!("ingressos_teste_{}", std::process::id());
    let mut admin = sqlx::PgConnection::connect(&format!("{servidor}/postgres")).await.expect("PostgreSQL no ar?");
    sqlx::query(&format!("CREATE DATABASE {nome_banco}")).execute(&mut admin).await.unwrap();

    let resultado = tokio::spawn(executar(format!("{servidor}/{nome_banco}"))).await;

    sqlx::query(&format!("DROP DATABASE {nome_banco} WITH (FORCE)")).execute(&mut admin).await.unwrap();
    resultado.expect("cenários no PostgreSQL falharam");
}

async fn executar(url: String) {
    let banco = BancoPostgres::conectar(&url).await.unwrap();
    banco.migrar().await.unwrap();
    let agora = NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().and_hms_opt(12, 0, 0).unwrap();
    let app = Aplicacao::nova(Dependencias {
        usuarios: Arc::new(banco.usuarios()),
        organizadores: Arc::new(banco.organizadores()),
        eventos: Arc::new(banco.eventos()),
        vendas: Arc::new(banco.vendas()),
        hash: Arc::new(HashArgon2),
        relogio: Arc::new(RelogioFixo::em(agora)),
    });
    assert!(dados_exemplo::popular_se_vazio(&app).await.unwrap());
    assert!(!dados_exemplo::popular_se_vazio(&app).await.unwrap()); // não duplica

    let ana = app.autenticacao.entrar("ana@email.com", "ana12345").await.unwrap();
    let bruno = app.autenticacao.entrar("bruno@email.com", "bruno123").await.unwrap();
    let (show, cats) = app.eventos.listar_disponiveis().await.unwrap().remove(0);
    let cadeira = cats.iter().find(|c| c.categoria() == "Cadeira").unwrap();
    let pedido = |fileira: &str, numero| Pedido {
        evento_id: show.id(),
        ingresso_id: cadeira.id(),
        tipo: TipoIngresso::Meia,
        assento: Some((fileira.to_string(), numero)),
        forma_pagamento: FormaPagamento::Pix,
    };

    // Teste 1 — compra normal (meia-entrada, assento B2)
    let (venda, _) = app.vendas.confirmar(&ana, &pedido("B", 2)).await.unwrap();
    assert_eq!(venda.valor().to_string(), "R$ 90,00");
    let mapa = app.eventos.mapa(cadeira).await.unwrap();
    assert_eq!(mapa.situacao(1, 1), Some(Situacao::Ocupado));

    // Teste 3 — o mesmo assento não pode ser vendido de novo (transação + índice único)
    let r = app.vendas.confirmar(&bruno, &pedido("B", 2)).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(_))));

    // Teste 2 — cancelamento libera o assento para outra pessoa
    app.vendas.cancelar(&ana, venda.id()).await.unwrap();
    app.vendas.confirmar(&bruno, &pedido("B", 2)).await.unwrap();

    // relatório: 4 vendas da demonstração + 1 nova; 1 cancelada
    let sol = app.autenticacao.entrar("contato@soleventos.com", "sol12345").await.unwrap();
    let r = app.relatorios.relatorio(&sol, show.id()).await.unwrap();
    assert_eq!((r.vendidos, r.cancelados), (5, 1));
    assert_eq!(r.receita.to_string(), "R$ 770,00");

    // gestão: quantidade não fica abaixo do vendido; preço novo vale na próxima venda
    let pista = cats.iter().find(|c| c.categoria() == "Pista").unwrap();
    let preco = dominio::valores::Dinheiro::de_texto("99,90").unwrap();
    let r = app.eventos.alterar_categoria(&sol, show.id(), pista.id(), "Pista", preco, Some(0)).await;
    assert!(r.is_err());
    let alterada = app.eventos.alterar_categoria(&sol, show.id(), pista.id(), "Pista", preco, Some(80)).await.unwrap();
    assert_eq!((alterada.quantidade(), alterada.disponivel()), (80, 79));
    let gravada = app.eventos.categorias(show.id()).await.unwrap().into_iter().find(|c| c.id() == pista.id()).unwrap();
    assert_eq!((gravada.quantidade(), gravada.disponivel(), gravada.preco()), (80, 79, preco));
    let nova = app
        .eventos
        .adicionar_categoria(&sol, show.id(), NovaCategoria::com_assento("Mezanino", preco, 2, 5).unwrap())
        .await
        .unwrap();
    assert_eq!(app.eventos.mapa(&nova).await.unwrap().total(), 10);
    app.eventos.remover_categoria(&sol, show.id(), nova.id()).await.unwrap();
    assert!(app.eventos.remover(&sol, show.id()).await.is_err()); // tem vendas
    let vendas = app.relatorios.vendas_do_evento(&sol, show.id()).await.unwrap();
    assert_eq!(vendas.len(), 6);

    // unicidade garantida também pelo banco
    let dup = CadastroCliente {
        nome: "Outra".into(),
        cpf: "987.654.321-00".into(),
        email: "ana@email.com".into(),
        senha: "123456".into(),
    };
    assert!(matches!(app.autenticacao.cadastrar_cliente(&dup).await, Err(Erro::RegraNegocio(_))));
}
