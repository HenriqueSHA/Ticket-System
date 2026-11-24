# Imagens do Sistema de Ingressos (Equipe 11): um alvo por serviço.
#
#   docker build --target back  -t ticket-system-back  codigo
#   docker build --target front -t ticket-system-front codigo
#
# Os dois alvos compartilham os estágios de compilação (cargo-chef deixa as dependências em
# cache: só são recompiladas quando Cargo.toml/Cargo.lock mudam).

# ---------- 1. receita das dependências ----------
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planejador
COPY . .
RUN cargo chef prepare --recipe-path receita.json

# ---------- 2. compilação (os dois executáveis) ----------
FROM chef AS compilador
COPY --from=planejador /app/receita.json receita.json
RUN cargo chef cook --release --recipe-path receita.json
COPY . .
RUN cargo build --release --bin ingressos-api --bin ingressos-web

# ---------- 3. base das imagens finais ----------
FROM debian:bookworm-slim AS execucao
RUN useradd --system --uid 10001 --no-create-home ingressos
USER ingressos

# ---------- back: API REST (única que acessa o PostgreSQL) ----------
FROM execucao AS back
COPY --from=compilador /app/target/release/ingressos-api /usr/local/bin/ingressos-api
ENV ENDERECO_API=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=20s --retries=3 \
  CMD bash -c 'exec 3<>/dev/tcp/127.0.0.1/8080 && printf "GET /api/saude HTTP/1.0\r\n\r\n" >&3 && grep -q "ok" <&3'
CMD ["ingressos-api"]

# ---------- front: interface web (fala só com o back) ----------
FROM execucao AS front
COPY --from=compilador /app/target/release/ingressos-web /usr/local/bin/ingressos-web
ENV ENDERECO=0.0.0.0:3000
EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=3 \
  CMD bash -c 'exec 3<>/dev/tcp/127.0.0.1/3000 && printf "GET /saude HTTP/1.0\r\n\r\n" >&3 && grep -q "ok" <&3'
CMD ["ingressos-web"]
