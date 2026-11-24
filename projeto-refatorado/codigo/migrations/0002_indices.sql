-- Índices para as consultas mais frequentes (revisão de desempenho).
-- vendas por categoria: relatório, lista de vendas, remoção de categoria/evento
CREATE INDEX IF NOT EXISTS vendas_por_ingresso ON vendas (ingresso_id);
-- eventos por organizador: "Meus eventos"
CREATE INDEX IF NOT EXISTS eventos_por_organizador ON eventos (organizador_id);
