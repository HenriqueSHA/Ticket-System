# Refactoring report (2026-11-24)

Comparison between the previous version (`codigo/` and `documentos/` at the repository root) and the
refactored version (`projeto-refatorado/`). The previous version was not changed.

| Area | Before | After |
|---|---|---|
| Delivery 2 requirements | Rescheduling (7.3/7.4) was missing | Rescheduling implemented (alg. 5b) with page, API and tests |
| Automated tests | 52 + 1 integration (README said 46, outdated) | 54 + 1 integration (55), plus new cases in existing tests |
| Static analysis | Default clippy | Clippy **pedantic** on the whole workspace, zero warnings |
| Money | Parsed through `f64` ("1.234" became R$ 1.23) | Integer arithmetic only; ambiguous input rejected |
| Numeric casts | Unchecked `as` in seat map and CPF | `try_from` with checked bounds |
| Event form | 2nd row pre-filled with "Cadeira" blocked the form | Blank rows with placeholder examples; regression test |
| Docker | Fixed names (one copy per machine) | `PROJETO`, `PREFIXO`, `VOLUME_BANCO`: new version runs side by side on port 3001 |
| CI | Only `codigo/` | New `ci-refatorado.yml` workflow |
| Pseudocode | Algorithms 1 to 7 | + 5b (reschedule), 8 (manage event), 9 (organizer self-signup) |
| Flowcharts | 9 | 12 (3 new + main program updated) |
| Diagrams | None | Architecture (3 containers), classes and ER model |
| Report | Delivery 3 (sections 1 to 8) | Final report, sections 1 to 20, 30 figures |
| Evidence | No screenshots of the web version | 15 screenshots + 27 s video from an automated script |
| Documentation | Technical README in Portuguese | Bilingual README, requirements checklist, this report, CHANGELOG |

## How to compare

```
git diff --no-index codigo projeto-refatorado/codigo
http://localhost:3000 (current)   ×   http://localhost:3001 (refactored)
```

## Unchanged

Architecture (6 crates, 3 containers), database schema (same migrations) and visual identity. The
current version's data volume `ticket-system-dados` was not touched; the new version uses
`ticket-system-v2-dados`.
