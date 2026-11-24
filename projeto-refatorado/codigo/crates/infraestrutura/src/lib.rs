//! # Infraestrutura — implementações concretas das portas do domínio
//!
//! - [`postgres`]: repositórios no `PostgreSQL` (biblioteca sqlx), com transações;
//! - [`memoria`]: os mesmos repositórios guardando tudo em memória (usado nos testes);
//! - [`seguranca`]: hash de senha com Argon2;
//! - [`relogio`]: relógio do sistema e relógio fixo (testes).
//!
//! Como os dois conjuntos de repositórios implementam os mesmos `trait`s, um pode
//! substituir o outro sem nenhuma mudança nos casos de uso (Substituição de Liskov).

pub mod memoria;
pub mod postgres;
pub mod relogio;
pub mod seguranca;
