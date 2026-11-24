# 14. Tratamento de erros e exceções

## 14.1 Estratégia

Rust não tem exceções. Toda operação que pode falhar devolve `Resultado<T>`, que é ou o valor ou um
`Erro`, e o compilador obriga quem chama a tratar os dois casos. Os erros foram divididos em cinco
tipos, cada um com uma resposta diferente:

| Tipo de erro | Quando acontece | Exemplo de mensagem | Resposta HTTP (back) |
|---|---|---|---|
| `Validacao` | Dado digitado inválido | "CPF inválido", "valor "abc" inválido (exemplo: 120,50)" | 422 |
| `NaoEncontrado` | Registro inexistente | "compra não encontrada", "assento Z99 inexistente" | 404 |
| `RegraNegocio` | Operação proibida pela regra | "assento já vendido. Escolha outro", "evento lotado" | 409 |
| `AcessoNegado` | Sem login, perfil errado ou bloqueado | "e-mail ou senha inválidos", "organizador bloqueado. Procure o administrador" | 403 |
| `Infraestrutura` | Falha técnica (banco fora do ar) | "erro interno no servidor. Tente novamente em instantes" | 500 |

## 14.2 Situações tratadas

- **Entrada inválida:** CPF, CNPJ e e-mail são validados ao criar o objeto (Value Object). Preço
  negativo, data passada, capacidade zero, fileira fora de A–Z e senha curta são recusados com a
  mensagem do campo. O formulário volta preenchido para a pessoa corrigir.
- **Valores fora do intervalo:** quantidade de categoria abaixo do já vendido, capacidade menor que
  a soma das categorias, números muito grandes no preço (estouro).
- **Registros inexistentes:** evento, categoria, assento ou compra que não existem, ou que são de
  outra pessoa. Nesse caso a resposta é "não encontrado", para não revelar que o registro existe.
- **Operações incorretas:** comprar assento ocupado, cancelar duas vezes, cancelar ou remarcar depois
  do evento, remover categoria com vendas, check-in de ingresso cancelado.
- **Concorrência:** duas compras do mesmo assento ao mesmo tempo. A transação e o índice único
  garantem que só uma vença. Os erros de restrição do banco (códigos 23505, 23503 e 23514) viram
  `RegraNegocio` com uma mensagem clara.
- **Serviço indisponível:** se o back estiver fora do ar, o front mostra "serviço indisponível no momento. Tente novamente em instantes" em
  vez de travar.
- **Falha técnica:** detalhes internos (como mensagens do banco) nunca aparecem para o usuário; ficam
  só no registro do servidor.
- **Segurança:** a sessão expira em 8 horas; o texto digitado é escapado no HTML (proteção contra
  XSS); as senhas são guardadas só como hash Argon2.
