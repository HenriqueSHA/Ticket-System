use super::Codificavel;
use crate::Resultado;
use crate::valores::{Email, texto_obrigatorio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Perfil {
    Cliente,
    Organizador,
    SuperAdmin,
}

impl Perfil {
    pub fn nome(&self) -> &'static str {
        match self {
            Perfil::Cliente => "Cliente",
            Perfil::Organizador => "Organizador",
            Perfil::SuperAdmin => "SuperAdmin",
        }
    }
}

impl Codificavel for Perfil {
    const TODOS: &'static [Self] = &[Perfil::Cliente, Perfil::Organizador, Perfil::SuperAdmin];

    fn codigo(&self) -> &'static str {
        match self {
            Perfil::Cliente => "CLIENTE",
            Perfil::Organizador => "ORGANIZADOR",
            Perfil::SuperAdmin => "SUPERADMIN",
        }
    }
}

/// Usuário do sistema. A senha **não** é guardada: só o hash (restrição 7.6).
#[derive(Debug, Clone)]
pub struct Usuario {
    id: i64,
    nome: String,
    documento: String,
    email: Email,
    perfil: Perfil,
    senha_hash: String,
}

impl Usuario {
    /// Usuário ainda não gravado (id = 0). Use a `FabricaUsuarios` da camada de aplicação.
    pub fn novo(nome: &str, documento: &str, email: Email, perfil: Perfil, senha_hash: String) -> Resultado<Self> {
        Ok(Usuario {
            id: 0,
            nome: texto_obrigatorio(nome, "nome")?,
            documento: documento.to_string(),
            email,
            perfil,
            senha_hash,
        })
    }

    /// Recria um usuário lido do banco de dados.
    pub fn reconstruir(
        id: i64,
        nome: String,
        documento: String,
        email: Email,
        perfil: Perfil,
        senha_hash: String,
    ) -> Self {
        Usuario { id, nome, documento, email, perfil, senha_hash }
    }

    pub fn com_id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn nome(&self) -> &str {
        &self.nome
    }
    /// CPF (cliente/admin) ou CNPJ (organizador), somente dígitos.
    pub fn documento(&self) -> &str {
        &self.documento
    }
    pub fn email(&self) -> &Email {
        &self.email
    }
    pub fn perfil(&self) -> Perfil {
        self.perfil
    }
    pub fn senha_hash(&self) -> &str {
        &self.senha_hash
    }
}
