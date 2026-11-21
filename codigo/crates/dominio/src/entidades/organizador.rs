use super::Codificavel;
use crate::Resultado;
use crate::valores::{Cnpj, texto_obrigatorio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StatusOrganizador {
    Ativo,
    Bloqueado,
}

impl Codificavel for StatusOrganizador {
    const TODOS: &'static [Self] = &[StatusOrganizador::Ativo, StatusOrganizador::Bloqueado];

    fn codigo(&self) -> &'static str {
        match self {
            StatusOrganizador::Ativo => "ATIVO",
            StatusOrganizador::Bloqueado => "BLOQUEADO",
        }
    }
}

/// Dados validados de um organizador que ainda vai ser gravado.
#[derive(Debug, Clone)]
pub struct NovoOrganizador {
    razao_social: String,
    cnpj: Cnpj,
    contato: String,
}

impl NovoOrganizador {
    pub fn novo(razao_social: &str, cnpj: Cnpj, contato: &str) -> Resultado<Self> {
        Ok(NovoOrganizador {
            razao_social: texto_obrigatorio(razao_social, "razão social")?,
            cnpj,
            contato: texto_obrigatorio(contato, "contato")?,
        })
    }
    pub fn razao_social(&self) -> &str {
        &self.razao_social
    }
    pub fn cnpj(&self) -> &Cnpj {
        &self.cnpj
    }
    pub fn contato(&self) -> &str {
        &self.contato
    }
}

/// Empresa produtora de eventos. Entra no sistema pelo usuário vinculado (`usuario_id`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Organizador {
    id: i64,
    usuario_id: i64,
    razao_social: String,
    cnpj: Cnpj,
    contato: String,
    status: StatusOrganizador,
}

impl Organizador {
    pub fn reconstruir(
        id: i64,
        usuario_id: i64,
        razao_social: String,
        cnpj: Cnpj,
        contato: String,
        status: StatusOrganizador,
    ) -> Self {
        Organizador { id, usuario_id, razao_social, cnpj, contato, status }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn usuario_id(&self) -> i64 {
        self.usuario_id
    }
    pub fn razao_social(&self) -> &str {
        &self.razao_social
    }
    pub fn cnpj(&self) -> &Cnpj {
        &self.cnpj
    }
    pub fn contato(&self) -> &str {
        &self.contato
    }
    pub fn status(&self) -> StatusOrganizador {
        self.status
    }
    pub fn esta_ativo(&self) -> bool {
        self.status == StatusOrganizador::Ativo
    }

    // ---- comportamentos (Algoritmo 7) ----

    pub fn ativar(&mut self) {
        self.status = StatusOrganizador::Ativo;
    }

    pub fn bloquear(&mut self) {
        self.status = StatusOrganizador::Bloqueado;
    }

    pub fn editar(&mut self, razao_social: &str, contato: &str) -> Resultado<()> {
        let razao_social = texto_obrigatorio(razao_social, "razão social")?;
        let contato = texto_obrigatorio(contato, "contato")?;
        self.razao_social = razao_social;
        self.contato = contato;
        Ok(())
    }
}
