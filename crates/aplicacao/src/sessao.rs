use dominio::entidades::Perfil;
use dominio::{Erro, Resultado};

/// Usuário que fez login. É criado pelo `ServicoAutenticacao::entrar` e enviado a cada
/// caso de uso, que confere se o perfil tem permissão (controle de acesso — seção 7.4).
#[derive(Debug, Clone)]
pub struct Sessao {
    pub usuario_id: i64,
    pub nome: String,
    pub perfil: Perfil,
    /// Preenchido apenas para o perfil Organizador.
    pub organizador_id: Option<i64>,
}

impl Sessao {
    pub fn exigir(&self, perfil: Perfil) -> Resultado<()> {
        if self.perfil == perfil {
            Ok(())
        } else {
            Err(Erro::acesso_negado("função não disponível para o seu perfil"))
        }
    }
}
