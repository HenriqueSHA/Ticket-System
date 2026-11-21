# A3 — Sistema de Cadastro e Venda de Ingressos (Equipe 11)

Trabalho A3 de **Algoritmos e Programação** — UNIFACS 2026.2 — Prof. Robson Silva.

**Equipe:** Fellipe Leonny dos Santos Ribeiro · Henrique Albergaria Santos · Yasmin Oliveira de Souza

Sistema web em **Rust** com banco **PostgreSQL** para cadastro de eventos e venda de ingressos,
com três perfis: cliente, organizador e SuperAdmin.

## Como rodar

```bash
cp .env.example .env
docker compose up -d --build     # sobe tudo em contêineres
```

Abra **http://localhost:3000**. Logins de demonstração e detalhes técnicos em
[`codigo/README.md`](codigo/README.md).

## Organização desta pasta

| Pasta / arquivo | Conteúdo |
|-----------------|----------|
| [`codigo/`](codigo/) | código-fonte (workspace Rust, Dockerfile, migrações do banco, testes) |
| [`docker-compose.yml`](docker-compose.yml) · `.env.example` | execução em contêineres (servidor ou máquina local) |
| [`documentos/norte/`](documentos/norte/) | material de referência fornecido pelo professor |
| [`documentos/entregues/`](documentos/entregues/) | documentos já entregues (Entrega 2) |
| [`documentos/relatorio/secoes/`](documentos/relatorio/secoes/) | texto-fonte das seções do relatório (Markdown) |
| [`documentos/relatorio/fluxogramas/`](documentos/relatorio/fluxogramas/) | fluxogramas (fonte `.dot` em `fonte/` e imagens `.png`) |
| [`documentos/relatorio/prints/`](documentos/relatorio/prints/) | capturas de tela do sistema |
| [`documentos/relatorio/saida/`](documentos/relatorio/saida/) | relatórios `.docx` gerados |
| [`ferramentas/`](ferramentas/) | scripts que geram fluxogramas e montam o relatório |
| [`PLANO.md`](PLANO.md) | plano de trabalho com checklist das fases |

## Entregas

| Etapa | Peso | Data | Situação |
|-------|------|------|----------|
| 1. Definição do problema | 10% | 21/09/2026 | entregue |
| 2. Modelagem da solução | 20% | 05/10/2026 | entregue (`documentos/entregues/`) |
| 3. Projeto do algoritmo | 20% | junto com a 4 | seção 8 em `documentos/relatorio/` |
| 4. Implementação e testes | 25% | 16/11/2026 | código em `codigo/`; seções 9–15 em andamento |
| 5. Solução final e apresentação | 25% | 30/11/2026 | a fazer |

## Gerar os documentos

```bash
npm install --prefix ferramentas/fluxogramas      # uma vez
python3 ferramentas/fluxogramas/gerar.py           # .dot → .png
pip install pillow                                 # uma vez
python3 ferramentas/montar_relatorio.py            # monta o .docx em documentos/relatorio/saida/
```
