# Relatório da refatoração (24/11/2026)

Comparação entre a versão anterior (pasta `codigo/` e `documentos/` na raiz) e a versão refatorada
(`projeto-refatorado/`). A versão antiga não foi alterada.

## Resumo

| Área | Antes | Depois |
|---|---|---|
| Requisitos da Entrega 2 | Remarcação (7.3/7.4) não existia | Remarcação implementada (Alg. 5b), com tela, API e testes |
| Testes automáticos | 52 + 1 de integração (o README dizia 46, desatualizado) | 54 + 1 de integração (55), com novos casos em testes existentes |
| Análise estática | clippy padrão | clippy **pedantic** em todo o workspace, sem nenhum aviso |
| Dinheiro | Texto convertido com `f64` ("1.234" virava R$ 1,23) | Só aritmética inteira; entrada ambígua é recusada |
| Conversões numéricas | `as` sem verificação no mapa de assentos e no CPF | `try_from` com limites conferidos |
| Formulário de evento | A 2ª linha vinha preenchida com "Cadeira" e bloqueava o cadastro | Linhas em branco com exemplo no placeholder; teste de regressão |
| Docker | Nomes fixos (só uma cópia por máquina) | `PROJETO`, `PREFIXO`, `VOLUME_BANCO`: a versão nova roda ao lado na porta 3001 |
| CI | Só a pasta `codigo/` | Novo workflow `ci-refatorado.yml` para a pasta nova |
| Pseudocódigo | Algoritmos 1 a 7 | + 5b (remarcar), 8 (gerenciar evento), 9 (autocadastro); Alg. 1 com a opção nova |
| Fluxogramas | 9 | 12 (3 novos + o principal atualizado) |
| Diagramas | Nenhum | Arquitetura (3 contêineres), classes e modelo ER |
| Relatório | Entrega 3 (seções 1 a 8) | Relatório final com as seções 1 a 20 e 30 figuras |
| Evidências | Nenhum print da versão web | 15 prints + vídeo de 27 s gerados por roteiro automático |
| Documentação | README técnico em português | README em português e inglês, checklist de requisitos, este relatório e CHANGELOG |

## Código

- **Remarcação (Algoritmo 5b):**
  - `Venda::verificar_remarcacao`, que divide as regras com o cancelamento;
  - `RepositorioVendas::remarcar`, uma transação no PostgreSQL e na memória;
  - `ServicoVendas::remarcar`;
  - rota `POST /api/compras/{id}/remarcacao`;
  - página `/compras/{id}/remarcar`, com o mapa da mesma categoria;
  - botão "Trocar assento" em "Meus ingressos".
- **Qualidade:**
  - `[workspace.lints]` com o clippy pedantic;
  - correções automáticas (`Self`, closures redundantes, `must_use`);
  - correções manuais (conversões, passagem por valor);
  - exceções só locais e comentadas.
- **Dinheiro:**
  - `Dinheiro::de_texto` reescrito com inteiros;
  - `com_desconto` recebe o percentual inteiro (`DESCONTO_MEIA = 50`);
  - novos testes para três casas decimais, estouro e ",5".
- **Front:**
  - o formulário de novo evento não vem mais com nomes de categoria preenchidos;
  - o `README` técnico corrigido (o nome do serviço `app` não existe mais; os comandos agora usam `back`/`front`).

## Documentos

- Seção 8:
  - Algoritmos 5b, 8 e 9;
  - menu do Algoritmo 1 atualizado;
  - constante `DESCONTO_MEIA` em percentual;
  - nota sobre a interface web;
  - subseções renumeradas (8.1 a 8.12).
- Seções 9 a 20 escritas do zero, com base no código real. Os números citados (testes, limites,
  mensagens de erro) foram conferidos no código.
- Seção 7.3 ajustada no docx: o autocadastro de organizador passa a constar na regra do SuperAdmin.
- O gerador `ferramentas/montar_relatorio.py`:
  - monta as seções 8 a 20 a partir dos arquivos `.md`;
  - numera as figuras automaticamente;
  - gira os diagramas muito largos.
- `ferramentas/demonstracao.mjs` refaz os prints e o vídeo sempre que a interface mudar.

## Como comparar

```
git diff --no-index codigo projeto-refatorado/codigo      # código
http://localhost:3000   (versão atual)   ×   http://localhost:3001   (versão refatorada)
```

## O que não mudou

- Arquitetura (6 crates, 3 contêineres), banco de dados (mesmas migrações) e identidade visual.
- Os dados da versão atual: o volume `ticket-system-dados` não foi tocado. A versão nova usa
  `ticket-system-v2-dados`.
