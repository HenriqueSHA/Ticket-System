use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use dominio::portas::HashSenha;
use dominio::{Erro, Resultado};

/// Hash de senha com **Argon2** (algoritmo recomendado para senhas).
/// Cada hash tem um *salt* aleatório: duas pessoas com a mesma senha têm hashes diferentes.
#[derive(Debug, Default, Clone, Copy)]
pub struct HashArgon2;

impl HashSenha for HashArgon2 {
    fn gerar(&self, senha: &str) -> Resultado<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(senha.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| Erro::Infraestrutura(format!("falha ao gerar hash da senha: {e}")))
    }

    fn verificar(&self, senha: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|h| Argon2::default().verify_password(senha.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn gera_e_verifica() {
        let hash = HashArgon2.gerar("segredo1").unwrap();
        assert!(hash.starts_with("$argon2"));
        assert!(HashArgon2.verificar("segredo1", &hash));
        assert!(!HashArgon2.verificar("errada", &hash));
        assert_ne!(hash, HashArgon2.gerar("segredo1").unwrap()); // salt diferente
    }
}
