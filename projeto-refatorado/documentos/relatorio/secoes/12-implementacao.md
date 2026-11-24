# 12. Implementação

## 12.1 Do algoritmo ao programa

Cada algoritmo da seção 8 virou um método de um serviço da camada de aplicação, e cada decisão do
fluxograma virou uma verificação que devolve um erro com a mesma mensagem. O front-end mostra as
páginas e o back-end executa os algoritmos. Os dois conversam por HTTP, e só o back acessa o banco.

| Algoritmo (seção 8) | Onde está no código |
|---|---|
| 1 — Programa principal e login | `aplicacao/src/servicos/autenticacao.rs` (`entrar`), `web/src/rotas/publico.rs` |
| 2 — Cadastrar cliente e validar CPF | `autenticacao.rs` (`cadastrar_cliente`), `dominio/src/valores/documento.rs` |
| 3 — Cadastrar evento e gerar a matriz | `servicos/eventos.rs` (`cadastrar`), `entidades/evento.rs` (`EventoBuilder`) |
| 4 — Comprar ingresso | `servicos/vendas.rs` (`preparar`, `confirmar`), `postgres/vendas.rs` (`registrar`) |
| 5 — Cancelar compra | `servicos/vendas.rs` (`cancelar`), `entidades/venda.rs` (`verificar_cancelamento`) |
| 5b — Remarcar compra | `servicos/vendas.rs` (`remarcar`), `entidades/venda.rs` (`verificar_remarcacao`) |
| 6 — Relatório e check-in | `servicos/relatorios.rs`, `entidades/relatorio.rs` (`calcular`) |
| 7 — Gerenciar organizadores | `servicos/administracao.rs`, `entidades/organizador.rs` |
| 8 — Gerenciar evento | `servicos/eventos.rs` (`alterar`, `adicionar_categoria`, `alterar_categoria`, `remover_categoria`, `remover`) |
| 9 — Autocadastro de organizador | `aplicacao/src/cadastro_organizador.rs` (`registrar`) |
| Mapa de assentos (matriz) | `entidades/assento.rs` (`MapaAssentos::montar` e `linhas`) |

## 12.2 Funcionalidades implementadas

- **Público:** criar conta de cliente (com CPF), criar conta de organizador (com CNPJ numérico ou
  alfanumérico) e entrar no sistema.
- **Cliente:** listar eventos com ingressos disponíveis, comprar (categoria, inteira ou meia,
  assento no mapa e forma de pagamento), ver o comprovante, consultar o histórico, cancelar e trocar
  o assento.
- **Organizador:** cadastrar eventos com várias categorias, alterar dados, capacidade e status,
  adicionar, alterar (preço e quantidade) e remover categorias, excluir evento sem vendas, ver o
  relatório e a lista de vendas e registrar a presença (check-in).
- **SuperAdmin:** cadastrar, listar, ativar, bloquear, editar e remover organizadores.

## 12.3 Trechos relevantes

**Validação do CPF (Alg. 2) — vetor e repetição.** Os pesos decrescentes do pseudocódigo são
aplicados lendo o vetor de trás para frente:

```
let digito = |n: usize| {
    // pesos n+1, n, ..., 2 da esquerda para a direita = 2, 3, ... lendo de trás para frente
    let soma: u32 = d[..n].iter().rev().zip(2u32..).map(|(digito, peso)| digito * peso).sum();
    let resto = (soma * 10) % 11;
    if resto == 10 { 0 } else { resto }
};
digito(9) == d[9] && digito(10) == d[10]
```

**Preço da meia-entrada (Alg. 4) — padrão Strategy.** Cada tipo de ingresso tem a sua política, e
a venda só chama `calcular`:

```
pub const DESCONTO_MEIA: i64 = 50;

impl PoliticaPreco for PrecoMeiaEntrada {
    fn calcular(&self, preco: Dinheiro) -> Dinheiro {
        preco.com_desconto(DESCONTO_MEIA)
    }
}
```

**Verificações da compra (Alg. 4, fluxograma 4).** As mesmas decisões do fluxograma, na mesma ordem:

```
pub async fn preparar(&self, sessao: &Sessao, pedido: &Pedido) -> Resultado<ResumoCompra> {
    sessao.exigir(Perfil::Cliente)?;
    let evento = self.evento_para_venda(pedido.evento_id).await?;        // aberto, futuro, com vagas?
    let ingresso = self.ingresso_para_venda(&evento, pedido.ingresso_id).await?; // categoria esgotada?
    let assento = match (&pedido.assento, ingresso.com_assento()) {
        (Some((fileira, numero)), true) => Some(self.assento_para_venda(&ingresso, fileira, *numero).await?),
        (None, true) => return Err(Erro::validacao("escolha um assento no mapa")),
        (_, false) => None,
    };
    let valor = pedido.tipo.politica().calcular(ingresso.preco());
    ...
}
```

**Remarcação (Alg. 5b).** Reaproveita as regras do cancelamento e a verificação de assento livre da
compra:

```
pub async fn remarcar(&self, sessao: &Sessao, venda_id: i64, fileira: &str, numero: i32) -> Resultado<Venda> {
    sessao.exigir(Perfil::Cliente)?;
    let mut venda = self.vendas.buscar(venda_id).await?.ok_or_else(|| Erro::nao_encontrado("compra não encontrada"))?;
    let evento = ...;
    venda.verificar_remarcacao(sessao.usuario_id, &evento, self.relogio.hoje())?;
    let ingresso = ...;                                       // mesma categoria da compra
    let novo = self.assento_para_venda(&ingresso, fileira, numero).await?;
    self.vendas.remarcar(&venda, novo.id()).await?;           // transação no banco
    venda.trocar_assento(novo.id());
    Ok(venda)
}
```

**Relatório (Alg. 6) — contadores e acumulador.**

```
for v in vendas {
    if v.status() == StatusVenda::Cancelada { cancelados += 1; continue; }
    vendidos += 1;
    receita = receita + v.valor();
    match v.tipo() { TipoIngresso::Inteira => inteiras += 1, TipoIngresso::Meia => meias += 1 }
    if v.status() == StatusVenda::Utilizada { presentes += 1; }
}
let ocupacao = f64::from(vendidos) / f64::from(evento.capacidade()) * 100.0;
```

## 12.4 Interface gráfica

A interface é web e roda no navegador, sem instalar nada no computador do usuário. As páginas são
geradas no servidor com o Askama, e o mapa de assentos é desenhado a partir da matriz. Os assentos
vendidos aparecem com ✕ e não podem ser escolhidos. As telas estão na seção 16.

## 12.5 Execução

```
cp .env.example .env
docker compose up -d --build        # sobe db, back e front
# abrir http://localhost:3000
```
