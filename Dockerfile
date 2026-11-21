# Imagem do Sistema de Ingressos (Equipe 11).
#
# Build em 3 estágios com cargo-chef: as dependências ficam em cache e só são
# recompiladas quando o Cargo.toml/Cargo.lock mudam.
#
#   docker build -t ticket-system .

# ---------- 1. receita das dependências ----------
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planejador
COPY . .
RUN cargo chef prepare --recipe-path receita.json

# ---------- 2. compilação ----------
FROM chef AS compilador
COPY --from=planejador /app/receita.json receita.json
RUN cargo chef cook --release --recipe-path receita.json
COPY . .
RUN cargo build --release --bin ingressos

# ---------- 3. imagem final (só o executável) ----------
FROM debian:bookworm-slim AS execucao
RUN useradd --system --uid 10001 --no-create-home ingressos
COPY --from=compilador /app/target/release/ingressos /usr/local/bin/ingressos
USER ingressos
ENV ENDERECO=0.0.0.0:3000
EXPOSE 3000
# Verifica se a aplicação responde em /saude (sem precisar de curl na imagem).
HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=3 \
  CMD bash -c 'exec 3<>/dev/tcp/127.0.0.1/3000 && printf "GET /saude HTTP/1.0\r\n\r\n" >&3 && grep -q "ok" <&3'
CMD ["ingressos"]
