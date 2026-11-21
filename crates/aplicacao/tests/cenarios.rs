//! Testes de cenário (seção 15 do relatório): normal, alternativo e excepcional.
//!
//! Usam os repositórios **em memória** e um relógio fixo em 21/11/2026 12:00, então não
//! precisam do PostgreSQL e dão sempre o mesmo resultado. Os dados de partida são os de
//! demonstração (`dados_exemplo`):
//!   - "Show de Rock" (+20 dias, permite cancelamento): Pista R$120 (sem assento),
//!     Cadeira R$180 (5×8), Camarote R$350 (2×10)
//!   - "Noite do Stand-up" (+35 dias, NÃO permite cancelamento): Plateia R$80 (6×10)
//!   - vendas: Ana → Cadeira A4 (inteira) e A5 (meia); Bruno → Pista (meia), Camarote B3, Plateia C6

use std::sync::Arc;

use aplicacao::*;
use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};
use dominio::Erro;
use dominio::entidades::*;
use dominio::valores::Dinheiro;
use infraestrutura::memoria::RepositoriosMemoria;
use infraestrutura::relogio::RelogioFixo;
use infraestrutura::seguranca::HashArgon2;

struct Cenario {
    app: Aplicacao,
    relogio: Arc<RelogioFixo>,
}

fn agora_teste() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().and_hms_opt(12, 0, 0).unwrap()
}

async fn cenario() -> Cenario {
    let repos = RepositoriosMemoria::novo();
    let relogio = Arc::new(RelogioFixo::em(agora_teste()));
    let app = Aplicacao::nova(Dependencias {
        usuarios: Arc::new(repos.clone()),
        organizadores: Arc::new(repos.clone()),
        eventos: Arc::new(repos.clone()),
        vendas: Arc::new(repos),
        hash: Arc::new(HashArgon2),
        relogio: relogio.clone(),
    });
    assert!(dados_exemplo::popular_se_vazio(&app).await.unwrap());
    Cenario { app, relogio }
}

impl Cenario {
    async fn login(&self, email: &str, senha: &str) -> Sessao {
        self.app.autenticacao.entrar(email, senha).await.expect("login de demonstração")
    }
    async fn ana(&self) -> Sessao {
        self.login("ana@email.com", "ana12345").await
    }
    async fn bruno(&self) -> Sessao {
        self.login("bruno@email.com", "bruno123").await
    }
    async fn sol(&self) -> Sessao {
        self.login("contato@soleventos.com", "sol12345").await
    }
    async fn admin(&self) -> Sessao {
        self.login("admin@ingressos.com", "admin123").await
    }
    /// Evento e categoria pelo nome (os ids não importam para o teste).
    async fn categoria(&self, evento: &str, categoria: &str) -> (Evento, Ingresso) {
        for (e, cats) in self.app.eventos.listar_disponiveis().await.unwrap() {
            if e.nome().starts_with(evento) {
                let c = cats.into_iter().find(|c| c.categoria() == categoria).unwrap();
                return (e, c);
            }
        }
        panic!("evento {evento} não encontrado");
    }
    async fn disponivel(&self, evento: &Evento, ingresso: &Ingresso) -> i32 {
        let cats = self.app.eventos.categorias(evento.id()).await.unwrap();
        cats.iter().find(|c| c.id() == ingresso.id()).unwrap().disponivel()
    }
}

fn pedido(e: &Evento, i: &Ingresso, tipo: TipoIngresso, assento: Option<(&str, i32)>) -> Pedido {
    Pedido {
        evento_id: e.id(),
        ingresso_id: i.id(),
        tipo,
        assento: assento.map(|(f, n)| (f.to_string(), n)),
        forma_pagamento: FormaPagamento::Pix,
    }
}

// ======================= TESTE 1 — CENÁRIO NORMAL =======================

/// Cliente compra Cadeira B2, inteira, no PIX.
/// Esperado: venda CONFIRMADA de R$ 180,00; assento B2 passa a ocupado; disponível 38 → 37.
#[tokio::test]
async fn teste1_normal_compra_com_assento_marcado() {
    let c = cenario().await;
    let ana = c.ana().await;
    let (show, cadeira) = c.categoria("Show", "Cadeira").await;
    assert_eq!(c.disponivel(&show, &cadeira).await, 38);

    let (venda, resumo) = c.app.vendas.confirmar(&ana, &pedido(&show, &cadeira, TipoIngresso::Inteira, Some(("B", 2)))).await.unwrap();

    assert_eq!(venda.status(), StatusVenda::Confirmada);
    assert_eq!(venda.valor().to_string(), "R$ 180,00");
    assert_eq!(resumo.assento.unwrap().rotulo(), "B2");
    assert_eq!(c.disponivel(&show, &cadeira).await, 37);
    let mapa = c.app.eventos.mapa(&cadeira).await.unwrap();
    assert_eq!(mapa.situacao(1, 1), Some(Situacao::Ocupado)); // fileira B, número 2
    assert_eq!(mapa.livres(), 37);
    let comprovante = c.app.vendas.comprovante(&ana, venda.id()).await.unwrap();
    assert_eq!(comprovante.assento.as_deref(), Some("B2"));
}

// ===================== TESTE 2 — CENÁRIO ALTERNATIVO =====================

/// Cliente compra Pista (sem assento) com meia-entrada e depois cancela.
/// Esperado: R$ 60,00 (50% de R$ 120,00); após cancelar, status CANCELADA e o ingresso volta.
#[tokio::test]
async fn teste2_alternativo_meia_entrada_sem_assento_e_cancelamento() {
    let c = cenario().await;
    let bruno = c.bruno().await;
    let (show, pista) = c.categoria("Show", "Pista").await;
    let antes = c.disponivel(&show, &pista).await;

    let (venda, resumo) = c.app.vendas.confirmar(&bruno, &pedido(&show, &pista, TipoIngresso::Meia, None)).await.unwrap();
    assert_eq!(venda.valor().to_string(), "R$ 60,00");
    assert!(resumo.assento.is_none());
    assert_eq!(c.disponivel(&show, &pista).await, antes - 1);

    let cancelada = c.app.vendas.cancelar(&bruno, venda.id()).await.unwrap();
    assert_eq!(cancelada.status(), StatusVenda::Cancelada);
    assert_eq!(c.disponivel(&show, &pista).await, antes);
    let relatorio = c.app.relatorios.relatorio(&c.sol().await, show.id()).await.unwrap();
    assert_eq!(relatorio.cancelados, 1);
}

/// Cancelar uma compra com assento libera o assento para outra pessoa.
#[tokio::test]
async fn teste2b_alternativo_cancelamento_libera_assento() {
    let c = cenario().await;
    let (ana, bruno) = (c.ana().await, c.bruno().await);
    let (show, cadeira) = c.categoria("Show", "Cadeira").await;
    let a4 = c.app.vendas.historico(&ana).await.unwrap().into_iter().find(|h| h.assento.as_deref() == Some("A4")).unwrap();

    c.app.vendas.cancelar(&ana, a4.venda.id()).await.unwrap();
    let (venda, _) = c.app.vendas.confirmar(&bruno, &pedido(&show, &cadeira, TipoIngresso::Inteira, Some(("A", 4)))).await.unwrap();
    assert_eq!(venda.cliente_id(), bruno.usuario_id);
}

// ===================== TESTE 3 — CENÁRIOS EXCEPCIONAIS =====================

/// Comprar assento já vendido (A4, da Ana). Esperado: erro e NADA gravado.
#[tokio::test]
async fn teste3_excepcional_assento_ja_vendido_nao_gera_venda_duplicada() {
    let c = cenario().await;
    let bruno = c.bruno().await;
    let (show, cadeira) = c.categoria("Show", "Cadeira").await;
    let antes = c.disponivel(&show, &cadeira).await;
    let historico_antes = c.app.vendas.historico(&bruno).await.unwrap().len();

    let r = c.app.vendas.confirmar(&bruno, &pedido(&show, &cadeira, TipoIngresso::Inteira, Some(("A", 4)))).await;

    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("já vendido")));
    assert_eq!(c.disponivel(&show, &cadeira).await, antes);
    assert_eq!(c.app.vendas.historico(&bruno).await.unwrap().len(), historico_antes);
}

#[tokio::test]
async fn teste3_excepcional_assento_inexistente_e_categoria_de_outro_evento() {
    let c = cenario().await;
    let ana = c.ana().await;
    let (show, cadeira) = c.categoria("Show", "Cadeira").await;
    let (_, plateia) = c.categoria("Noite", "Plateia").await;

    let r = c.app.vendas.confirmar(&ana, &pedido(&show, &cadeira, TipoIngresso::Inteira, Some(("Z", 99)))).await;
    assert!(matches!(r, Err(Erro::NaoEncontrado(_))));
    let r = c.app.vendas.confirmar(&ana, &pedido(&show, &plateia, TipoIngresso::Inteira, Some(("A", 1)))).await;
    assert!(matches!(r, Err(Erro::NaoEncontrado(_))));
    let r = c.app.vendas.confirmar(&ana, &pedido(&show, &cadeira, TipoIngresso::Inteira, None)).await;
    assert!(matches!(r, Err(Erro::Validacao(_))));
}

/// Inclui os dois CPFs digitados no teste manual que motivou esta versão.
#[tokio::test]
async fn teste3_excepcional_cadastro_de_cliente_invalido() {
    let c = cenario().await;
    let auth = &c.app.autenticacao;
    let cadastro = |nome: &str, cpf: &str, email: &str, senha: &str| CadastroCliente {
        nome: nome.into(),
        cpf: cpf.into(),
        email: email.into(),
        senha: senha.into(),
    };

    let r = auth.cadastrar_cliente(&cadastro("Henrique", "98998998943", "henrique@gmail.com", "123456")).await;
    assert!(matches!(r, Err(Erro::Validacao(ref m)) if m.contains("CPF")));
    let r = auth.cadastrar_cliente(&cadastro("Henrique", "12345678909 01", "henrique@gmail.com", "123456")).await;
    assert!(matches!(r, Err(Erro::Validacao(_))));
    let r = auth.cadastrar_cliente(&cadastro("", "987.654.321-00", "carla@email.com", "123456")).await;
    assert!(matches!(r, Err(Erro::Validacao(ref m)) if m.contains("nome")));
    let r = auth.cadastrar_cliente(&cadastro("Carla", "987.654.321-00", "carla.email.com", "123456")).await;
    assert!(matches!(r, Err(Erro::Validacao(_))));
    let r = auth.cadastrar_cliente(&cadastro("Carla", "987.654.321-00", "carla@email.com", "123")).await;
    assert!(matches!(r, Err(Erro::Validacao(_))));
    let r = auth.cadastrar_cliente(&cadastro("Outra Ana", "987.654.321-00", "ana@email.com", "123456")).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(_))));
    let r = auth.cadastrar_cliente(&cadastro("Outra", "123.456.789-09", "nova@email.com", "123456")).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(_)))); // CPF do administrador

    // cadastro válido e login em seguida (o fluxo que o usuário tentou)
    auth.cadastrar_cliente(&cadastro("Carla Dias", "987.654.321-00", "Carla@Email.com", "123456")).await.unwrap();
    let sessao = auth.entrar("carla@email.com", "123456").await.unwrap();
    assert_eq!(sessao.perfil, Perfil::Cliente);
}

#[tokio::test]
async fn teste3_excepcional_login_invalido_e_organizador_bloqueado() {
    let c = cenario().await;
    let auth = &c.app.autenticacao;
    assert!(matches!(auth.entrar("ana@email.com", "errada").await, Err(Erro::AcessoNegado(_))));
    assert!(matches!(auth.entrar("ninguem@email.com", "123456").await, Err(Erro::AcessoNegado(_))));

    let admin = c.admin().await;
    let sol_id = c.sol().await.organizador_id.unwrap();
    c.app.administracao.bloquear(&admin, sol_id).await.unwrap();
    assert!(matches!(auth.entrar("contato@soleventos.com", "sol12345").await, Err(Erro::AcessoNegado(_))));
    c.app.administracao.ativar(&admin, sol_id).await.unwrap();
    assert!(auth.entrar("contato@soleventos.com", "sol12345").await.is_ok());
}

#[tokio::test]
async fn teste3_excepcional_cadastro_de_evento_invalido() {
    let c = cenario().await;
    let sol = c.sol().await;
    let hoje = agora_teste().date();
    let dados = DadosEvento {
        nome: "Evento Teste".into(),
        descricao: String::new(),
        local: "Salvador".into(),
        data: hoje.checked_add_days(Days::new(10)).unwrap(),
        horario: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        capacidade: 100,
        permite_cancelamento: true,
    };
    let preco = Dinheiro::de_centavos(5000).unwrap();
    let eventos = &c.app.eventos;

    let passado = DadosEvento { data: hoje, ..dados.clone() };
    assert!(matches!(eventos.iniciar_cadastro(&sol, passado).await, Err(Erro::Validacao(_))));
    let sem_capacidade = DadosEvento { capacidade: 0, ..dados.clone() };
    assert!(matches!(eventos.iniciar_cadastro(&sol, sem_capacidade).await, Err(Erro::Validacao(_))));
    assert!(NovaCategoria::sem_assento("Pista", preco, 0).is_err());
    assert!(NovaCategoria::com_assento("Cadeira", preco, 27, 10).is_err());
    assert!(Dinheiro::de_texto("-10").is_err());

    let mut builder = eventos.iniciar_cadastro(&sol, dados.clone()).await.unwrap();
    assert!(matches!(
        builder.adicionar_categoria(NovaCategoria::sem_assento("Pista", preco, 101).unwrap()),
        Err(Erro::RegraNegocio(_))
    ));
    builder.adicionar_categoria(NovaCategoria::sem_assento("Pista", preco, 60).unwrap()).unwrap();
    builder.adicionar_categoria(NovaCategoria::com_assento("Cadeira", preco, 4, 10).unwrap()).unwrap();
    let evento = eventos.cadastrar(&sol, &builder.construir().unwrap()).await.unwrap();
    assert_eq!(eventos.categorias(evento.id()).await.unwrap().len(), 2);

    // cliente não cadastra evento
    assert!(matches!(eventos.iniciar_cadastro(&c.ana().await, dados).await, Err(Erro::AcessoNegado(_))));
}

#[tokio::test]
async fn teste3_excepcional_evento_lotado() {
    let c = cenario().await;
    let (sol, ana) = (c.sol().await, c.ana().await);
    let dados = DadosEvento {
        nome: "Sessão Intimista".into(),
        descricao: String::new(),
        local: "Café".into(),
        data: agora_teste().date().checked_add_days(Days::new(5)).unwrap(),
        horario: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        capacidade: 2,
        permite_cancelamento: true,
    };
    let mut b = c.app.eventos.iniciar_cadastro(&sol, dados).await.unwrap();
    b.adicionar_categoria(NovaCategoria::sem_assento("Única", Dinheiro::de_centavos(3000).unwrap(), 2).unwrap()).unwrap();
    let evento = c.app.eventos.cadastrar(&sol, &b.construir().unwrap()).await.unwrap();
    let cat = c.app.eventos.categorias(evento.id()).await.unwrap().remove(0);

    for _ in 0..2 {
        c.app.vendas.confirmar(&ana, &pedido(&evento, &cat, TipoIngresso::Inteira, None)).await.unwrap();
    }
    let r = c.app.vendas.confirmar(&ana, &pedido(&evento, &cat, TipoIngresso::Inteira, None)).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("lotado")));
    assert!(c.app.eventos.listar_disponiveis().await.unwrap().iter().all(|(e, _)| e.id() != evento.id()));
}

#[tokio::test]
async fn teste3_excepcional_cancelamentos_proibidos() {
    let c = cenario().await;
    let (ana, bruno) = (c.ana().await, c.bruno().await);
    let hist_bruno = c.app.vendas.historico(&bruno).await.unwrap();
    let hist_ana = c.app.vendas.historico(&ana).await.unwrap();
    let plateia = hist_bruno.iter().find(|h| h.categoria == "Plateia").unwrap().venda.id();
    let (a4, a5) = (
        hist_ana.iter().find(|h| h.assento.as_deref() == Some("A4")).unwrap().venda.id(),
        hist_ana.iter().find(|h| h.assento.as_deref() == Some("A5")).unwrap().venda.id(),
    );

    let r = c.app.vendas.cancelar(&bruno, plateia).await; // Stand-up não permite
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("não permite")));
    assert!(matches!(c.app.vendas.cancelar(&bruno, a4).await, Err(Erro::NaoEncontrado(_)))); // compra de outra pessoa
    c.app.vendas.cancelar(&ana, a4).await.unwrap();
    assert!(matches!(c.app.vendas.cancelar(&ana, a4).await, Err(Erro::RegraNegocio(_)))); // duas vezes

    c.relogio.avancar_dias(30); // o Show (+20 dias) já aconteceu
    let r = c.app.vendas.cancelar(&ana, a5).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("após a data")));
}

#[tokio::test]
async fn teste3_excepcional_controle_de_acesso() {
    let c = cenario().await;
    let (ana, sol, admin) = (c.ana().await, c.sol().await, c.admin().await);
    let arena = c.login("arena@cultural.com", "arena123").await;
    let (show, pista) = c.categoria("Show", "Pista").await;

    assert!(matches!(c.app.relatorios.relatorio(&ana, show.id()).await, Err(Erro::AcessoNegado(_))));
    assert!(matches!(c.app.administracao.listar(&ana).await, Err(Erro::AcessoNegado(_))));
    let r = c.app.vendas.confirmar(&sol, &pedido(&show, &pista, TipoIngresso::Inteira, None)).await;
    assert!(matches!(r, Err(Erro::AcessoNegado(_))));
    assert!(matches!(c.app.relatorios.relatorio(&arena, show.id()).await, Err(Erro::NaoEncontrado(_))));
    let r = c.app.administracao.remover(&admin, sol.organizador_id.unwrap()).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(_))));
}

// ===================== relatório e check-in =====================

/// Vendas do Show: Cadeira A4 inteira R$180, A5 meia R$90, Pista meia R$60, Camarote B3 R$350.
#[tokio::test]
async fn relatorio_e_registro_de_presenca() {
    let c = cenario().await;
    let sol = c.sol().await;
    let (show, _) = c.categoria("Show", "Pista").await;

    let r = c.app.relatorios.relatorio(&sol, show.id()).await.unwrap();
    assert_eq!((r.vendidos, r.inteiras, r.meias, r.cancelados, r.presentes), (4, 2, 2, 0, 0));
    assert_eq!(r.receita.to_string(), "R$ 680,00");
    assert_eq!(r.ocupacao_formatada(), "2,5"); // 4 de 160

    let a4 = c.app.vendas.historico(&c.ana().await).await.unwrap().into_iter()
        .find(|h| h.assento.as_deref() == Some("A4")).unwrap().venda.id();
    assert_eq!(c.app.relatorios.registrar_presenca(&sol, a4).await.unwrap().status(), StatusVenda::Utilizada);
    let r2 = c.app.relatorios.registrar_presenca(&sol, a4).await;
    assert!(matches!(r2, Err(Erro::RegraNegocio(ref m)) if m.contains("já utilizado")));
    assert_eq!(c.app.relatorios.relatorio(&sol, show.id()).await.unwrap().presentes, 1);

    let arena = c.login("arena@cultural.com", "arena123").await;
    assert!(matches!(c.app.relatorios.registrar_presenca(&arena, a4).await, Err(Erro::NaoEncontrado(_))));
}

#[tokio::test]
async fn administracao_de_organizadores() {
    let c = cenario().await;
    let admin = c.admin().await;
    let dados = CadastroOrganizador {
        razao_social: "Nova Produtora".into(),
        cnpj: "33.444.555/0001-81".into(),
        contato: "(71) 9999-0000".into(),
        email: "nova@produtora.com".into(),
        senha: "nova1234".into(),
    };
    let org = c.app.administracao.cadastrar_organizador(&admin, &dados).await.unwrap();
    assert!(c.login("nova@produtora.com", "nova1234").await.organizador_id.is_some());

    let editado = c.app.administracao.editar(&admin, org.id(), "Nova Produtora S.A.", "(71) 9999-1111").await.unwrap();
    assert_eq!(editado.razao_social(), "Nova Produtora S.A.");
    assert!(c.app.administracao.editar(&admin, org.id(), "", "x").await.is_err());

    let cnpj_invalido = CadastroOrganizador { cnpj: "11.111.111/1111-11".into(), ..dados.clone() };
    assert!(matches!(c.app.administracao.cadastrar_organizador(&admin, &cnpj_invalido).await, Err(Erro::Validacao(_))));

    c.app.administracao.remover(&admin, org.id()).await.unwrap(); // sem vendas: pode remover
    assert!(c.app.autenticacao.entrar("nova@produtora.com", "nova1234").await.is_err());
}

/// Empresa cria a própria conta de organizador, entra, cadastra um evento e um cliente compra.
#[tokio::test]
async fn organizador_cria_conta_e_vende_ingressos() {
    let c = cenario().await;
    let dados = CadastroOrganizador {
        razao_social: "Produtora Nova Onda".into(),
        cnpj: "33.444.555/0001-81".into(),
        contato: "(71) 98888-7777".into(),
        email: "contato@novaonda.com".into(),
        senha: "onda1234".into(),
    };
    let org = c.app.autenticacao.cadastrar_organizador(&dados).await.unwrap();
    assert!(org.esta_ativo());
    // mesmo CNPJ ou e-mail de novo: recusado
    assert!(matches!(c.app.autenticacao.cadastrar_organizador(&dados).await, Err(Erro::RegraNegocio(_))));
    let invalido = CadastroOrganizador { cnpj: "11.222.333/0001-00".into(), email: "x@y.com".into(), ..dados.clone() };
    assert!(matches!(c.app.autenticacao.cadastrar_organizador(&invalido).await, Err(Erro::Validacao(_))));

    let sessao = c.login("contato@novaonda.com", "onda1234").await;
    assert_eq!(sessao.perfil, Perfil::Organizador);
    let evento = DadosEvento {
        nome: "Luau na Praia".into(),
        descricao: String::new(),
        local: "Praia do Forte".into(),
        data: agora_teste().date().checked_add_days(Days::new(15)).unwrap(),
        horario: NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
        capacidade: 200,
        permite_cancelamento: true,
    };
    let mut b = c.app.eventos.iniciar_cadastro(&sessao, evento).await.unwrap();
    b.adicionar_categoria(NovaCategoria::sem_assento("Areia", Dinheiro::de_texto("45,00").unwrap(), 150).unwrap()).unwrap();
    b.adicionar_categoria(NovaCategoria::com_assento("Mesa", Dinheiro::de_texto("90").unwrap(), 5, 10).unwrap()).unwrap();
    let criado = c.app.eventos.cadastrar(&sessao, &b.construir().unwrap()).await.unwrap();

    let (e, areia) = c.categoria("Luau", "Areia").await;
    assert_eq!(e.id(), criado.id());
    assert_eq!(areia.disponivel(), 150);
    let (venda, _) = c.app.vendas.confirmar(&c.ana().await, &pedido(&e, &areia, TipoIngresso::Meia, None)).await.unwrap();
    assert_eq!(venda.valor().to_string(), "R$ 22,50");
    assert_eq!(c.app.relatorios.relatorio(&sessao, e.id()).await.unwrap().vendidos, 1);
}

// ===================== gestão do evento pelo organizador =====================

/// Organizador ajusta preço e quantidade, adiciona e remove categorias e muda a capacidade.
#[tokio::test]
async fn organizador_gerencia_categorias_e_capacidade() {
    let c = cenario().await;
    let sol = c.sol().await;
    let ev = &c.app.eventos;
    let (show, pista) = c.categoria("Show", "Pista").await; // 100 ingressos, 1 vendido; capacidade 160 = 100 + 40 + 20

    // preço e quantidade da Pista
    let reais = |t: &str| Dinheiro::de_texto(t).unwrap();
    let nova = ev.alterar_categoria(&sol, show.id(), pista.id(), "Pista Premium", reais("130,00"), Some(90)).await.unwrap();
    assert_eq!((nova.categoria(), nova.quantidade(), nova.disponivel()), ("Pista Premium", 90, 89));
    // não pode ficar abaixo do vendido nem passar da capacidade (90 + 40 + 20 = 150; vagas = 160 - 60 = 100)
    let r = ev.alterar_categoria(&sol, show.id(), pista.id(), "Pista", reais("130"), Some(0)).await;
    assert!(matches!(r, Err(Erro::Validacao(_))));
    let r = ev.alterar_categoria(&sol, show.id(), pista.id(), "Pista", reais("130"), Some(101)).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("capacidade")));
    // a compra seguinte usa o novo preço
    let (_, pista) = c.categoria("Show", "Pista Premium").await;
    let (venda, _) = c.app.vendas.confirmar(&c.ana().await, &pedido(&show, &pista, TipoIngresso::Inteira, None)).await.unwrap();
    assert_eq!(venda.valor().to_string(), "R$ 130,00");

    // nova categoria cabe nas 10 vagas que sobraram (90 + 40 + 20 = 150)
    let r = ev.adicionar_categoria(&sol, show.id(), NovaCategoria::sem_assento("Backstage", reais("500"), 11).unwrap()).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(_))));
    let back = ev.adicionar_categoria(&sol, show.id(), NovaCategoria::com_assento("Backstage", reais("500"), 2, 5).unwrap()).await.unwrap();
    assert_eq!(c.app.eventos.mapa(&back).await.unwrap().total(), 10);

    // capacidade: não pode ficar abaixo da soma das categorias (160); pode aumentar
    let mut dados = show.dados().clone();
    dados.capacidade = 150;
    assert!(ev.alterar(&sol, show.id(), dados.clone(), StatusEvento::Aberto).await.is_err());
    dados.capacidade = 200;
    assert_eq!(ev.alterar(&sol, show.id(), dados, StatusEvento::Aberto).await.unwrap().capacidade(), 200);

    // remover: categoria com vendas não; sem vendas sim
    let cadeira = ev.categorias(show.id()).await.unwrap().into_iter().find(|i| i.categoria() == "Cadeira").unwrap();
    assert!(matches!(ev.remover_categoria(&sol, show.id(), cadeira.id()).await, Err(Erro::RegraNegocio(_))));
    ev.remover_categoria(&sol, show.id(), back.id()).await.unwrap();
    assert_eq!(ev.categorias(show.id()).await.unwrap().len(), 3);

    // outro organizador não mexe no evento
    let arena = c.login("arena@cultural.com", "arena123").await;
    let r = ev.alterar_categoria(&arena, show.id(), cadeira.id(), "X", reais("1"), None).await;
    assert!(matches!(r, Err(Erro::NaoEncontrado(_))));
}

#[tokio::test]
async fn organizador_exclui_evento_sem_vendas_e_ve_lista_de_vendas() {
    let c = cenario().await;
    let sol = c.sol().await;
    let (show, _) = c.categoria("Show", "Pista").await;
    let r = c.app.eventos.remover(&sol, show.id()).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("já tem vendas")));

    let vendas = c.app.relatorios.vendas_do_evento(&sol, show.id()).await.unwrap();
    assert_eq!(vendas.len(), 4);
    assert!(vendas.iter().any(|v| v.cliente == "Ana Souza" && v.assento.as_deref() == Some("A4")));
    let arena = c.login("arena@cultural.com", "arena123").await;
    assert!(c.app.relatorios.vendas_do_evento(&arena, show.id()).await.is_err());

    // evento novo, sem vendas: pode excluir; e a última categoria não pode ser removida
    let dados = DadosEvento {
        nome: "Ensaio Aberto".into(),
        descricao: String::new(),
        local: "Sala 1".into(),
        data: agora_teste().date().checked_add_days(Days::new(3)).unwrap(),
        horario: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        capacidade: 30,
        permite_cancelamento: true,
    };
    let mut b = c.app.eventos.iniciar_cadastro(&sol, dados).await.unwrap();
    b.adicionar_categoria(NovaCategoria::sem_assento("Geral", Dinheiro::ZERO, 30).unwrap()).unwrap();
    let evento = c.app.eventos.cadastrar(&sol, &b.construir().unwrap()).await.unwrap();
    let unica = c.app.eventos.categorias(evento.id()).await.unwrap().remove(0);
    assert!(c.app.eventos.remover_categoria(&sol, evento.id(), unica.id()).await.is_err());
    c.app.eventos.remover(&sol, evento.id()).await.unwrap();
    assert!(c.app.eventos.gerenciar(&sol, evento.id()).await.is_err());
}

/// O e-mail é único no sistema inteiro: nem cliente nem organizador repetem e-mail já usado
/// (mesmo com letras maiúsculas/espaços). Organizador pode usar CNPJ alfanumérico.
#[tokio::test]
async fn email_unico_entre_perfis_e_cnpj_alfanumerico() {
    let c = cenario().await;
    let auth = &c.app.autenticacao;
    let cliente = |email: &str, cpf: &str| CadastroCliente { nome: "Henrique".into(), cpf: cpf.into(), email: email.into(), senha: "123456".into() };
    auth.cadastrar_cliente(&cliente("henrique@gmail.com", "987.654.321-00")).await.unwrap();
    for repetido in ["henrique@gmail.com", " HENRIQUE@Gmail.com "] {
        let r = auth.cadastrar_cliente(&cliente(repetido, "246.813.579-28")).await;
        assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("e-mail")), "{repetido}");
    }
    let org = |email: &str, cnpj: &str| CadastroOrganizador {
        razao_social: "Produtora Nova".into(),
        cnpj: cnpj.into(),
        contato: "71 9999-0000".into(),
        email: email.into(),
        senha: "123456".into(),
    };
    let r = auth.cadastrar_organizador(&org("henrique@gmail.com", "12.ABC.345/01DE-35")).await;
    assert!(matches!(r, Err(Erro::RegraNegocio(ref m)) if m.contains("e-mail")));
    let criado = auth.cadastrar_organizador(&org("produtora@nova.com", "12.abc.345/01de-35")).await.unwrap();
    assert_eq!(criado.cnpj().to_string(), "12.ABC.345/01DE-35");
    assert!(c.login("produtora@nova.com", "123456").await.organizador_id.is_some());
}
