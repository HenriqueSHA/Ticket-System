# Sistema de Cadastro e Venda de Ingressos — código-fonte

Aplicação web em **Rust** com banco de dados **PostgreSQL** (Equipe 11 — A3 UNIFACS 2026.2).
Visão geral do trabalho e dos documentos: [README da raiz](../README.md).

> Os comandos `docker compose` rodam na **raiz do repositório** (onde está o `docker-compose.yml`);
> os comandos `cargo` rodam dentro de `codigo/`.

## Como executar

### Em um servidor (tudo em contêiner)

Pré-requisito: Docker com o plugin Compose.

```bash
git clone https://github.com/HenriqueSHA/Ticket-System.git
cd Ticket-System              # raiz do repositório
cp .env.example .env          # troque POSTGRES_PASSWORD!
docker compose up -d --build  # compila a imagem e sobe app + PostgreSQL
```

O sistema fica em **http://IP-DO-SERVIDOR:3000** (porta configurável em `PORTA_WEB`).

| Variável (.env)     | Padrão      | Para que serve                                              |
|---------------------|-------------|-------------------------------------------------------------|
| `POSTGRES_USER`     | ingressos   | usuário do banco                                            |
| `POSTGRES_PASSWORD` | ingressos   | senha do banco — **troque em produção**                     |
| `POSTGRES_DB`       | ingressos   | nome do banco                                               |
| `PORTA_WEB`         | 3000        | porta pública da aplicação                                  |
| `PORTA_BANCO`       | 5433        | porta do PostgreSQL, aberta só em 127.0.0.1                 |
| `COOKIE_SEGURO`     | false       | `true` quando o site estiver atrás de HTTPS (proxy reverso) |

Comandos úteis:

```bash
docker compose ps                 # situação (os dois serviços têm healthcheck)
docker compose logs -f app        # registros da aplicação
docker compose up -d --build app  # atualizar depois de um git pull
docker compose down               # parar (os dados ficam no volume)
docker compose down -v            # parar e APAGAR o banco
```

A imagem é construída em três estágios (`Dockerfile`): as dependências ficam em cache, o
executável é compilado em modo *release* e a imagem final contém só o binário (≈120 MB),
rodando com um usuário sem privilégios. As tabelas são criadas automaticamente na primeira
execução. `GET /saude` responde `ok` (usado pelo healthcheck).

### Desenvolvimento (na sua máquina)

Pré-requisitos: [Rust](https://rustup.rs) e Docker.

```bash
cp .env.example .env                 # na raiz do repositório
docker compose up -d db              # só o PostgreSQL (porta 5433)
cd codigo && cargo run               # compila e inicia o sistema (lê o .env da raiz)
```

Abra **http://127.0.0.1:3000**. Na primeira execução o banco recebe dados de demonstração:

| Perfil      | E-mail                   | Senha     |
|-------------|--------------------------|-----------|
| SuperAdmin  | admin@ingressos.com      | admin123  |
| Organizador | contato@soleventos.com   | sol12345  |
| Organizador | arena@cultural.com       | arena123  |
| Cliente     | ana@email.com            | ana12345  |
| Cliente     | bruno@email.com          | bruno123  |

Para zerar o banco: `docker compose down -v && docker compose up -d db`.

## Funcionalidades

| Perfil | O que pode fazer |
|--------|------------------|
| **Cliente** | criar conta (CPF validado), ver eventos com ingressos disponíveis, comprar escolhendo categoria, inteira/meia, assento no mapa e forma de pagamento, comprovante, histórico e cancelamento |
| **Organizador** | criar a própria conta (CNPJ validado), cadastrar eventos com várias categorias (preço e quantidade, com ou sem assento marcado), **gerenciar cada evento**: ver vendidos/disponíveis por categoria, editar nome/preço/quantidade, adicionar e remover categorias, alterar capacidade, abrir/encerrar vendas e excluir evento sem vendas; relatório, lista de vendas e check-in |
| **SuperAdmin** | cadastrar, listar, ativar, bloquear, editar e remover organizadores |

Regras protegidas: a quantidade de uma categoria nunca fica abaixo do que já foi vendido; a soma
das categorias não passa da capacidade; categoria ou evento com vendas não é apagado (o histórico
dos clientes é preservado); o novo preço vale só para as próximas vendas.

## Testes

```bash
cargo test                  # 31 testes (domínio + cenários com repositórios em memória)
cargo test -- --ignored     # + teste de integração no PostgreSQL real (precisa do docker)
```

Os cenários pedidos na seção 15 do relatório estão em `crates/aplicacao/tests/cenarios.rs`:
**Teste 1** (normal — compra com assento), **Teste 2** (alternativo — meia-entrada sem assento e
cancelamento) e **Teste 3** (excepcionais — assento já vendido, evento lotado, dados inválidos,
organizador bloqueado, controle de acesso, cancelamento proibido).

## Arquitetura (camadas)

```
codigo/
├── Dockerfile                imagem da aplicação (build multi-estágio)
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
| Gestão do evento (categorias, preço, quantidade, capacidade) | `servicos/eventos.rs` (`alterar_categoria`, `adicionar_categoria`...), `entidades/ingresso.rs` |
| Cadastro de organizador            | `aplicacao/src/cadastro_organizador.rs` (usado pela tela pública e pelo SuperAdmin) |
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
