-- Esquema do Sistema de Cadastro e Venda de Ingressos (seção 13 do relatório).
-- Executado automaticamente pela aplicação na inicialização (sqlx migrate).

CREATE TABLE usuarios (
    id          BIGSERIAL PRIMARY KEY,
    nome        TEXT NOT NULL,
    documento   TEXT NOT NULL UNIQUE,          -- CPF ou CNPJ, somente dígitos
    email       TEXT NOT NULL UNIQUE,
    senha_hash  TEXT NOT NULL,                 -- hash Argon2 (a senha nunca é gravada)
    perfil      TEXT NOT NULL CHECK (perfil IN ('CLIENTE', 'ORGANIZADOR', 'SUPERADMIN'))
);

CREATE TABLE organizadores (
    id            BIGSERIAL PRIMARY KEY,
    usuario_id    BIGINT NOT NULL UNIQUE REFERENCES usuarios(id) ON DELETE CASCADE,
    razao_social  TEXT NOT NULL,
    cnpj          TEXT NOT NULL UNIQUE,
    contato       TEXT NOT NULL,
    status        TEXT NOT NULL CHECK (status IN ('ATIVO', 'BLOQUEADO'))
);

CREATE TABLE eventos (
    id                    BIGSERIAL PRIMARY KEY,
    organizador_id        BIGINT NOT NULL REFERENCES organizadores(id) ON DELETE CASCADE,
    nome                  TEXT NOT NULL,
    descricao             TEXT NOT NULL,
    local                 TEXT NOT NULL,
    data                  DATE NOT NULL,
    horario               TIME NOT NULL,
    capacidade            INTEGER NOT NULL CHECK (capacidade > 0),
    total_vendido         INTEGER NOT NULL DEFAULT 0 CHECK (total_vendido BETWEEN 0 AND capacidade),
    permite_cancelamento  BOOLEAN NOT NULL,
    status                TEXT NOT NULL CHECK (status IN ('ABERTO', 'ENCERRADO'))
);

-- Categorias de ingresso de cada evento (Pista, Cadeira, Camarote...).
CREATE TABLE ingressos (
    id                    BIGSERIAL PRIMARY KEY,
    evento_id             BIGINT NOT NULL REFERENCES eventos(id) ON DELETE CASCADE,
    categoria             TEXT NOT NULL,
    preco_centavos        BIGINT NOT NULL CHECK (preco_centavos >= 0),
    quantidade            INTEGER NOT NULL CHECK (quantidade > 0),
    disponivel            INTEGER NOT NULL CHECK (disponivel BETWEEN 0 AND quantidade),
    com_assento           BOOLEAN NOT NULL,
    fileiras              INTEGER NOT NULL DEFAULT 0,
    assentos_por_fileira  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE assentos (
    id           BIGSERIAL PRIMARY KEY,
    evento_id    BIGINT NOT NULL REFERENCES eventos(id) ON DELETE CASCADE,
    ingresso_id  BIGINT NOT NULL REFERENCES ingressos(id) ON DELETE CASCADE,
    setor        TEXT NOT NULL,
    fileira      TEXT NOT NULL,
    numero       INTEGER NOT NULL,
    situacao     TEXT NOT NULL CHECK (situacao IN ('LIVRE', 'OCUPADO')),
    UNIQUE (ingresso_id, fileira, numero)
);

CREATE TABLE vendas (
    id               BIGSERIAL PRIMARY KEY,
    data_compra      TIMESTAMP NOT NULL,
    valor_centavos   BIGINT NOT NULL CHECK (valor_centavos >= 0),
    tipo             TEXT NOT NULL CHECK (tipo IN ('INTEIRA', 'MEIA')),
    forma_pagamento  TEXT NOT NULL CHECK (forma_pagamento IN ('PIX', 'CREDITO', 'DEBITO')),
    status           TEXT NOT NULL CHECK (status IN ('CONFIRMADA', 'CANCELADA', 'UTILIZADA')),
    cliente_id       BIGINT NOT NULL REFERENCES usuarios(id),
    ingresso_id      BIGINT NOT NULL REFERENCES ingressos(id),
    assento_id       BIGINT REFERENCES assentos(id)
);

-- Garantia extra contra venda duplicada: no máximo UMA venda ativa por assento.
CREATE UNIQUE INDEX vendas_um_ativo_por_assento ON vendas (assento_id)
    WHERE assento_id IS NOT NULL AND status <> 'CANCELADA';

CREATE INDEX vendas_por_cliente ON vendas (cliente_id);
CREATE INDEX ingressos_por_evento ON ingressos (evento_id);
CREATE INDEX eventos_por_data ON eventos (data);
