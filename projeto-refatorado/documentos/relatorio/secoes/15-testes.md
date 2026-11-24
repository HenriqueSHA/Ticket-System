# 15. Testes realizados

## 15.1 Como os testes foram feitos

Os testes são **automáticos**: escritos em Rust e executados com `cargo test`. Eles rodam em todo
envio de código para o GitHub (pipeline de CI), e uma alteração que quebre qualquer teste é barrada.
Os cenários usam os dados de demonstração e um relógio fixo em 21/11/2026 12:00, para que o
resultado seja sempre o mesmo:

- "Show de Rock" (daqui a 20 dias, permite cancelamento): Pista R$ 120,00 (sem assento), Cadeira
  R$ 180,00 (5 × 8 assentos) e Camarote R$ 350,00 (2 × 10);
- "Noite do Stand-up" (daqui a 35 dias, **não** permite cancelamento): Plateia R$ 80,00 (6 × 10);
- vendas já feitas: Ana tem Cadeira A4 (inteira) e A5 (meia); Bruno tem Pista (meia), Camarote B3
  e Plateia C6.

| Conjunto | Quantidade | O que verifica |
|---|---|---|
| Unidade (domínio) | 17 | CPF, CNPJ, e-mail, dinheiro, meia-entrada, matriz de assentos, regras do evento e da categoria |
| Cenários (aplicação) | 20 | Os algoritmos completos, com os repositórios em memória |
| Integração com o PostgreSQL | 1 (com várias etapas) | Migrações, transações e índice único no banco real |
| API (back) | 9 | Rotas HTTP, permissões por perfil, expiração do login, erros sem detalhes internos |
| Ponta a ponta (front + back) | 6 | Navegação pelas telas como um navegador, incluindo proteção contra XSS |
| Outros (infraestrutura) | 2 | Hash Argon2 e repositório em memória |
| **Total** | **55** | **Todos aprovados** |

## 15.2 Cenários (Cenário → Entrada → Resultado esperado → Resultado obtido → Situação)

| Nº | Cenário | Entrada | Resultado esperado | Resultado obtido | Situação |
|---|---|---|---|---|---|
| 1 | **Normal** — compra com assento marcado | Ana: Show de Rock, Cadeira, inteira, assento B2, PIX | Venda de R$ 180,00; B2 ocupado; disponível da Cadeira passa de 38 para 37 | Venda CONFIRMADA de R$ 180,00; B2 ocupado no mapa; 37 disponíveis | Aprovado |
| 2 | **Alternativo** — meia-entrada sem assento e cancelamento | Bruno: Pista (R$ 120,00), meia, sem assento; depois cancela | Venda de R$ 60,00; ao cancelar, status CANCELADA e a vaga volta | R$ 60,00; após o cancelamento a quantidade voltou ao valor anterior e o relatório conta 1 cancelado | Aprovado |
| 3 | **Alternativo** — cancelamento libera o assento | Ana cancela a A4; Bruno compra a A4 | A compra do Bruno é aceita | Venda do Bruno registrada na A4 | Aprovado |
| 4 | **Alternativo** — remarcação (troca de assento) | Ana troca a A4 pela B2 | B2 ocupado, A4 livre; valor e quantidade iguais | Comprovante mostra B2; Bruno compra a A4 em seguida | Aprovado |
| 5 | **Excepcional** — assento já vendido | Bruno tenta a A4 (da Ana) | Erro "assento já vendido"; nada gravado | Erro exibido; disponível e histórico iguais aos de antes | Aprovado |
| 6 | **Excepcional** — assento inexistente / categoria de outro evento | Assento Z99; categoria do Stand-up no Show | Erro "não encontrado" | Erro "não encontrado" nos dois casos | Aprovado |
| 7 | **Excepcional** — cadastro de cliente inválido | CPF 989.989.989-43; nome vazio; e-mail sem @; senha "123"; e-mail e CPF já usados | Cada caso recusado com a mensagem do campo; depois, cadastro válido e login | 6 recusas corretas; o cadastro válido entra em seguida | Aprovado |
| 8 | **Excepcional** — evento inválido | Data de hoje; capacidade 0; categoria com quantidade 0 ou 27 fileiras; preço "-10"; categoria de 101 lugares em evento de 100; cliente tentando cadastrar | Todos recusados | Todos recusados; o evento válido (Pista 60 + Cadeira 4 × 10) é gravado | Aprovado |
| 9 | **Excepcional** — evento lotado | Comprar quando o total vendido = capacidade | Erro "evento lotado" | Erro exibido; nada gravado | Aprovado |
| 10 | **Excepcional** — cancelamentos proibidos | Stand-up (não permite); compra de outra pessoa; cancelar 2 vezes; depois do evento | Todos recusados | 4 recusas com as mensagens certas | Aprovado |
| 11 | **Excepcional** — remarcações proibidas | Assento ocupado; assento inexistente; Stand-up; compra sem assento (Pista); compra de outra pessoa | Todas recusadas; o assento continua o mesmo | 5 recusas; comprovante continua com A4 | Aprovado |
| 12 | **Excepcional** — login inválido e organizador bloqueado | Senha errada; organizador bloqueado pelo SuperAdmin | Acesso negado | Acesso negado nos dois casos | Aprovado |
| 13 | **Excepcional** — controle de acesso | Cliente abre relatório ou lista de organizadores; organizador tenta comprar; outro organizador abre o relatório; SuperAdmin remove organizador com vendas | Acesso negado, não encontrado ou regra | Todas as 5 tentativas recusadas | Aprovado |
| 14 | **Normal** — organizador cria conta e vende | Autocadastro com CNPJ 33.444.555/0001-81; repetir o CNPJ; CNPJ inválido; criar evento; cliente compra | Conta criada; repetição e CNPJ inválido recusados; venda aparece no relatório | Conforme o esperado (1 vendido no relatório). O CNPJ alfanumérico 12.ABC.345/01DE-35 é testado no cadastro pelas telas | Aprovado |
| 15 | **Alternativo** — gestão de categorias | Pista vira "Pista Premium", R$ 130,00 e 90 lugares; quantidade 0; quantidade 101 em evento de 100 | Alteração gravada (89 disponíveis); 0 e 101 recusados; próxima venda sai por R$ 130,00 | Conforme o esperado | Aprovado |
| 16 | **Normal** — relatório e check-in | Organizador abre o relatório do Show e registra a presença da A4 duas vezes | 4 vendidos (2 inteiras, 2 meias), R$ 680,00, ocupação 2,5%; segundo check-in recusado | Conforme o esperado; presentes = 1 | Aprovado |
| 17 | **Excepcional** — concorrência no banco | Duas compras do mesmo assento no PostgreSQL | Só uma é gravada | A segunda recebe "assento já vendido" | Aprovado |
| 18 | **Excepcional** — texto malicioso | Razão social com `<script>alert(2)</script>` | O texto aparece escapado, sem executar | HTML mostra `&lt;script&gt;` | Aprovado |
| 19 | **Excepcional** — back fora do ar | Front tenta entrar com o back desligado | Mensagem amigável, sem travar | "serviço indisponível no momento" | Aprovado |

## 15.3 Como reproduzir

```
cd codigo
cargo test                                             # 54 testes, sem banco
docker compose up -d db                                # (na raiz) sobe o PostgreSQL
cargo test -p aplicacao --test postgres -- --ignored   # integração com o banco real
```
