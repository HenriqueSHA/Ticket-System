//! Testes da API por HTTP real: sobe o back numa porta livre (repositórios em memória, relógio
//! fixo em 21/11/2026 12:00) e faz as mesmas requisições que o front faz.

use std::sync::Arc;

use api::{EstadoApi, rotas};
use aplicacao::{Aplicacao, Dependencias, dados_exemplo};
use chrono::{Days, NaiveDate, NaiveTime};
use contrato::*;
use dominio::entidades::{FormaPagamento, Perfil, TipoIngresso};
use infraestrutura::memoria::RepositoriosMemoria;
use infraestrutura::relogio::RelogioFixo;
use infraestrutura::seguranca::HashArgon2;
use reqwest::{Client, Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

struct Api {
    base: String,
    http: Client,
}

async fn subir() -> Api {
    subir_com_validade(api::DURACAO_SESSAO).await
}

async fn subir_com_validade(validade: std::time::Duration) -> Api {
    let repos = RepositoriosMemoria::novo();
    let hoje = NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().and_hms_opt(12, 0, 0).unwrap();
    let app = Aplicacao::nova(Dependencias {
        usuarios: Arc::new(repos.clone()),
        organizadores: Arc::new(repos.clone()),
        eventos: Arc::new(repos.clone()),
        vendas: Arc::new(repos),
        hash: Arc::new(HashArgon2),
        relogio: Arc::new(RelogioFixo::em(hoje)),
    });
    dados_exemplo::popular_se_vazio(&app).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, rotas::criar(EstadoApi::com_validade(app, validade))).await.unwrap() });
    Api { base, http: Client::new() }
}

impl Api {
    /// Faz a requisição e devolve (código HTTP, corpo JSON bruto).
    async fn pedir<C: Serialize>(&self, metodo: Method, caminho: &str, token: Option<&str>, corpo: Option<&C>) -> (StatusCode, String) {
        let mut req = self.http.request(metodo, format!("{}{caminho}", self.base));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        if let Some(c) = corpo {
            req = req.json(c);
        }
        let resp = req.send().await.unwrap();
        (resp.status(), resp.text().await.unwrap())
    }

    async fn get<T: DeserializeOwned>(&self, caminho: &str, token: Option<&str>) -> T {
        let (st, corpo) = self.pedir::<()>(Method::GET, caminho, token, None).await;
        assert!(st.is_success(), "GET {caminho}: {st} {corpo}");
        serde_json::from_str(&corpo).unwrap()
    }

    async fn post<C: Serialize, T: DeserializeOwned>(&self, caminho: &str, token: Option<&str>, corpo: &C) -> T {
        let (st, texto) = self.pedir(Method::POST, caminho, token, Some(corpo)).await;
        assert!(st.is_success(), "POST {caminho}: {st} {texto}");
        serde_json::from_str(&texto).unwrap()
    }

    /// Espera um erro: confere o código HTTP e devolve o `Erro` do corpo.
    async fn falha<C: Serialize>(&self, metodo: Method, caminho: &str, token: Option<&str>, corpo: Option<&C>, esperado: StatusCode) -> Erro {
        let (st, texto) = self.pedir(metodo, caminho, token, corpo).await;
        assert_eq!(st, esperado, "{caminho}: {texto}");
        serde_json::from_str(&texto).unwrap_or_else(|_| panic!("erro fora do formato JSON: {texto}"))
    }

    async fn entrar(&self, email: &str, senha: &str) -> String {
        let r: RespostaLogin = self.post("/api/sessoes", None, &PedidoLogin { email: email.into(), senha: senha.into() }).await;
        r.token
    }
}

fn pedido(evento: i64, categoria: i64, assento: Option<(&str, i32)>) -> Pedido {
    Pedido {
        evento_id: evento,
        ingresso_id: categoria,
        tipo: TipoIngresso::Inteira,
        assento: assento.map(|(f, n)| (f.to_string(), n)),
        forma_pagamento: FormaPagamento::Pix,
    }
}

#[tokio::test]
async fn saude_hoje_e_rota_inexistente() {
    let api = subir().await;
    let (st, corpo) = api.pedir::<()>(Method::GET, "/api/saude", None, None).await;
    assert_eq!((st, corpo.as_str()), (StatusCode::OK, "ok"));
    let h: Hoje = api.get("/api/hoje", None).await;
    assert_eq!(h.hoje.to_string(), "2026-11-21");
    let e = api.falha::<()>(Method::GET, "/api/nao-existe", None, None, StatusCode::NOT_FOUND).await;
    assert!(matches!(e, Erro::NaoEncontrado(_)));
}

#[tokio::test]
async fn login_sessao_e_saida() {
    let api = subir().await;
    let e = api
        .falha(Method::POST, "/api/sessoes", None, Some(&PedidoLogin { email: "ana@email.com".into(), senha: "errada".into() }), StatusCode::FORBIDDEN)
        .await;
    assert_eq!(e.mensagem(), "e-mail ou senha inválidos");

    let token = api.entrar("ana@email.com", "ana12345").await;
    let s: Sessao = api.get("/api/sessao", Some(&token)).await;
    assert_eq!((s.nome.as_str(), s.perfil), ("Ana Souza", Perfil::Cliente));

    let (st, _) = api.pedir::<()>(Method::DELETE, "/api/sessoes", Some(&token), None).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    api.falha::<()>(Method::GET, "/api/sessao", Some(&token), None, StatusCode::FORBIDDEN).await; // token revogado
    api.falha::<()>(Method::GET, "/api/compras", None, None, StatusCode::FORBIDDEN).await; // sem token
}

#[tokio::test]
async fn cadastro_de_cliente_e_organizador() {
    let api = subir().await;
    let cliente = |cpf: &str, email: &str| CadastroCliente { nome: "Carla".into(), cpf: cpf.into(), email: email.into(), senha: "123456".into() };
    let criado: UsuarioCriado = api.post("/api/clientes", None, &cliente("987.654.321-00", "carla@email.com")).await;
    assert_eq!(criado.email, "carla@email.com");
    api.falha(Method::POST, "/api/clientes", None, Some(&cliente("246.813.579-28", "CARLA@email.com")), StatusCode::CONFLICT).await;
    api.falha(Method::POST, "/api/clientes", None, Some(&cliente("111.111.111-11", "x@y.com")), StatusCode::UNPROCESSABLE_ENTITY).await;
    // JSON quebrado também volta no formato da API
    let (st, corpo) = api.pedir(Method::POST, "/api/clientes", None, Some(&serde_json::json!({"nome": 1}))).await;
    assert_eq!(st, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(serde_json::from_str::<Erro>(&corpo).is_ok());

    let org = CadastroOrganizador {
        razao_social: "Produtora Alfa".into(),
        cnpj: "12.ABC.345/01DE-35".into(),
        contato: "71 9999".into(),
        email: "alfa@produtora.com".into(),
        senha: "alfa1234".into(),
    };
    let o: Organizador = api.post("/api/organizadores", None, &org).await;
    assert_eq!(o.cnpj().to_string(), "12.ABC.345/01DE-35");
    let token = api.entrar("alfa@produtora.com", "alfa1234").await;
    let eu: Organizador = api.get("/api/organizador", Some(&token)).await;
    assert_eq!(eu.razao_social(), "Produtora Alfa");
}

#[tokio::test]
async fn fluxo_de_compra_por_http() {
    let api = subir().await;
    let token = api.entrar("ana@email.com", "ana12345").await;
    let eventos: Vec<EventoComCategorias> = api.get("/api/eventos", None).await;
    let show = eventos.iter().find(|e| e.evento.nome().starts_with("Show")).unwrap();
    let cadeira = show.categorias.iter().find(|c| c.categoria() == "Cadeira").unwrap();
    let id = show.evento.id();

    let mapa: MapaAssentos = api.get(&format!("/api/eventos/{id}/categorias/{}/mapa", cadeira.id()), None).await;
    assert_eq!(mapa.livres(), 38);

    let p = pedido(id, cadeira.id(), Some(("B", 2)));
    let resumo: ResumoCompra = api.post("/api/compras/previa", Some(&token), &p).await;
    assert_eq!(resumo.valor.to_string(), "R$ 180,00");
    let compra: CompraConfirmada = api.post("/api/compras", Some(&token), &p).await;

    // o mesmo assento de novo: 409 e nada gravado
    let e = api.falha(Method::POST, "/api/compras", Some(&token), Some(&p), StatusCode::CONFLICT).await;
    assert!(e.mensagem().contains("já vendido"));

    let comprovante: ItemHistorico = api.get(&format!("/api/compras/{}", compra.venda.id()), Some(&token)).await;
    assert_eq!(comprovante.assento.as_deref(), Some("B2"));
    let cancelada: Venda = api.post::<(), _>(&format!("/api/compras/{}/cancelamento", compra.venda.id()), Some(&token), &()).await;
    assert_eq!(cancelada.status().nome(), "Cancelada");
    let mapa: MapaAssentos = api.get(&format!("/api/eventos/{id}/categorias/{}/mapa", cadeira.id()), None).await;
    assert_eq!(mapa.livres(), 38); // assento liberado
}

#[tokio::test]
async fn organizador_cadastra_e_gerencia_evento_por_http() {
    let api = subir().await;
    let token = api.entrar("contato@soleventos.com", "sol12345").await;
    let reais = |t: &str| Dinheiro::de_texto(t).unwrap();
    let dados = DadosEvento {
        nome: "Festa".into(),
        descricao: String::new(),
        local: "Praça".into(),
        data: NaiveDate::from_ymd_opt(2026, 11, 21).unwrap().checked_add_days(Days::new(10)).unwrap(),
        horario: NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        capacidade: 100,
        permite_cancelamento: true,
    };
    let categoria = |nome: &str, quantidade, setor| CategoriaNova { nome: nome.into(), preco: reais("50"), quantidade, setor };

    // duas categorias inválidas: as duas mensagens voltam juntas
    let ruim = EventoNovo {
        dados: dados.clone(),
        categorias: vec![categoria("Pista", 0, None), categoria("Mesa", 0, Some(Setor { fileiras: 30, assentos_por_fileira: 5 }))],
    };
    let e = api.falha(Method::POST, "/api/organizador/eventos", Some(&token), Some(&ruim), StatusCode::UNPROCESSABLE_ENTITY).await;
    assert_eq!(e.mensagem().split(SEPARADOR_MENSAGENS).count(), 2);

    let bom = EventoNovo { dados, categorias: vec![categoria("Pista", 60, None)] };
    let evento: Evento = api.post("/api/organizador/eventos", Some(&token), &bom).await;
    let id = evento.id();
    let mesa: Ingresso = api
        .post(&format!("/api/organizador/eventos/{id}/categorias"), Some(&token), &categoria("Mesa", 0, Some(Setor { fileiras: 4, assentos_por_fileira: 10 })))
        .await;
    assert_eq!(mesa.quantidade(), 40);

    let g: EventoComCategorias = api.get(&format!("/api/organizador/eventos/{id}"), Some(&token)).await;
    let pista = g.categorias.iter().find(|c| c.categoria() == "Pista").unwrap();
    let alt = CategoriaAlterada { nome: "Pista".into(), preco: reais("55,90"), quantidade: Some(50) };
    let (st, texto) = api.pedir(Method::PUT, &format!("/api/organizador/eventos/{id}/categorias/{}", pista.id()), Some(&token), Some(&alt)).await;
    assert_eq!(st, StatusCode::OK, "{texto}");
    let alt = CategoriaAlterada { quantidade: Some(61), ..alt }; // 61 + 40 > 100
    api.falha(Method::PUT, &format!("/api/organizador/eventos/{id}/categorias/{}", pista.id()), Some(&token), Some(&alt), StatusCode::CONFLICT).await;

    let r: RelatorioEvento = api.get(&format!("/api/organizador/eventos/{id}/relatorio"), Some(&token)).await;
    assert_eq!(r.vendidos, 0);
    let (st, _) = api.pedir::<()>(Method::DELETE, &format!("/api/organizador/eventos/{id}"), Some(&token), None).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn permissoes_por_perfil() {
    let api = subir().await;
    let ana = api.entrar("ana@email.com", "ana12345").await;
    let sol = api.entrar("contato@soleventos.com", "sol12345").await;
    let admin = api.entrar("admin@ingressos.com", "admin123").await;

    api.falha::<()>(Method::GET, "/api/admin/organizadores", Some(&ana), None, StatusCode::FORBIDDEN).await;
    api.falha::<()>(Method::GET, "/api/organizador/eventos", Some(&ana), None, StatusCode::FORBIDDEN).await;
    api.falha::<()>(Method::GET, "/api/compras", Some(&sol), None, StatusCode::FORBIDDEN).await;
    let orgs: Vec<Organizador> = api.get("/api/admin/organizadores", Some(&admin)).await;
    assert_eq!(orgs.len(), 2);

    // bloqueio pelo admin impede o login do organizador
    let id = orgs.iter().find(|o| o.razao_social() == "Sol Eventos Ltda").unwrap().id();
    let _: Organizador = api.post::<(), _>(&format!("/api/admin/organizadores/{id}/bloquear"), Some(&admin), &()).await;
    api.falha(Method::POST, "/api/sessoes", None, Some(&PedidoLogin { email: "contato@soleventos.com".into(), senha: "sol12345".into() }), StatusCode::FORBIDDEN)
        .await;
    // a sessão que já estava aberta também perde o acesso às funções de organizador
    api.falha::<()>(Method::GET, "/api/organizador/eventos", Some(&sol), None, StatusCode::FORBIDDEN).await;
}

#[tokio::test]
async fn login_expira() {
    let api = subir_com_validade(std::time::Duration::from_millis(50)).await;
    let token = api.entrar("ana@email.com", "ana12345").await;
    let _: Sessao = api.get("/api/sessao", Some(&token)).await;
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    let e = api.falha::<()>(Method::GET, "/api/sessao", Some(&token), None, StatusCode::FORBIDDEN).await;
    assert!(e.mensagem().contains("expirada"));
}
