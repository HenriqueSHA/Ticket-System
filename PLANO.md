# Plano de trabalho — A3 Equipe 11 (Sistema de Ingressos)

> Arquivo de retomada: marque `[x]` ao concluir. Cada etapa termina com **commit + push**
> (quando mexer no código) e com a checklist dela conferida. Ordem pensada para não refazer
> trabalho: primeiro a arquitetura final, depois os documentos que a descrevem.

**Situação em 21/11/2026:** sistema web em Rust + PostgreSQL funcionando em 2 contêineres
(`app` = interface + regras juntas, `postgres`). Entregas 1, 2 e 3 feitas; seções 9–15
pendentes; Entrega 5 em **30/11/2026**.

---

## Fase 0 — Preparação ✅
- [x] 0.1 Parar contêineres que não são do projeto (n8n-upwork, yellowisland_web, buildkit — só parados, `docker start <nome>` religa)
- [x] 0.2 Escrever este plano
- [x] 0.3 Decisões do grupo (ver "Decisões" no fim)

## Fase 1 — Organizar a pasta A3 desde a raiz ✅
Estrutura alvo:
```
A3/
├── README.md                 índice: o que é cada pasta e como gerar tudo
├── PLANO.md                  este arquivo
├── .github/workflows/        CI/CD (GitHub Actions)
├── docker-compose.yml        front + back + banco (servidor: `docker compose up -d`)
├── .env.example              configuração
├── codigo/                   workspace Rust (crates + Dockerfile + migrations)
├── documentos/
│   ├── norte/                material do professor
│   ├── entregues/            documentos já entregues (Entrega 2, Entrega 3)
│   └── relatorio/
│       ├── secoes/           texto-fonte de cada seção (08-algoritmo.md, 09-…md, …)
│       ├── fluxogramas/      fonte .dot + imagens .png
│       ├── prints/           telas da versão web
│       └── saida/            .docx gerados
└── ferramentas/              scripts (montar relatório, gerar prints, gerar fluxogramas)
```
- [x] 1.1 Criar a estrutura e mover os arquivos (sem perder nada); o `.git` sobe de `codigo/` para a raiz `A3/` mantendo o histórico
- [x] 1.2 Remover só o que é lixo comprovado: PDF duplicado (idêntico, conferido por `cmp`) e prints da versão de terminal (obsoleta)
- [x] 1.3 Ajustar caminhos dos scripts (`montar_relatorio.py`, `gerar.py` dos fluxogramas)
- [x] 1.4 README.md da raiz
- **Checklist:** scripts rodam nos caminhos novos ✔ · nada de `entrega3/`, `entrega4/`, `entregas/` soltos na raiz ✔
  · dados do banco migrados para o volume `ticket-system-dados` ✔ (o volume antigo `codigo_dados-postgres` ficou como backup)

## Fase 2 — Arquitetura em 3 contêineres (front · back · banco) ✅
```
navegador ──► front (web, :3000) ──HTTP/JSON──► back (api, :8080, só rede interna) ──► db (PostgreSQL)
```
- [x] 2.1 Crate `contrato`: tipos JSON (DTOs) compartilhados por front e back
- [x] 2.2 Crate `api` (back): REST JSON com Axum sobre os casos de uso existentes; autenticação por token (back emite, front guarda em cookie); erros com código HTTP certo
- [x] 2.3 Testes da API (rotas, códigos HTTP, permissões) com repositórios em memória
- [x] 2.4 Front: trocar a chamada direta à `Aplicacao` por um cliente HTTP (`trait` + implementação reqwest) — as telas continuam as mesmas
- [x] 2.5 Dockerfile com um alvo por serviço (`api`, `web`) + `docker-compose.yml` com `front`, `back`, `db` (back e db sem porta pública)
- [x] 2.6 Subir os 3 contêineres, healthchecks, testar todos os fluxos pelo navegador
- **Checklist:** `docker compose ps` mostra 3 serviços healthy ✔ · front não acessa o banco (nem resolve o nome `db`) ✔ · back sem porta pública ✔ · 46 testes passam ✔ · commit/push ✔

## Fase 3 — Qualidade, testes e CI/CD
- [ ] 3.1 Revisão do código (code review): erros, duplicações, otimizações (ex.: N+1 consultas na lista de eventos, índices no banco)
- [ ] 3.2 Corrigir o que a revisão achar + testes que provem cada correção
- [ ] 3.3 Testes que faltam: front (renderização/rotas com back falso), API, PostgreSQL
- [ ] 3.4 `cargo fmt` em tudo + configuração (`rustfmt.toml`)
- [ ] 3.5 GitHub Actions — **CI** em todo push/PR: fmt, clippy (sem avisos), testes, testes no PostgreSQL (serviço do Actions), build das imagens Docker
- [ ] 3.6 GitHub Actions — **CD** na `main`: publicar as imagens `front` e `back` no GitHub Container Registry; servidor atualiza com `docker compose pull && docker compose up -d`
- [ ] 3.7 Ver a pipeline verde no GitHub
- **Checklist:** badge de CI no README · pipeline verde · imagens publicadas

## Fase 4 — Documentos coerentes com o sistema final
- [ ] 4.1 Revisar seções 1–7 (Entrega 2) contra o sistema: tabela 7.2 (preço em centavos, novos campos), regra 7.3 "só SuperAdmin cria organizador" → autocadastro, 7.4/7.5
- [ ] 4.2 Atualizar pseudocódigo (seção 8): Alg. 1 com interface web, Alg. 7 com autocadastro, **novo algoritmo de gestão do evento** (editar categoria/preço/quantidade/capacidade)
- [ ] 4.3 Fluxogramas: refazer os que mudaram + novos (cadastro de organizador, gestão de categoria) + diagrama da arquitetura (3 contêineres) e diagrama de classes
- [ ] 4.4 Escrever seções 9–15: estruturas de dados · organização/POO/SOLID/padrões · tecnologias · implementação (trechos de código) · persistência (tabelas/SQL/transações) · erros e exceções · testes (cenário → entrada → esperado → obtido → situação)
- [ ] 4.5 Prints reais da versão web (puppeteer) para as seções 12, 15 e 16
- [ ] 4.6 Montar o .docx "Entregas 3 e 4" (seções 1–15) no padrão do modelo do professor
- [ ] 4.7 Revisão final de coerência: nomes, números de figura, testes citados = testes existentes
- **Checklist:** todo algoritmo citado existe no código · toda tela citada tem print · docx abre sem erro

## Fase 5 — Entrega 5 (30/11)
- [ ] 5.1 Seções 16–20 (resultados, limitações, melhorias, conclusão, referências)
- [ ] 5.2 Apresentação (slides) + roteiro da demonstração
- [ ] 5.3 Ensaio da demo do zero (`docker compose up`)

---

## Decisões (21/11/2026)
1. **Front:** telas atuais em Rust (crate `web`), falando com o back só por HTTP/JSON.
2. **Repositório:** a pasta **A3 inteira** vira o repositório (código + documentos), público.
   A estrutura local e a do GitHub são a mesma.
3. **CD:** publicar imagens no GitHub Container Registry; job de deploy por SSH já pronto,
   mas só roda quando os *secrets* do servidor (AWS) forem cadastrados no GitHub.
