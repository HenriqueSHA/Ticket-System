# Checklist de requisitos (R0)

Fontes: modelo de relatório do professor (`documentos/norte/002Modeloderelatoriofinal...docx`), slides
da disciplina e o relatório da Entrega 2 (`documentos/entregues/`). Data da verificação: 24/11/2026.

## Requisitos do sistema (Entrega 2, seções 4, 5 e 7)

| # | Requisito | Onde é atendido | Situação |
|---|---|---|---|
| 1 | Cadastro, alteração e consulta de organizadores | Autocadastro (Alg. 9) e painel do SuperAdmin (Alg. 7) | ✅ |
| 2 | Cadastro, alteração e consulta de eventos | Alg. 3 e 8 (`servicos/eventos.rs`) | ✅ |
| 3 | Cadastro de clientes | Alg. 2 (`cadastrar_cliente`, CPF validado) | ✅ |
| 4 | Categorias e tipos de ingresso | Categorias por evento + inteira/meia (Strategy) | ✅ |
| 5 | Gerenciamento de assentos e disponibilidade | Matriz `MapaAssentos`, situação LIVRE/OCUPADO | ✅ |
| 6 | Consulta de eventos disponíveis | `/eventos` (só eventos abertos, futuros e com vaga) | ✅ |
| 7 | Compra de ingressos | Alg. 4, transação no banco | ✅ |
| 8 | Registro das vendas | Tabela `vendas` | ✅ |
| 9 | Histórico de compras do cliente | "Meus ingressos" | ✅ |
| 10 | SuperAdmin gerencia organizadores | Alg. 7 (ativar, bloquear, editar, remover) | ✅ |
| 11 | Impedir venda duplicada | Verificação + transação + índice único parcial | ✅ |
| 12 | Cancelamento (se o evento permitir e não tiver ocorrido) | Alg. 5 | ✅ |
| 13 | **Remarcação** (regras 7.3 e 7.4) | Alg. 5b — **faltava; implementada na refatoração** | ✅ novo |
| 14 | Relatórios: vendidos, ocupação, presença, receita | Alg. 6 + check-in | ✅ |
| 15 | Cada organizador só vê os próprios eventos | `evento_do_organizador` | ✅ |
| 16 | Validar campos obrigatórios antes de gravar | Value Objects + validação dos serviços | ✅ |
| 17 | Senhas nunca exibidas | Hash Argon2; nenhum DTO expõe o hash | ✅ |

## Requisitos do trabalho (modelo do professor)

| # | Requisito | Onde | Situação |
|---|---|---|---|
| 1 | Seções 1 a 20 do relatório | `documentos/relatorio/saida/Equipe 11 - Relatório Final (Entregas 3, 4 e 5).docx` | ✅ |
| 2 | Pseudocódigo, fluxogramas e estruturas sequenciais, condicionais e de repetição | Seção 8 (9 algoritmos, 12 fluxogramas) | ✅ |
| 3 | Estruturas de dados explicadas (vetor, matriz, listas...) | Seção 9 | ✅ |
| 4 | POO: classes, objetos, atributos, métodos, relacionamentos | Seção 10 + diagrama de classes | ✅ |
| 5 | Tecnologias com a finalidade de cada uma | Seção 11 | ✅ |
| 6 | Implementação com trechos de código ligados ao algoritmo | Seção 12 | ✅ |
| 7 | Persistência: banco, SGBD, tabelas, SQL | Seção 13 + modelo ER | ✅ |
| 8 | Tratamento de erros | Seção 14 | ✅ |
| 9 | Testes: cenário → entrada → esperado → obtido → situação (mín. 3) | Seção 15 (19 cenários; 55 testes automáticos) | ✅ |
| 10 | Evidências (telas) e análise | Seção 16 (15 prints) + vídeo | ✅ |
| 11 | Limitações, melhorias, conclusão e referências | Seções 17 a 20 | ✅ |
| 12 | Apresentação | `documentos/apresentacao/` (roteiro e slides) | ✅ |

## Requisitos combinados com a equipe

| Requisito | Situação |
|---|---|
| Rust + PostgreSQL + POO + interface gráfica (web) | ✅ |
| SOLID e padrões de projeto | ✅ (seção 10.4 e 10.5) |
| Coerência com os fluxogramas | ✅ (tabela algoritmo × código, seção 12.1) |
| 3 contêineres: front, back e banco | ✅ |
| Testes no CI/CD do GitHub | ✅ (`ci-cd.yml` e `ci-refatorado.yml`) |
