//! Cliente HTTP do back-end (contêiner `back`).
//!
//! Tem os mesmos grupos e nomes de métodos dos serviços da aplicação (`autenticacao`, `eventos`,
//! `vendas`, `relatorios`, `administracao`), mas cada chamada vira uma requisição JSON à API.
//! O front nunca acessa o banco de dados.

use chrono::NaiveDate;
use contrato::*;
use reqwest::{Client, Method};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::sessao::Sessao;

/// Conexão HTTP compartilhada pelos grupos de operações.
#[derive(Clone)]
struct Http {
    cliente: Client,
    base: String,
}

impl Http {
    async fn pedir<T: DeserializeOwned, C: Serialize + ?Sized>(
        &self,
        metodo: Method,
        caminho: &str,
        token: Option<&str>,
        corpo: Option<&C>,
    ) -> Resultado<T> {
        let mut req = self.cliente.request(metodo, format!("{}{caminho}", self.base));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        if let Some(c) = corpo {
            req = req.json(c);
        }
        let resposta = req.send().await.map_err(|e| {
            eprintln!("[back indisponível] {e}");
            Erro::Infraestrutura("serviço indisponível no momento. Tente novamente em instantes".to_string())
        })?;
        let status = resposta.status();
        let texto = resposta.text().await.map_err(|e| Erro::Infraestrutura(e.to_string()))?;
        if status.is_success() {
            // 204 (sem corpo) vira `()`
            let texto = if texto.is_empty() { "null" } else { texto.as_str() };
            serde_json::from_str(texto).map_err(|e| Erro::Infraestrutura(format!("resposta inesperada do back: {e}")))
        } else {
            // o back sempre responde o erro como JSON do domínio; se não, é falha técnica
            Err(serde_json::from_str::<Erro>(&texto)
                .unwrap_or_else(|_| Erro::Infraestrutura(format!("erro {status} no back: {texto}"))))
        }
    }

    async fn get<T: DeserializeOwned>(&self, caminho: &str, token: Option<&str>) -> Resultado<T> {
        self.pedir::<T, ()>(Method::GET, caminho, token, None).await
    }

    async fn enviar<T: DeserializeOwned, C: Serialize + ?Sized>(
        &self,
        metodo: Method,
        caminho: &str,
        token: Option<&str>,
        corpo: &C,
    ) -> Resultado<T> {
        self.pedir(metodo, caminho, token, Some(corpo)).await
    }

    async fn sem_corpo<T: DeserializeOwned>(&self, metodo: Method, caminho: &str, token: &str) -> Resultado<T> {
        self.pedir::<T, ()>(metodo, caminho, Some(token), None).await
    }
}

/// Fachada do back-end para as telas (mesma organização da `Aplicacao`).
pub struct Backend {
    pub autenticacao: ApiAutenticacao,
    pub eventos: ApiEventos,
    pub vendas: ApiVendas,
    pub relatorios: ApiRelatorios,
    pub administracao: ApiAdministracao,
    http: Http,
}

impl Backend {
    /// `base`: endereço do back, ex.: `http://back:8080`.
    pub fn novo(base: &str) -> Self {
        let http = Http { cliente: Client::new(), base: base.trim_end_matches('/').to_string() };
        Backend {
            autenticacao: ApiAutenticacao(http.clone()),
            eventos: ApiEventos(http.clone()),
            vendas: ApiVendas(http.clone()),
            relatorios: ApiRelatorios(http.clone()),
            administracao: ApiAdministracao(http.clone()),
            http,
        }
    }

    /// Data de hoje segundo o back (a mesma usada nas regras de negócio).
    pub async fn hoje(&self) -> Resultado<NaiveDate> {
        Ok(self.http.get::<Hoje>("/api/hoje", None).await?.hoje)
    }
}

fn pares(lista: Vec<EventoComCategorias>) -> Vec<(Evento, Vec<Ingresso>)> {
    lista.into_iter().map(|e| (e.evento, e.categorias)).collect()
}

// ------------------------------- autenticação -------------------------------

pub struct ApiAutenticacao(Http);

impl ApiAutenticacao {
    /// Algoritmo 1 — login; devolve a sessão com o token emitido pelo back.
    pub async fn entrar(&self, email: &str, senha: &str) -> Resultado<Sessao> {
        let pedido = PedidoLogin { email: email.to_string(), senha: senha.to_string() };
        let r: RespostaLogin = self.0.enviar(Method::POST, "/api/sessoes", None, &pedido).await?;
        Ok(Sessao::nova(r.token, r.sessao))
    }

    pub async fn sair(&self, s: &Sessao) -> Resultado<()> {
        self.0.sem_corpo(Method::DELETE, "/api/sessoes", &s.token).await
    }

    /// Recupera a sessão de um token (usado se o front reiniciar e perder o cache).
    pub async fn sessao(&self, token: &str) -> Resultado<Sessao> {
        let dados = self.0.get("/api/sessao", Some(token)).await?;
        Ok(Sessao::nova(token.to_string(), dados))
    }

    pub async fn cadastrar_cliente(&self, dados: &CadastroCliente) -> Resultado<UsuarioCriado> {
        self.0.enviar(Method::POST, "/api/clientes", None, dados).await
    }

    pub async fn cadastrar_organizador(&self, dados: &CadastroOrganizador) -> Resultado<Organizador> {
        self.0.enviar(Method::POST, "/api/organizadores", None, dados).await
    }
}

// --------------------------------- eventos ----------------------------------

pub struct ApiEventos(Http);

impl ApiEventos {
    pub async fn listar_disponiveis(&self) -> Resultado<Vec<(Evento, Vec<Ingresso>)>> {
        Ok(pares(self.0.get("/api/eventos", None).await?))
    }

    pub async fn categorias(&self, evento_id: i64) -> Resultado<Vec<Ingresso>> {
        self.0.get(&format!("/api/eventos/{evento_id}/categorias"), None).await
    }

    pub async fn mapa(&self, ingresso: &Ingresso) -> Resultado<MapaAssentos> {
        self.0.get(&format!("/api/eventos/{}/categorias/{}/mapa", ingresso.evento_id(), ingresso.id()), None).await
    }

    pub async fn organizador_ativo(&self, s: &Sessao) -> Resultado<Organizador> {
        self.0.get("/api/organizador", Some(&s.token)).await
    }

    /// Eventos do organizador, cada um com suas categorias.
    pub async fn listar_do_organizador(&self, s: &Sessao) -> Resultado<Vec<(Evento, Vec<Ingresso>)>> {
        Ok(pares(self.0.get("/api/organizador/eventos", Some(&s.token)).await?))
    }

    /// Algoritmo 3 — cadastra o evento com todas as categorias.
    pub async fn cadastrar(&self, s: &Sessao, novo: &EventoNovo) -> Resultado<Evento> {
        self.0.enviar(Method::POST, "/api/organizador/eventos", Some(&s.token), novo).await
    }

    pub async fn gerenciar(&self, s: &Sessao, evento_id: i64) -> Resultado<(Evento, Vec<Ingresso>)> {
        let e: EventoComCategorias = self.0.get(&format!("/api/organizador/eventos/{evento_id}"), Some(&s.token)).await?;
        Ok((e.evento, e.categorias))
    }

    pub async fn alterar(&self, s: &Sessao, evento_id: i64, dados: DadosEvento, status: StatusEvento) -> Resultado<Evento> {
        let corpo = EventoAlterado { dados, status };
        self.0.enviar(Method::PUT, &format!("/api/organizador/eventos/{evento_id}"), Some(&s.token), &corpo).await
    }

    pub async fn remover(&self, s: &Sessao, evento_id: i64) -> Resultado<()> {
        self.0.sem_corpo(Method::DELETE, &format!("/api/organizador/eventos/{evento_id}"), &s.token).await
    }

    pub async fn adicionar_categoria(&self, s: &Sessao, evento_id: i64, categoria: &CategoriaNova) -> Resultado<Ingresso> {
        let caminho = format!("/api/organizador/eventos/{evento_id}/categorias");
        self.0.enviar(Method::POST, &caminho, Some(&s.token), categoria).await
    }

    pub async fn categoria_do_organizador(&self, s: &Sessao, evento_id: i64, ingresso_id: i64) -> Resultado<(Evento, Ingresso)> {
        let caminho = format!("/api/organizador/eventos/{evento_id}/categorias/{ingresso_id}");
        let c: CategoriaDoEvento = self.0.get(&caminho, Some(&s.token)).await?;
        Ok((c.evento, c.ingresso))
    }

    pub async fn alterar_categoria(
        &self,
        s: &Sessao,
        evento_id: i64,
        ingresso_id: i64,
        nome: &str,
        preco: Dinheiro,
        quantidade: Option<i32>,
    ) -> Resultado<Ingresso> {
        let caminho = format!("/api/organizador/eventos/{evento_id}/categorias/{ingresso_id}");
        let corpo = CategoriaAlterada { nome: nome.to_string(), preco, quantidade };
        self.0.enviar(Method::PUT, &caminho, Some(&s.token), &corpo).await
    }

    pub async fn remover_categoria(&self, s: &Sessao, evento_id: i64, ingresso_id: i64) -> Resultado<()> {
        let caminho = format!("/api/organizador/eventos/{evento_id}/categorias/{ingresso_id}");
        self.0.sem_corpo(Method::DELETE, &caminho, &s.token).await
    }
}

// ---------------------------------- vendas ----------------------------------

pub struct ApiVendas(Http);

impl ApiVendas {
    /// Algoritmo 4, passo 1.
    pub async fn evento_para_venda(&self, evento_id: i64) -> Resultado<Evento> {
        self.0.get(&format!("/api/eventos/{evento_id}/venda"), None).await
    }

    /// Algoritmo 4, passo 2.
    pub async fn ingresso_para_venda(&self, evento: &Evento, ingresso_id: i64) -> Resultado<Ingresso> {
        self.0.get(&format!("/api/eventos/{}/categorias/{ingresso_id}/venda", evento.id()), None).await
    }

    pub async fn preparar(&self, s: &Sessao, pedido: &Pedido) -> Resultado<ResumoCompra> {
        self.0.enviar(Method::POST, "/api/compras/previa", Some(&s.token), pedido).await
    }

    /// Algoritmo 4 — confirma a compra.
    pub async fn confirmar(&self, s: &Sessao, pedido: &Pedido) -> Resultado<(Venda, ResumoCompra)> {
        let c: CompraConfirmada = self.0.enviar(Method::POST, "/api/compras", Some(&s.token), pedido).await?;
        Ok((c.venda, c.resumo))
    }

    pub async fn historico(&self, s: &Sessao) -> Resultado<Vec<ItemHistorico>> {
        self.0.get("/api/compras", Some(&s.token)).await
    }

    pub async fn comprovante(&self, s: &Sessao, venda_id: i64) -> Resultado<ItemHistorico> {
        self.0.get(&format!("/api/compras/{venda_id}"), Some(&s.token)).await
    }

    /// Algoritmo 5 — cancelar compra.
    pub async fn cancelar(&self, s: &Sessao, venda_id: i64) -> Resultado<Venda> {
        self.0.sem_corpo(Method::POST, &format!("/api/compras/{venda_id}/cancelamento"), &s.token).await
    }
}

// -------------------------------- relatórios --------------------------------

pub struct ApiRelatorios(Http);

impl ApiRelatorios {
    /// Algoritmo 6 — relatório do evento.
    pub async fn relatorio(&self, s: &Sessao, evento_id: i64) -> Resultado<RelatorioEvento> {
        self.0.get(&format!("/api/organizador/eventos/{evento_id}/relatorio"), Some(&s.token)).await
    }

    pub async fn vendas_do_evento(&self, s: &Sessao, evento_id: i64) -> Resultado<Vec<VendaDetalhada>> {
        self.0.get(&format!("/api/organizador/eventos/{evento_id}/vendas"), Some(&s.token)).await
    }

    pub async fn registrar_presenca(&self, s: &Sessao, venda_id: i64) -> Resultado<Venda> {
        self.0.sem_corpo(Method::POST, &format!("/api/organizador/vendas/{venda_id}/checkin"), &s.token).await
    }
}

// ------------------------------ administração -------------------------------

pub struct ApiAdministracao(Http);

impl ApiAdministracao {
    pub async fn listar(&self, s: &Sessao) -> Resultado<Vec<Organizador>> {
        self.0.get("/api/admin/organizadores", Some(&s.token)).await
    }

    pub async fn cadastrar_organizador(&self, s: &Sessao, dados: &CadastroOrganizador) -> Resultado<Organizador> {
        self.0.enviar(Method::POST, "/api/admin/organizadores", Some(&s.token), dados).await
    }

    pub async fn buscar(&self, s: &Sessao, id: i64) -> Resultado<Organizador> {
        self.0.get(&format!("/api/admin/organizadores/{id}"), Some(&s.token)).await
    }

    pub async fn editar(&self, s: &Sessao, id: i64, razao_social: &str, contato: &str) -> Resultado<Organizador> {
        let corpo = OrganizadorEditado { razao_social: razao_social.to_string(), contato: contato.to_string() };
        self.0.enviar(Method::PUT, &format!("/api/admin/organizadores/{id}"), Some(&s.token), &corpo).await
    }

    pub async fn ativar(&self, s: &Sessao, id: i64) -> Resultado<Organizador> {
        self.0.sem_corpo(Method::POST, &format!("/api/admin/organizadores/{id}/ativar"), &s.token).await
    }

    pub async fn bloquear(&self, s: &Sessao, id: i64) -> Resultado<Organizador> {
        self.0.sem_corpo(Method::POST, &format!("/api/admin/organizadores/{id}/bloquear"), &s.token).await
    }

    pub async fn remover(&self, s: &Sessao, id: i64) -> Resultado<()> {
        self.0.sem_corpo(Method::DELETE, &format!("/api/admin/organizadores/{id}"), &s.token).await
    }
}
