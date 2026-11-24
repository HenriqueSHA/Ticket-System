# Sistema de Cadastro e Venda de Ingressos — versão refatorada

[English](README.md) · **Português**

Trabalho A3 de Algoritmos e Programação (UNIFACS 2026.2, Prof. Robson Silva) — Equipe 11:
Fellipe Leonny dos Santos Ribeiro, Henrique Albergaria Santos e Yasmin Oliveira de Souza.

Plataforma web para organizadores cadastrarem eventos e venderem ingressos, com mapa de assentos,
meia-entrada, cancelamento, troca de assento, relatórios com check-in e um SuperAdmin que gerencia
os organizadores. Feita em **Rust**, com **PostgreSQL**, em **3 contêineres** (front, back e banco).

## Executar

```bash
cp .env.example .env
docker compose up -d --build     # abre em http://localhost:3000 (PORTA_WEB)
```

Logins de demonstração: `ana@email.com` / `ana12345` (cliente), `contato@soleventos.com` /
`sol12345` (organizador), `admin@ingressos.com` / `admin123` (SuperAdmin).

Para rodar **ao lado** da versão atual, use outro `PROJETO`, `PREFIXO`, `VOLUME_BANCO` e
`PORTA_WEB` no `.env` (a cópia de teste usa `ticket-system-v2` na porta 3001).

## Pastas

| Pasta | Conteúdo |
|---|---|
| [`codigo/`](codigo/README.md) | Código Rust (6 crates), Dockerfile, migrações e testes |
| `documentos/relatorio/secoes/` | Seções 8 a 20 do relatório em Markdown (fonte) |
| `documentos/relatorio/fluxogramas/` e `diagramas/` | Fluxogramas e diagramas (PNG + fonte `.dot`) |
| `documentos/relatorio/prints/` | Telas da aplicação usadas na seção 16 |
| `documentos/relatorio/saida/` | **Relatório final (.docx)**, seções 1 a 20 |
| `documentos/apresentacao/` | Roteiro e slides da apresentação |
| `docs/pt-BR/`, `docs/en/` | Checklist de requisitos e relatório da refatoração |
| `ferramentas/` | Geração do relatório, dos fluxogramas e dos prints/vídeo |

## Gerar os documentos

```bash
python3 ferramentas/fluxogramas/gerar.py      # fluxogramas e diagramas (Node + Chrome)
PLAYWRIGHT=$(npm root -g)/@playwright/cli/node_modules/playwright-core \
  node ferramentas/demonstracao.mjs http://localhost:3000   # prints (e vídeo, com uma 3ª pasta)
python3 ferramentas/montar_relatorio.py       # relatório final .docx
```

## Qualidade

- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`, com o clippy **pedantic** ligado.
- 55 testes: domínio, cenários (seção 15), PostgreSQL real, API e ponta a ponta.
- O CI roda tudo isso a cada push (`.github/workflows/ci-refatorado.yml`).

Veja o que mudou em [docs/pt-BR/refactoring-report.md](docs/pt-BR/refactoring-report.md) e o
checklist em [docs/pt-BR/requirements.md](docs/pt-BR/requirements.md).
