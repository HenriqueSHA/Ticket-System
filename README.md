# Sistema de Cadastro e Venda de Ingressos — Equipe 11

Trabalho A3 de **Algoritmos e Programação** (UNIFACS 2026.2).
Aplicação web em **Rust** com banco de dados **PostgreSQL**.

## Como executar

Pré-requisitos: [Rust](https://rustup.rs) e [Docker](https://docs.docker.com/get-docker/).

```bash
cd codigo
docker compose up -d     # sobe o PostgreSQL (porta 5433)
cargo run                # compila e inicia o sistema
```

Abra **http://127.0.0.1:3000** no navegador. Na primeira execução as tabelas são criadas
(`migrations/`) e o banco recebe dados de demonstração:

| Perfil      | E-mail                   | Senha     |
|-------------|--------------------------|-----------|
| SuperAdmin  | admin@ingressos.com      | admin123  |
| Organizador | contato@soleventos.com   | sol12345  |
| Organizador | arena@cultural.com       | arena123  |
| Cliente     | ana@email.com            | ana12345  |
| Cliente     | bruno@email.com          | bruno123  |

Para zerar o banco: `docker compose down -v && docker compose up -d`.
Configurações (endereço do banco e porta da web) ficam no arquivo `.env` (modelo em `.env.example`).

## Testes

```bash
cargo test                  # 25 testes (domínio + cenários com repositórios em memória)
cargo test -- --ignored     # + teste de integração no PostgreSQL real (precisa do docker)
```

Os cenários pedidos na seção 15 do relatório estão em `crates/aplicacao/tests/cenarios.rs`:
**Teste 1** (normal — compra com assento), **Teste 2** (alternativo — meia-entrada sem assento e
cancelamento) e **Teste 3** (excepcionais — assento já vendido, evento lotado, dados inválidos,
organizador bloqueado, controle de acesso, cancelamento proibido).

## Arquitetura (camadas)

```
codigo/
├── docker-compose.yml        PostgreSQL 16
├── migrations/               SQL das tabelas (executado automaticamente)
└── crates/
    ├── dominio/              regras de negócio puras — não depende de nada
    │   ├── valores/          Cpf, Cnpj, Email, Dinheiro (só existem se forem válidos)
    │   ├── entidades/        Usuario, Organizador, Evento, Ingresso, Assento, Venda, MapaAssentos...
    │   ├── precos.rs         Strategy: inteira / meia-entrada
    │   └── portas.rs         interfaces (traits) dos repositórios, hash e relógio
    ├── aplicacao/            casos de uso = Algoritmos 1 a 7 do relatório
    │   ├── servicos/         autenticacao, eventos, vendas, relatorios, administracao
    │   ├── fabrica.rs        Factory de usuários
    │   └── lib.rs            Facade `Aplicacao` + injeção de dependências
    ├── infraestrutura/       implementações das portas
    │   ├── postgres/         repositórios PostgreSQL (sqlx) com transações
    │   ├── memoria.rs        repositórios em memória (testes)
    │   ├── seguranca.rs      hash de senha Argon2
    │   └── relogio.rs        relógio do sistema / relógio fixo
    └── web/                  interface gráfica (Axum + templates HTML/CSS)
        ├── src/rotas/        páginas de cada perfil
        ├── templates/        HTML (Askama, verificado na compilação)
        └── static/           CSS
```

A dependência sempre aponta para dentro: `web → aplicacao → dominio ← infraestrutura`.

### Algoritmos do relatório × código

| Algoritmo (seção 8)                | Onde está                                                         |
|------------------------------------|-------------------------------------------------------------------|
| 1 — Programa principal / login     | `aplicacao/src/servicos/autenticacao.rs` (`entrar`), `web/src/rotas/publico.rs` |
| 2 — Cadastrar cliente + validar CPF| `autenticacao.rs` (`cadastrar_cliente`), `dominio/src/valores/documento.rs` |
| 3 — Cadastrar evento               | `servicos/eventos.rs`, `dominio/src/entidades/evento.rs` (`EventoBuilder`) |
| 4 — Comprar ingresso               | `servicos/vendas.rs` (`preparar`, `confirmar`), `postgres/vendas.rs` (`registrar`) |
| 5 — Cancelar compra                | `servicos/vendas.rs` (`cancelar`), `entidades/venda.rs` (`verificar_cancelamento`) |
| 6 — Relatório e check-in           | `servicos/relatorios.rs`, `entidades/relatorio.rs` (`calcular`)   |
| 7 — Gerenciar organizadores        | `servicos/administracao.rs`, `entidades/organizador.rs`           |
| Mapa de assentos (matriz)          | `entidades/assento.rs` (`MapaAssentos`)                           |

### SOLID

- **S** — cada serviço cuida de um grupo de algoritmos; cada repositório, de uma tabela/agregado.
- **O** — novos tipos de ingresso entram como uma nova `PoliticaPreco`, sem alterar a venda.
- **L** — `RepositoriosMemoria` e os repositórios PostgreSQL são intercambiáveis (os mesmos testes passam nos dois).
- **I** — interfaces separadas: `RepositorioUsuarios`, `RepositorioOrganizadores`, `RepositorioEventos`, `RepositorioVendas`, `HashSenha`, `Relogio`.
- **D** — os casos de uso dependem só dos `trait`s; as implementações são injetadas em `web/src/main.rs`.

### Padrões de projeto

| Padrão                 | Onde                                                            |
|------------------------|-----------------------------------------------------------------|
| Repository             | `dominio/src/portas.rs` + `infraestrutura/src/postgres/`        |
| Strategy               | `dominio/src/precos.rs` (inteira / meia-entrada)                |
| Builder                | `EventoBuilder` (`dominio/src/entidades/evento.rs`)             |
| Factory                | `FabricaUsuarios` (`aplicacao/src/fabrica.rs`)                  |
| Facade                 | `Aplicacao` (`aplicacao/src/lib.rs`)                            |
| Injeção de dependência | `Dependencias` + `web/src/main.rs` (composition root)           |
| Value Object           | `Cpf`, `Cnpj`, `Email`, `Dinheiro`                              |
| Post/Redirect/Get      | rotas da web (aviso após gravar)                                |

### POO em Rust

Rust não tem `class` nem herança. Usamos:
- `struct` + `impl` = classe com atributos e métodos;
- campos privados + métodos de acesso = **encapsulamento** (ex.: `Venda` só muda de situação por `marcar_cancelada()`);
- `trait` = **abstração**/interface; implementações diferentes do mesmo `trait` = **polimorfismo**
  (`PoliticaPreco`, `Codificavel`, repositórios);
- **composição** no lugar de herança (ex.: `Evento` contém `DadosEvento`; `Aplicacao` contém os serviços).
