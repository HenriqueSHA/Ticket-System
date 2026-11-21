//! Dados de demonstração para um banco vazio.
//!
//! Tudo é criado pelos mesmos casos de uso da interface, então passa por todas as validações.
//! As datas são relativas a hoje, para os eventos estarem sempre no futuro.

use chrono::{Days, NaiveTime};
use dominio::Resultado;
use dominio::entidades::*;
use dominio::valores::Dinheiro;

use crate::{Aplicacao, CadastroCliente, CadastroOrganizador, Pedido};

/// Logins de demonstração: (perfil, e-mail, senha).
pub const LOGINS: [(&str, &str, &str); 5] = [
    ("SuperAdmin", "admin@ingressos.com", "admin123"),
    ("Organizador", "contato@soleventos.com", "sol12345"),
    ("Organizador", "arena@cultural.com", "arena123"),
    ("Cliente", "ana@email.com", "ana12345"),
    ("Cliente", "bruno@email.com", "bruno123"),
];

fn reais(valor: i64) -> Dinheiro {
    Dinheiro::de_centavos(valor * 100).expect("valor fixo válido")
}

fn hora(h: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, 0, 0).expect("horário fixo válido")
}

/// Em produção (sem dados de demonstração): cria só o SuperAdmin informado, se o banco estiver
/// vazio. Devolve `true` se criou.
pub async fn criar_superadmin_se_vazio(
    app: &Aplicacao,
    nome: &str,
    cpf: &str,
    email: &str,
    senha: &str,
) -> Resultado<bool> {
    if app.usuarios.contar().await? > 0 {
        return Ok(false);
    }
    let admin = app.fabrica.criar_superadmin(nome, cpf, email, senha)?;
    app.usuarios.inserir(&admin).await?;
    Ok(true)
}

/// Popula o banco se não houver nenhum usuário. Devolve `true` se populou.
pub async fn popular_se_vazio(app: &Aplicacao) -> Resultado<bool> {
    if app.usuarios.contar().await? > 0 {
        return Ok(false);
    }
    let daqui = |dias: u64| app.hoje().checked_add_days(Days::new(dias)).expect("data válida");

    // O SuperAdmin é criado na instalação (não existe tela para criar outro).
    let admin = app.fabrica.criar_superadmin("Administrador", "123.456.789-09", LOGINS[0].1, LOGINS[0].2)?;
    app.usuarios.inserir(&admin).await?;
    let s_admin = app.autenticacao.entrar(LOGINS[0].1, LOGINS[0].2).await?;

    for (razao, cnpj, contato, (_, email, senha)) in [
        ("Sol Eventos Ltda", "11.222.333/0001-81", "(71) 3333-1000", LOGINS[1]),
        ("Arena Cultural Produções", "45.723.174/0001-10", "(71) 3333-2000", LOGINS[2]),
    ] {
        let dados = CadastroOrganizador {
            razao_social: razao.into(),
            cnpj: cnpj.into(),
            contato: contato.into(),
            email: email.into(),
            senha: senha.into(),
        };
        app.administracao.cadastrar_organizador(&s_admin, &dados).await?;
    }
    let s_sol = app.autenticacao.entrar(LOGINS[1].1, LOGINS[1].2).await?;
    let s_arena = app.autenticacao.entrar(LOGINS[2].1, LOGINS[2].2).await?;

    let eventos = [
        (
            &s_sol,
            "Show de Rock — Banda Trovão",
            "Turnê de 20 anos da banda",
            "Arena Salvador",
            20,
            21,
            160,
            true,
            vec![
                NovaCategoria::sem_assento("Pista", reais(120), 100)?,
                NovaCategoria::com_assento("Cadeira", reais(180), 5, 8)?,
                NovaCategoria::com_assento("Camarote", reais(350), 2, 10)?,
            ],
        ),
        (
            &s_sol,
            "Noite do Stand-up",
            "Quatro comediantes em uma noite",
            "Teatro Castro Alves",
            35,
            20,
            60,
            false,
            vec![NovaCategoria::com_assento("Plateia", reais(80), 6, 10)?],
        ),
        (
            &s_arena,
            "Festival de Jazz de Verão",
            "Dois palcos ao ar livre",
            "Pelourinho",
            50,
            18,
            300,
            true,
            vec![
                NovaCategoria::sem_assento("Pista", reais(90), 250)?,
                NovaCategoria::sem_assento("VIP", reais(200), 50)?,
            ],
        ),
    ];
    let mut criados = Vec::new();
    for (sessao, nome, descricao, local, dias, h, capacidade, cancela, categorias) in eventos {
        let dados = DadosEvento {
            nome: nome.into(),
            descricao: descricao.into(),
            local: local.into(),
            data: daqui(dias),
            horario: hora(h),
            capacidade,
            permite_cancelamento: cancela,
        };
        let mut builder = app.eventos.iniciar_cadastro(sessao, dados).await?;
        for c in categorias {
            builder.adicionar_categoria(c)?;
        }
        criados.push(app.eventos.cadastrar(sessao, &builder.construir()?).await?);
    }

    for (nome, cpf, (_, email, senha)) in
        [("Ana Souza", "529.982.247-25", LOGINS[3]), ("Bruno Lima", "111.444.777-35", LOGINS[4])]
    {
        let dados = CadastroCliente { nome: nome.into(), cpf: cpf.into(), email: email.into(), senha: senha.into() };
        app.autenticacao.cadastrar_cliente(&dados).await?;
    }
    let ana = app.autenticacao.entrar(LOGINS[3].1, LOGINS[3].2).await?;
    let bruno = app.autenticacao.entrar(LOGINS[4].1, LOGINS[4].2).await?;

    // Algumas vendas para o mapa de assentos e o relatório não começarem vazios.
    let show = app.eventos.categorias(criados[0].id()).await?;
    let standup = app.eventos.categorias(criados[1].id()).await?;
    let compras = [
        (&ana, &criados[0], &show[1], TipoIngresso::Inteira, Some(("A", 4)), FormaPagamento::Pix),
        (&ana, &criados[0], &show[1], TipoIngresso::Meia, Some(("A", 5)), FormaPagamento::Pix),
        (&bruno, &criados[0], &show[0], TipoIngresso::Meia, None, FormaPagamento::Debito),
        (&bruno, &criados[0], &show[2], TipoIngresso::Inteira, Some(("B", 3)), FormaPagamento::Credito),
        (&bruno, &criados[1], &standup[0], TipoIngresso::Inteira, Some(("C", 6)), FormaPagamento::Credito),
    ];
    for (cliente, evento, ingresso, tipo, assento, forma_pagamento) in compras {
        let pedido = Pedido {
            evento_id: evento.id(),
            ingresso_id: ingresso.id(),
            tipo,
            assento: assento.map(|(f, n)| (f.to_string(), n)),
            forma_pagamento,
        };
        app.vendas.confirmar(cliente, &pedido).await?;
    }
    Ok(true)
}
