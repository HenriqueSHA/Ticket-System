//! Testes ponta a ponta: sobem o **back** (API com repositórios em memória) e o **front** em
//! portas livres e navegam pelas telas como um navegador (cookies + formulários).

use std::sync::Arc;

use aplicacao::{Aplicacao, Dependencias, dados_exemplo};
use chrono::{Days, NaiveDate};
use infraestrutura::memoria::RepositoriosMemoria;
use infraestrutura::relogio::RelogioFixo;
use infraestrutura::seguranca::HashArgon2;
use reqwest::{Client, StatusCode};

fn hoje() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 21).unwrap()
}

/// Sobe back + front e devolve o endereço do front.
async fn subir() -> String {
    let repos = RepositoriosMemoria::novo();
    let app = Aplicacao::nova(Dependencias {
        usuarios: Arc::new(repos.clone()),
        organizadores: Arc::new(repos.clone()),
        eventos: Arc::new(repos.clone()),
        vendas: Arc::new(repos),
        hash: Arc::new(HashArgon2),
        relogio: Arc::new(RelogioFixo::em(hoje().and_hms_opt(12, 0, 0).unwrap())),
    });
    dados_exemplo::popular_se_vazio(&app).await.unwrap();
    let back = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url_back = format!("http://{}", back.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(back, api::rotas::criar(api::EstadoApi::novo(app))).await.unwrap() });
    subir_front(&url_back).await
}

async fn subir_front(url_back: &str) -> String {
    let front = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", front.local_addr().unwrap());
    let estado = web::Estado::novo(url_back, false);
    tokio::spawn(async move { axum::serve(front, web::rotas::criar(estado)).await.unwrap() });
    url
}

/// "Navegador": guarda cookies e segue redirecionamentos.
fn navegador() -> Client {
    Client::builder().cookie_store(true).build().unwrap()
}

async fn pagina(n: &Client, url: &str) -> (StatusCode, String) {
    let r = n.get(url).send().await.unwrap();
    (r.status(), r.text().await.unwrap())
}

async fn enviar(n: &Client, url: &str, campos: &[(&str, &str)]) -> (String, String) {
    let r = n.post(url).form(campos).send().await.unwrap();
    let final_url = r.url().path().to_string();
    (final_url, r.text().await.unwrap())
}

async fn entrar(n: &Client, front: &str, email: &str, senha: &str) -> String {
    let (destino, _) = enviar(n, &format!("{front}/entrar"), &[("email", email), ("senha", senha)]).await;
    destino
}

/// Primeiro número depois de `prefixo` no HTML (ex.: id do evento num link).
fn numero_apos(html: &str, prefixo: &str) -> String {
    let resto = &html[html.find(prefixo).unwrap_or_else(|| panic!("não achei {prefixo}")) + prefixo.len()..];
    resto.chars().take_while(char::is_ascii_digit).collect()
}

#[tokio::test]
async fn sem_login_vai_para_a_entrada() {
    let front = subir().await;
    let n = navegador();
    let r = n.get(format!("{front}/meus-ingressos")).send().await.unwrap();
    assert_eq!(r.url().path(), "/entrar");
}

#[tokio::test]
async fn cliente_compra_pelo_navegador() {
    let front = subir().await;
    let n = navegador();
    assert_eq!(entrar(&n, &front, "ana@email.com", "ana12345").await, "/eventos");

    let (_, eventos) = pagina(&n, &format!("{front}/eventos")).await;
    assert!(eventos.contains("Show de Rock"));
    let id = numero_apos(&eventos, "href=\"/eventos/");

    // página de compra com o mapa da Cadeira
    let (_, compra) = pagina(&n, &format!("{front}/eventos/{id}")).await;
    // aba da categoria: <a ... href="/eventos/ID?categoria=N">Cadeira · ...
    let cadeira: String = compra
        .split("?categoria=")
        .find(|trecho| trecho.split_once('"').is_some_and(|(_, resto)| resto.starts_with(">Cadeira")))
        .map(|trecho| trecho.chars().take_while(char::is_ascii_digit).collect())
        .expect("aba da Cadeira");
    let (_, mapa) = pagina(&n, &format!("{front}/eventos/{id}?categoria={cadeira}")).await;
    assert!(mapa.contains("value=\"B-2\""), "assento B2 livre no mapa");
    assert!(!mapa.contains("value=\"A-4\""), "A4 já vendido não pode ser escolhido");

    let campos = [("categoria", cadeira.as_str()), ("tipo", "MEIA"), ("assento", "B-2"), ("pagamento", "PIX")];
    let (_, resumo) = enviar(&n, &format!("{front}/eventos/{id}/resumo"), &campos).await;
    assert!(resumo.contains("R$ 90,00"));
    let (destino, comprovante) = enviar(&n, &format!("{front}/eventos/{id}/confirmar"), &campos).await;
    assert!(destino.starts_with("/compras/"), "{destino}");
    assert!(comprovante.contains("Compra realizada!") && comprovante.contains("B2"));

    // de novo o mesmo assento: volta para a compra com o aviso
    let (destino, pagina_erro) = enviar(&n, &format!("{front}/eventos/{id}/confirmar"), &campos).await;
    assert_eq!(destino, format!("/eventos/{id}"));
    assert!(pagina_erro.contains("já vendido"));

    let (_, meus) = pagina(&n, &format!("{front}/meus-ingressos")).await;
    assert!(meus.contains("B2") && meus.contains("Cancelar"));
}

#[tokio::test]
async fn cadastros_pelas_telas() {
    let front = subir().await;
    let n = navegador();
    let cliente = |cpf: &'static str| [("nome", "Carla"), ("cpf", cpf), ("email", "carla@email.com"), ("senha", "123456")];
    let (destino, html) = enviar(&n, &format!("{front}/cadastro"), &cliente("111.111.111-11")).await;
    assert_eq!(destino, "/cadastro");
    assert!(html.contains("CPF inválido") && html.contains("value=\"carla@email.com\""), "erro e campos preservados");
    let (destino, html) = enviar(&n, &format!("{front}/cadastro"), &cliente("987.654.321-00")).await;
    assert_eq!(destino, "/entrar");
    assert!(html.contains("Conta criada!"));
    let (_, html) = enviar(&n, &format!("{front}/cadastro"), &cliente("246.813.579-28")).await;
    assert!(html.contains("já existe um usuário com este e-mail"));

    let org = [("razao_social", "Produtora Alfa"), ("cnpj", "12.ABC.345/01DE-35"), ("contato", "71 9999"), ("email", "alfa@p.com"), ("senha", "alfa1234")];
    let (destino, _) = enviar(&n, &format!("{front}/cadastro/organizador"), &org).await;
    assert_eq!(destino, "/entrar");
    assert_eq!(entrar(&n, &front, "alfa@p.com", "alfa1234").await, "/organizador/eventos");
}

#[tokio::test]
async fn organizador_cria_evento_pelas_telas() {
    let front = subir().await;
    let n = navegador();
    entrar(&n, &front, "contato@soleventos.com", "sol12345").await;
    let data = hoje().checked_add_days(Days::new(10)).unwrap().format("%Y-%m-%d").to_string();
    let base = [("nome", "Festa"), ("descricao", ""), ("local", "Praça"), ("data", data.as_str()), ("horario", "20:00"), ("capacidade", "100"), ("permite_cancelamento", "on")];

    // duas categorias inválidas (quantidade 0 e 30 fileiras): as duas mensagens aparecem
    let mut ruim: Vec<(&str, &str)> = base.to_vec();
    ruim.extend([("cat_nome", "Pista"), ("cat_preco", "50"), ("cat_assento", "nao"), ("cat_quantidade", "0"), ("cat_fileiras", ""), ("cat_por_fileira", "")]);
    ruim.extend([("cat_nome", "Mesa"), ("cat_preco", "90"), ("cat_assento", "sim"), ("cat_quantidade", ""), ("cat_fileiras", "30"), ("cat_por_fileira", "5")]);
    let (destino, html) = enviar(&n, &format!("{front}/organizador/eventos/novo"), &ruim).await;
    assert_eq!(destino, "/organizador/eventos/novo");
    assert!(html.contains("Categoria 1 (Pista)") && html.contains("Categoria 2 (Mesa)"));

    let mut bom: Vec<(&str, &str)> = base.to_vec();
    bom.extend([("cat_nome", "Pista"), ("cat_preco", "50,00"), ("cat_assento", "nao"), ("cat_quantidade", "60"), ("cat_fileiras", ""), ("cat_por_fileira", "")]);
    let (destino, html) = enviar(&n, &format!("{front}/organizador/eventos/novo"), &bom).await;
    assert!(destino.starts_with("/organizador/eventos/"), "{destino}");
    assert!(html.contains("Festa") && html.contains("Ingressos por categoria"));
}

#[tokio::test]
async fn back_fora_do_ar_mostra_mensagem_amigavel() {
    // porta fechada: nenhum back respondendo
    let livre = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url_morta = format!("http://{}", livre.local_addr().unwrap());
    drop(livre);
    let front = subir_front(&url_morta).await;
    let n = navegador();
    let (destino, html) = enviar(&n, &format!("{front}/entrar"), &[("email", "ana@email.com"), ("senha", "ana12345")]).await;
    assert_eq!(destino, "/entrar");
    assert!(html.contains("indisponível"), "{html}");
}

/// Texto digitado pelo usuário nunca vira código na página (proteção contra XSS).
#[tokio::test]
async fn nome_malicioso_nao_vira_codigo() {
    let front = subir().await;
    let n = navegador();
    let malicioso = "Produtora x'); alert(1);//<script>alert(2)</script>";
    let org = [("razao_social", malicioso), ("cnpj", "33.444.555/0001-81"), ("contato", "71"), ("email", "x@x.com"), ("senha", "123456")];
    let (destino, _) = enviar(&n, &format!("{front}/cadastro/organizador"), &org).await;
    assert_eq!(destino, "/entrar");

    let admin = navegador();
    entrar(&admin, &front, "admin@ingressos.com", "admin123").await;
    let (_, html) = pagina(&admin, &format!("{front}/admin/organizadores")).await;
    assert!(!html.contains("<script>alert(2)"), "tag de script não pode aparecer crua");
    assert!(!html.contains("onsubmit"), "nenhum dado dentro de JavaScript inline");
    assert!(html.contains("&lt;script&gt;alert(2)&lt;/script&gt;"), "o texto aparece escapado");
}
