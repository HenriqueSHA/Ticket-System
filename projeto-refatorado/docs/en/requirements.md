# Requirements checklist (R0)

Sources: the professor's report template (`documentos/norte/`), course slides and the delivery 2
report (`documentos/entregues/`). Checked on 2026-11-24.

## System requirements (delivery 2, sections 4, 5 and 7)

| # | Requirement | Where it is met | Status |
|---|---|---|---|
| 1 | Register, edit and list organizers | Self-signup (alg. 9) and SuperAdmin panel (alg. 7) | ✅ |
| 2 | Register, edit and list events | Alg. 3 and 8 (`servicos/eventos.rs`) | ✅ |
| 3 | Register customers | Alg. 2 (`cadastrar_cliente`, validated CPF) | ✅ |
| 4 | Ticket categories and types | Categories per event + full/half price (Strategy) | ✅ |
| 5 | Seat management and availability | `MapaAssentos` matrix, FREE/TAKEN status | ✅ |
| 6 | List available events | `/eventos` (open, future, with free tickets) | ✅ |
| 7 | Buy tickets | Alg. 4, database transaction | ✅ |
| 8 | Record sales | `vendas` table | ✅ |
| 9 | Customer purchase history | "Meus ingressos" | ✅ |
| 10 | SuperAdmin manages organizers | Alg. 7 (activate, block, edit, remove) | ✅ |
| 11 | Prevent double selling | Check + transaction + partial unique index | ✅ |
| 12 | Cancellation (if allowed and before the event) | Alg. 5 | ✅ |
| 13 | **Rescheduling** (rules 7.3 and 7.4) | Alg. 5b — **missing; added in this refactor** | ✅ new |
| 14 | Reports: sold, occupancy, attendance, revenue | Alg. 6 + check-in | ✅ |
| 15 | Organizers only see their own events | `evento_do_organizador` | ✅ |
| 16 | Validate required fields before saving | Value Objects + service validation | ✅ |
| 17 | Passwords never shown | Argon2 hash; no DTO exposes it | ✅ |

## Course requirements (report template)

| # | Requirement | Where | Status |
|---|---|---|---|
| 1 | Report sections 1 to 20 | `documentos/relatorio/saida/Equipe 11 - Relatório Final (Entregas 3, 4 e 5).docx` | ✅ |
| 2 | Pseudocode, flowcharts, sequential/conditional/loop structures | Section 8 (9 algorithms, 12 flowcharts) | ✅ |
| 3 | Data structures explained | Section 9 | ✅ |
| 4 | OOP: classes, objects, attributes, methods, relationships | Section 10 + class diagram | ✅ |
| 5 | Technologies and their purpose | Section 11 | ✅ |
| 6 | Implementation with code excerpts tied to the algorithm | Section 12 | ✅ |
| 7 | Persistence: database, DBMS, tables, SQL | Section 13 + ER model | ✅ |
| 8 | Error handling | Section 14 | ✅ |
| 9 | Tests: scenario → input → expected → actual → status (min. 3) | Section 15 (19 scenarios; 55 automated tests) | ✅ |
| 10 | Evidence (screens) and analysis | Section 16 (15 screenshots) + video | ✅ |
| 11 | Limitations, improvements, conclusion, references | Sections 17 to 20 | ✅ |
| 12 | Presentation | `documentos/apresentacao/` (script and slides) | ✅ |
