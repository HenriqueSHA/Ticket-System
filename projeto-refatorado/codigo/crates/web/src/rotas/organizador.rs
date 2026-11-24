//! Páginas do organizador: cadastro de eventos (Algoritmo 3), gestão do evento (categorias,
//! preços, quantidades, capacidade, exclusão), relatório, vendas e check-in (Algoritmo 6).

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::{CookieJar, Form};
use chrono::{NaiveDate, NaiveTime};
use contrato::{CategoriaNova, EventoNovo, SEPARADOR_MENSAGENS};
use dominio::entidades::{DadosEvento, Ingresso, Setor, StatusEvento, total_disponivel};
use dominio::valores::Dinheiro;
use dominio::{Erro, Resultado};
use serde::Deserialize;

use super::{hoje, inteiro, redirecionar, render, tentar};
use crate::Estado;
use crate::paginas::*;
use crate::sessao::{Aviso, Logado, Sessao};

pub async fn eventos(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar) -> Response {
    let lista = tentar!(estado.back.eventos.listar_do_organizador(&s).await, s, "/");
    let hoje = hoje(&estado).await;
    let eventos = lista
        .into_iter()
        .map(|(evento, categorias)| LinhaEventoOrganizador {
            realizado: evento.ja_ocorreu(hoje),
            disponiveis: total_disponivel(&categorias),
            evento,
        })
        .collect();
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    (jar, render(PaginaEventosOrganizador { layout: Layout::novo("Meus eventos", Some(&s), aviso), eventos }))
        .into_response()
}

/// Formulário do evento. Os campos de categoria vêm repetidos (uma linha por categoria).
#[derive(Deserialize)]
pub struct FormEvento {
    nome: String,
    #[serde(default)]
    descricao: String,
    local: String,
    data: String,
    horario: String,
    #[serde(default)]
    capacidade: String,
    permite_cancelamento: Option<String>,
    vendas_abertas: Option<String>,
    #[serde(default)]
    cat_nome: Vec<String>,
    #[serde(default)]
    cat_preco: Vec<String>,
    #[serde(default)]
    cat_assento: Vec<String>,
    #[serde(default)]
    cat_quantidade: Vec<String>,
    #[serde(default)]
    cat_fileiras: Vec<String>,
    #[serde(default)]
    cat_por_fileira: Vec<String>,
}

impl FormEvento {
    fn campos(&self) -> CamposEvento {
        CamposEvento {
            nome: self.nome.clone(),
            descricao: self.descricao.clone(),
            local: self.local.clone(),
            data: self.data.clone(),
            horario: self.horario.clone(),
            capacidade: self.capacidade.clone(),
            permite_cancelamento: self.permite_cancelamento.is_some(),
            vendas_abertas: self.vendas_abertas.is_some(),
        }
    }

    /// Linhas de categoria preenchidas (linhas totalmente vazias são ignoradas).
    fn categorias(&self) -> Vec<LinhaCategoria> {
        let pegar = |v: &Vec<String>, i: usize| v.get(i).cloned().unwrap_or_default();
        (0..self.cat_nome.len())
            .map(|i| LinhaCategoria {
                nome: pegar(&self.cat_nome, i),
                preco: pegar(&self.cat_preco, i),
                com_assento: pegar(&self.cat_assento, i) == "sim",
                quantidade: pegar(&self.cat_quantidade, i),
                fileiras: pegar(&self.cat_fileiras, i),
                por_fileira: pegar(&self.cat_por_fileira, i),
            })
            .filter(|l| !(l.nome.trim().is_empty() && l.preco.trim().is_empty()))
            .collect()
    }

    fn dados(&self, capacidade: Option<i32>) -> Resultado<DadosEvento> {
        let data = NaiveDate::parse_from_str(self.data.trim(), "%Y-%m-%d")
            .map_err(|_| Erro::validacao("informe a data do evento"))?;
        let horario = NaiveTime::parse_from_str(self.horario.trim(), "%H:%M")
            .map_err(|_| Erro::validacao("informe o horário do evento"))?;
        let capacidade = match capacidade {
            Some(c) => c,
            None => inteiro(&self.capacidade, "capacidade")?,
        };
        Ok(DadosEvento {
            nome: self.nome.clone(),
            descricao: self.descricao.clone(),
            local: self.local.clone(),
            data,
            horario,
            capacidade,
            permite_cancelamento: self.permite_cancelamento.is_some(),
        })
    }
}

/// Lê uma linha de categoria do formulário. As regras (quantidade > 0, limites do mapa,
/// capacidade) são conferidas pelo back.
fn categoria_do_formulario(l: &LinhaCategoria) -> Resultado<CategoriaNova> {
    let preco = Dinheiro::de_texto(&l.preco)?;
    let (quantidade, setor) = if l.com_assento {
        let setor = Setor {
            fileiras: inteiro(&l.fileiras, "fileiras")?,
            assentos_por_fileira: inteiro(&l.por_fileira, "assentos por fileira")?,
        };
        (setor.fileiras * setor.assentos_por_fileira, Some(setor))
    } else {
        (inteiro(&l.quantidade, "quantidade")?, None)
    };
    Ok(CategoriaNova { nome: l.nome.clone(), preco, quantidade, setor })
}

fn pagina_novo(s: &Sessao, campos: CamposEvento, categorias: Vec<LinhaCategoria>, erros: Vec<String>) -> Response {
    render(PaginaFormEvento {
        layout: Layout::novo("Novo evento", Some(s), None),
        acao: "/organizador/eventos/novo".into(),
        editando: false,
        campos,
        categorias,
        erros,
        capacidade_minima: 1,
        voltar: "/organizador/eventos".into(),
    })
}

pub async fn form_novo(State(estado): State<Estado>, Logado(s): Logado) -> Response {
    tentar!(estado.back.eventos.organizador_ativo(&s).await, s, "/");
    let campos = CamposEvento { permite_cancelamento: true, vendas_abertas: true, ..Default::default() };
    // linhas em branco (só com exemplo no placeholder): as que ficarem vazias são ignoradas
    let categorias = vec![LinhaCategoria::default(), LinhaCategoria { com_assento: true, ..Default::default() }];
    pagina_novo(&s, campos, categorias, vec![])
}

/// Algoritmo 3 — lê o formulário e envia ao back, que valida os dados, cada categoria (laço)
/// e grava tudo junto. Os erros voltam um por linha.
pub async fn criar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Form(f): Form<FormEvento>,
) -> Response {
    let (campos, linhas) = (f.campos(), f.categorias());
    let dados = match f.dados(None) {
        Ok(d) => d,
        Err(e) => return pagina_novo(&s, campos, linhas, vec![e.mensagem().to_string()]),
    };
    let mut categorias = Vec::new();
    let mut erros = Vec::new();
    for (i, linha) in linhas.iter().enumerate() {
        match categoria_do_formulario(linha) {
            Ok(c) => categorias.push(c),
            Err(e) => erros.push(format!("Categoria {} ({}): {}", i + 1, linha.nome.trim(), e.mensagem())),
        }
    }
    if !erros.is_empty() {
        return pagina_novo(&s, campos, linhas, erros);
    }
    match estado.back.eventos.cadastrar(&s, &EventoNovo { dados, categorias }).await {
        Ok(evento) => redirecionar(
            &estado,
            jar,
            &format!("/organizador/eventos/{}", evento.id()),
            Aviso::sucesso(format!(
                "Evento \"{}\" cadastrado (código {}). As vendas já estão abertas.",
                evento.nome(),
                evento.id()
            )),
        ),
        Err(e) => {
            let erros = e.mensagem().split(SEPARADOR_MENSAGENS).map(str::to_string).collect();
            pagina_novo(&s, campos, linhas, erros)
        }
    }
}

fn pagina_editar(s: &Sessao, id: i64, campos: CamposEvento, capacidade_minima: i32, erros: Vec<String>) -> Response {
    render(PaginaFormEvento {
        layout: Layout::novo("Alterar evento", Some(s), None),
        acao: format!("/organizador/eventos/{id}/editar"),
        editando: true,
        campos,
        categorias: vec![],
        erros,
        capacidade_minima,
        voltar: format!("/organizador/eventos/{id}"),
    })
}

pub async fn form_editar(State(estado): State<Estado>, Logado(s): Logado, Path(id): Path<i64>) -> Response {
    let (e, categorias) = tentar!(estado.back.eventos.gerenciar(&s, id).await, s, "/organizador/eventos");
    let campos = CamposEvento {
        nome: e.nome().into(),
        descricao: e.descricao().into(),
        local: e.local().into(),
        data: e.data().format("%Y-%m-%d").to_string(),
        horario: e.horario().format("%H:%M").to_string(),
        capacidade: e.capacidade().to_string(),
        permite_cancelamento: e.permite_cancelamento(),
        vendas_abertas: e.status() == StatusEvento::Aberto,
    };
    pagina_editar(&s, id, campos, soma_categorias(&categorias), vec![])
}

fn soma_categorias(categorias: &[Ingresso]) -> i32 {
    categorias.iter().map(Ingresso::quantidade).sum()
}

/// Altera dados, capacidade e abertura/encerramento das vendas.
pub async fn editar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormEvento>,
) -> Response {
    let status = if f.vendas_abertas.is_some() { StatusEvento::Aberto } else { StatusEvento::Encerrado };
    let resultado = match f.dados(None) {
        Ok(dados) => estado.back.eventos.alterar(&s, id, dados, status).await,
        Err(e) => Err(e),
    };
    match resultado {
        Ok(e) => redirecionar(
            &estado,
            jar,
            &format!("/organizador/eventos/{id}"),
            Aviso::sucesso(format!("Evento \"{}\" alterado.", e.nome())),
        ),
        Err(e) => {
            let minima = match estado.back.eventos.gerenciar(&s, id).await {
                Ok((_, cats)) => soma_categorias(&cats),
                Err(_) => 1,
            };
            pagina_editar(&s, id, f.campos(), minima, vec![e.mensagem().to_string()])
        }
    }
}

/// Algoritmo 6 — relatório, mapa de cada setor com assento marcado e lista de vendas.
pub async fn relatorio(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let relatorio = tentar!(estado.back.relatorios.relatorio(&s, id).await, s, "/organizador/eventos");
    let vendas = tentar!(estado.back.relatorios.vendas_do_evento(&s, id).await, s, "/organizador/eventos");
    let mut mapas = Vec::new();
    for c in relatorio.categorias.iter().filter(|c| c.com_assento()) {
        mapas.push(tentar!(estado.back.eventos.mapa(c).await, s, "/organizador/eventos"));
    }
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let pagina =
        PaginaRelatorio { layout: Layout::novo("Relatório do evento", Some(&s), aviso), relatorio, mapas, vendas };
    (jar, render(pagina)).into_response()
}

pub async fn form_checkin(State(estado): State<Estado>, Logado(s): Logado) -> Response {
    tentar!(estado.back.eventos.organizador_ativo(&s).await, s, "/");
    render(PaginaCheckin { layout: Layout::novo("Check-in", Some(&s), None), codigo: String::new(), resultado: None })
}

#[derive(Deserialize)]
pub struct FormCheckin {
    codigo: String,
}

/// Registro de presença: o número do ingresso é o número da compra.
pub async fn checkin(State(estado): State<Estado>, Logado(s): Logado, Form(f): Form<FormCheckin>) -> Response {
    let numero = f.codigo.trim().trim_start_matches('0');
    let resultado = match numero.parse::<i64>() {
        Ok(id) => match estado.back.relatorios.registrar_presenca(&s, id).await {
            Ok(v) => Aviso::sucesso(format!("Entrada liberada — ingresso nº {:06}.", v.id())),
            Err(e) => Aviso::erro(e.mensagem()),
        },
        Err(_) => Aviso::erro("digite o número do ingresso"),
    };
    render(PaginaCheckin {
        layout: Layout::novo("Check-in", Some(&s), None),
        codigo: String::new(),
        resultado: Some(resultado),
    })
}

/// Check-in direto pela lista de vendas do relatório.
pub async fn checkin_da_lista(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path((id, venda)): Path<(i64, i64)>,
) -> Response {
    let aviso = match estado.back.relatorios.registrar_presenca(&s, venda).await {
        Ok(v) => Aviso::sucesso(format!("Entrada registrada — ingresso nº {:06}.", v.id())),
        Err(e) => Aviso::erro(e.mensagem()),
    };
    redirecionar(&estado, jar, &format!("/organizador/eventos/{id}/relatorio"), aviso)
}

// ------------------------- gerenciar evento -------------------------

/// Categoria digitada no formulário "Adicionar categoria".
#[derive(Deserialize)]
pub struct FormCategoria {
    nome: String,
    preco: String,
    #[serde(default)]
    assento: String,
    #[serde(default)]
    quantidade: String,
    #[serde(default)]
    fileiras: String,
    #[serde(default)]
    por_fileira: String,
}

impl FormCategoria {
    fn linha(self) -> LinhaCategoria {
        LinhaCategoria {
            nome: self.nome,
            preco: self.preco,
            com_assento: self.assento == "sim",
            quantidade: self.quantidade,
            fileiras: self.fileiras,
            por_fileira: self.por_fileira,
        }
    }
}

async fn pagina_gerenciar(
    estado: &Estado,
    s: &Sessao,
    id: i64,
    aviso: Option<Aviso>,
    nova: LinhaCategoria,
    erro_categoria: Option<String>,
) -> Response {
    let (evento, categorias) = tentar!(estado.back.eventos.gerenciar(s, id).await, *s, "/organizador/eventos");
    render(PaginaGerenciarEvento {
        layout: Layout::novo(evento.nome(), Some(s), aviso),
        vagas_para_categorias: evento.capacidade() - soma_categorias(&categorias),
        disponiveis: total_disponivel(&categorias),
        realizado: evento.ja_ocorreu(hoje(estado).await),
        evento,
        categorias,
        nova,
        erro_categoria,
    })
}

/// Página "Gerenciar evento": categorias com vendidos/disponíveis e todas as ações.
pub async fn gerenciar(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
) -> Response {
    let (jar, aviso) = estado.sessoes.retirar_aviso(jar);
    let nova = LinhaCategoria::default();
    (jar, pagina_gerenciar(&estado, &s, id, aviso, nova, None).await).into_response()
}

pub async fn adicionar_categoria(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path(id): Path<i64>,
    Form(f): Form<FormCategoria>,
) -> Response {
    let linha = f.linha();
    let resultado = match categoria_do_formulario(&linha) {
        Ok(cat) => estado.back.eventos.adicionar_categoria(&s, id, &cat).await,
        Err(e) => Err(e),
    };
    match resultado {
        Ok(i) => redirecionar(
            &estado,
            jar,
            &format!("/organizador/eventos/{id}"),
            Aviso::sucesso(format!("Categoria \"{}\" adicionada com {} ingressos.", i.categoria(), i.quantidade())),
        ),
        Err(e) => pagina_gerenciar(&estado, &s, id, None, linha, Some(e.mensagem().to_string())).await,
    }
}

pub async fn form_categoria(
    State(estado): State<Estado>,
    Logado(s): Logado,
    Path((id, categoria)): Path<(i64, i64)>,
) -> Response {
    let voltar = format!("/organizador/eventos/{id}");
    let (evento, ingresso) = tentar!(estado.back.eventos.categoria_do_organizador(&s, id, categoria).await, s, voltar);
    render(PaginaFormCategoria {
        layout: Layout::novo("Editar categoria", Some(&s), None),
        nome: ingresso.categoria().to_string(),
        preco: ingresso.preco().sem_simbolo(),
        quantidade: ingresso.quantidade().to_string(),
        erro: None,
        evento,
        ingresso,
    })
}

#[derive(Deserialize)]
pub struct FormAlterarCategoria {
    nome: String,
    preco: String,
    quantidade: Option<String>,
}

/// Altera nome, preço e (sem assento marcado) a quantidade de ingressos.
pub async fn alterar_categoria(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path((id, categoria)): Path<(i64, i64)>,
    Form(f): Form<FormAlterarCategoria>,
) -> Response {
    let quantidade = f.quantidade.as_deref().filter(|q| !q.trim().is_empty());
    let resultado = async {
        let preco = Dinheiro::de_texto(&f.preco)?;
        let quantidade = quantidade.map(|q| inteiro(q, "quantidade")).transpose()?;
        estado.back.eventos.alterar_categoria(&s, id, categoria, &f.nome, preco, quantidade).await
    }
    .await;
    match resultado {
        Ok(i) => redirecionar(
            &estado,
            jar,
            &format!("/organizador/eventos/{id}"),
            Aviso::sucesso(format!(
                "Categoria \"{}\" atualizada: {} — {} disponíveis de {}.",
                i.categoria(),
                i.preco(),
                i.disponivel(),
                i.quantidade()
            )),
        ),
        Err(e) => {
            let voltar = format!("/organizador/eventos/{id}");
            let (evento, ingresso) =
                tentar!(estado.back.eventos.categoria_do_organizador(&s, id, categoria).await, s, voltar);
            render(PaginaFormCategoria {
                layout: Layout::novo("Editar categoria", Some(&s), None),
                nome: f.nome,
                preco: f.preco,
                quantidade: quantidade.unwrap_or_default().to_string(),
                erro: Some(e.mensagem().to_string()),
                evento,
                ingresso,
            })
        }
    }
}

pub async fn remover_categoria(
    State(estado): State<Estado>,
    Logado(s): Logado,
    jar: CookieJar,
    Path((id, categoria)): Path<(i64, i64)>,
) -> Response {
    let aviso = match estado.back.eventos.remover_categoria(&s, id, categoria).await {
        Ok(()) => Aviso::sucesso("Categoria removida."),
        Err(e) => Aviso::erro(e.mensagem()),
    };
    redirecionar(&estado, jar, &format!("/organizador/eventos/{id}"), aviso)
}

/// Exclui o evento (só sem vendas; com vendas, o organizador encerra as vendas).
pub async fn excluir(State(estado): State<Estado>, Logado(s): Logado, jar: CookieJar, Path(id): Path<i64>) -> Response {
    match estado.back.eventos.remover(&s, id).await {
        Ok(()) => redirecionar(&estado, jar, "/organizador/eventos", Aviso::sucesso("Evento excluído.")),
        Err(e) => redirecionar(&estado, jar, &format!("/organizador/eventos/{id}"), Aviso::erro(e.mensagem())),
    }
}
