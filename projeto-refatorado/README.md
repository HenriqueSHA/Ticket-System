# Ticket Registration and Sales System — refactored version

**English** · [Português](README.pt-BR.md)

A3 project for Algorithms and Programming (UNIFACS 2026.2, Prof. Robson Silva) — Team 11:
Fellipe Leonny dos Santos Ribeiro, Henrique Albergaria Santos and Yasmin Oliveira de Souza.

A web platform where organizers create events and sell tickets, with a seat map, half-price
tickets, cancellation, seat changes, reports with check-in, and a SuperAdmin who manages the
organizers. Built in **Rust** with **PostgreSQL**, running in **3 containers** (front, back and database).

## Run

```bash
cp .env.example .env
docker compose up -d --build     # opens at http://localhost:3000 (PORTA_WEB)
```

Demo logins: `ana@email.com` / `ana12345` (customer), `contato@soleventos.com` / `sol12345`
(organizer), `admin@ingressos.com` / `admin123` (SuperAdmin).

To run it **next to** the current version, set a different `PROJETO`, `PREFIXO`, `VOLUME_BANCO`
and `PORTA_WEB` in `.env` (the test copy uses `ticket-system-v2` on port 3001).

## Folders

| Folder | Contents |
|---|---|
| [`codigo/`](codigo/README.md) | Rust code (6 crates), Dockerfile, migrations and tests |
| `documentos/relatorio/secoes/` | Report sections 8 to 20 in Markdown (source) |
| `documentos/relatorio/fluxogramas/` and `diagramas/` | Flowcharts and diagrams (PNG + `.dot` source) |
| `documentos/relatorio/prints/` | App screenshots used in section 16 |
| `documentos/relatorio/saida/` | **Final report (.docx)**, sections 1 to 20 |
| `documentos/apresentacao/` | Presentation script and slides |
| `docs/en/`, `docs/pt-BR/` | Requirements checklist and refactoring report |
| `ferramentas/` | Report, flowchart and screenshot/video generators |

## Build the documents

```bash
python3 ferramentas/fluxogramas/gerar.py      # flowcharts and diagrams (Node + Chrome)
PLAYWRIGHT=$(npm root -g)/@playwright/cli/node_modules/playwright-core \
  node ferramentas/demonstracao.mjs http://localhost:3000   # screenshots (and video, with a 3rd folder)
python3 ferramentas/montar_relatorio.py       # final .docx report
```

## Quality

- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`, with clippy **pedantic** enabled.
- 55 tests: domain, scenarios (section 15), real PostgreSQL, API and end-to-end.
- CI runs all of them on every push (`.github/workflows/ci-refatorado.yml`).

See what changed in [docs/en/refactoring-report.md](docs/en/refactoring-report.md) and the
checklist in [docs/en/requirements.md](docs/en/requirements.md).

The code, the report and the UI are in Portuguese, the language of the course.
