# 11. Tecnologias utilizadas

| Tecnologia | Versão | Finalidade no projeto |
|---|---|---|
| Rust | 1.94 (edição 2024) | Linguagem do back-end e do front-end; o compilador impede acesso a valor nulo e corrida de dados |
| Cargo | 1.94 | Compilação, dependências e execução dos testes |
| Axum | 0.8 | Servidor HTTP: rotas da API (back) e das páginas (front) |
| Tokio | 1.54 | Execução assíncrona (atende vários usuários ao mesmo tempo) |
| Askama | 0.12 | Modelos HTML verificados na compilação; escapa o texto do usuário (proteção contra XSS) |
| SQLx | 0.8 | Acesso ao PostgreSQL, transações e migrações do banco |
| PostgreSQL | 16 | Sistema gerenciador de banco de dados (SGBD) |
| Argon2 | 0.5 | Hash das senhas (a senha nunca é gravada) |
| Reqwest | 0.12 | Cliente HTTP usado pelo front para chamar o back |
| Serde | 1.0 | Conversão dos objetos para JSON e de volta |
| Chrono | 0.4 | Datas e horários dos eventos e das vendas |
| Docker e Docker Compose | 29 | Três contêineres (front, back e banco) com um comando só |
| GitHub e GitHub Actions | — | Versionamento do código e pipeline de CI/CD (formatação, clippy, testes, imagens) |
| GitHub Container Registry | — | Publicação das imagens do front e do back |
| rustfmt e Clippy | — | Padronização do código e análise estática (modo *pedantic*, avisos tratados como erro) |
| Graphviz | — | Geração dos fluxogramas e diagramas a partir de arquivos de texto |
| Visual Studio Code e Claude Code | — | Edição do código e apoio na revisão, testes e documentação |
| Google Chrome (modo headless) | — | Capturas de tela da aplicação para o relatório |
