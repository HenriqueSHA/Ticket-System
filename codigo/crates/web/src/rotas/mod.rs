//! Rotas (endereços) da interface web. Um módulo por perfil de usuário.

mod admin;
mod cliente;
mod organizador;
mod publico;

use crate::sessao::Sessao;
use askama::Template;
use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum_extra::extract::CookieJar;
use dominio::{Erro, Resultado};

use crate::Estado;
use crate::paginas::{Layout, PaginaErro};
use crate::sessao::Aviso;

pub fn criar(estado: Estado) -> Router {
    Router::new()
        .route("/", get(publico::inicio))
        .route("/estilo.css", get(estilo))
        .route("/saude", get(|| async { "ok" }))
        .route("/entrar", get(publico::form_entrar).post(publico::entrar))
        .route("/cadastro", get(publico::form_cadastro).post(publico::cadastrar))
        .route("/cadastro/organizador", get(publico::form_cadastro_organizador).post(publico::cadastrar_organizador))
        .route("/sair", post(publico::sair))
        // cliente
        .route("/eventos", get(cliente::eventos))
        .route("/eventos/{id}", get(cliente::comprar))
        .route("/eventos/{id}/resumo", post(cliente::resumo))
        .route("/eventos/{id}/confirmar", post(cliente::confirmar))
        .route("/compras/{id}", get(cliente::comprovante))
        .route("/compras/{id}/cancelar", post(cliente::cancelar))
        .route("/meus-ingressos", get(cliente::meus_ingressos))
        // organizador
        .route("/organizador/eventos", get(organizador::eventos))
        .route("/organizador/eventos/novo", get(organizador::form_novo).post(organizador::criar))
        .route("/organizador/eventos/{id}", get(organizador::gerenciar))
        .route("/organizador/eventos/{id}/editar", get(organizador::form_editar).post(organizador::editar))
        .route("/organizador/eventos/{id}/excluir", post(organizador::excluir))
        .route("/organizador/eventos/{id}/categorias", post(organizador::adicionar_categoria))
        .route(
            "/organizador/eventos/{id}/categorias/{categoria}",
            get(organizador::form_categoria).post(organizador::alterar_categoria),
        )
        .route("/organizador/eventos/{id}/categorias/{categoria}/remover", post(organizador::remover_categoria))
        .route("/organizador/eventos/{id}/relatorio", get(organizador::relatorio))
        .route("/organizador/eventos/{id}/vendas/{venda}/checkin", post(organizador::checkin_da_lista))
        .route("/organizador/checkin", get(organizador::form_checkin).post(organizador::checkin))
        // superadmin
        .route("/admin/organizadores", get(admin::lista))
        .route("/admin/organizadores/novo", get(admin::form_novo).post(admin::criar))
        .route("/admin/organizadores/{id}/editar", get(admin::form_editar).post(admin::editar))
        .route("/admin/organizadores/{id}/{acao}", post(admin::acao))
        .with_state(estado)
}

async fn estilo() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], include_str!("../../static/estilo.css"))
}

/// Gera o HTML da página.
pub(crate) fn render<T: Template>(pagina: T) -> Response {
    match pagina.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Erro ao montar a página: {e}")).into_response(),
    }
}

/// Redireciona guardando um aviso para a próxima página (padrão Post/Redirect/Get).
pub(crate) fn redirecionar(estado: &Estado, jar: CookieJar, destino: &str, aviso: Aviso) -> Response {
    (estado.sessoes.avisar(jar, aviso), Redirect::to(destino)).into_response()
}

/// Página de erro com o código HTTP adequado ao tipo de erro.
pub(crate) fn falha(sessao: Option<&Sessao>, erro: &Erro, voltar: &str) -> Response {
    let status = match erro {
        Erro::Validacao(_) => StatusCode::UNPROCESSABLE_ENTITY,
        Erro::NaoEncontrado(_) => StatusCode::NOT_FOUND,
        Erro::RegraNegocio(_) => StatusCode::CONFLICT,
        Erro::AcessoNegado(_) => StatusCode::FORBIDDEN,
        Erro::Infraestrutura(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let pagina = PaginaErro {
        layout: Layout::novo("Não foi possível concluir", sessao, None),
        mensagem: erro.to_string(),
        voltar: voltar.to_string(),
    };
    (status, render(pagina)).into_response()
}

/// Data de hoje segundo o back (se ele não responder, usa a do próprio servidor).
pub(crate) async fn hoje(estado: &Estado) -> chrono::NaiveDate {
    estado.back.hoje().await.unwrap_or_else(|_| chrono::Local::now().date_naive())
}

/// Lê um número inteiro de um campo de formulário.
pub(crate) fn inteiro(texto: &str, campo: &str) -> Resultado<i32> {
    texto.trim().parse().map_err(|_| Erro::validacao(format!("o campo \"{campo}\" deve ser um número inteiro")))
}

/// Executa e, se der erro, devolve a página de erro (o `RETORNE` dos fluxogramas).
macro_rules! tentar {
    ($expr:expr, $sessao:expr, $voltar:expr) => {
        match $expr {
            Ok(valor) => valor,
            Err(erro) => return $crate::rotas::falha(Some(&$sessao), &erro, &$voltar),
        }
    };
}
pub(crate) use tentar;
