use std::sync::Arc;

use dominio::entidades::{Organizador, Perfil, Usuario};
use dominio::portas::{RepositorioOrganizadores, RepositorioUsuarios};
use dominio::valores::Email;
use dominio::{Erro, Resultado};

use crate::cadastro_organizador::{self, CadastroOrganizador};
use crate::fabrica::{FabricaUsuarios, verificar_unicidade};
use crate::sessao::Sessao;

/// Dados do formulário "Criar conta".
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CadastroCliente {
    pub nome: String,
    pub cpf: String,
    pub email: String,
    pub senha: String,
}

pub struct ServicoAutenticacao {
    usuarios: Arc<dyn RepositorioUsuarios>,
    organizadores: Arc<dyn RepositorioOrganizadores>,
    fabrica: FabricaUsuarios,
}

impl ServicoAutenticacao {
    pub fn new(
        usuarios: Arc<dyn RepositorioUsuarios>,
        organizadores: Arc<dyn RepositorioOrganizadores>,
        fabrica: FabricaUsuarios,
    ) -> Self {
        ServicoAutenticacao { usuarios, organizadores, fabrica }
    }

    /// Algoritmo 1 — login: confere e-mail/senha e identifica o perfil.
    pub async fn entrar(&self, email: &str, senha: &str) -> Resultado<Sessao> {
        // mesma mensagem para e-mail inexistente e senha errada: não revela quem tem conta
        let invalido = || Erro::acesso_negado("e-mail ou senha inválidos");
        let email = Email::novo(email).map_err(|_| invalido())?;
        let usuario = self.usuarios.buscar_por_email(&email).await?.ok_or_else(invalido)?;
        if !self.fabrica.senha_confere(senha, &usuario) {
            return Err(invalido());
        }

        let organizador_id = if usuario.perfil() == Perfil::Organizador {
            let org = self
                .organizadores
                .buscar_por_usuario(usuario.id())
                .await?
                .ok_or_else(|| Erro::nao_encontrado("cadastro do organizador"))?;
            if !org.esta_ativo() {
                return Err(Erro::acesso_negado("organizador bloqueado. Procure o administrador"));
            }
            Some(org.id())
        } else {
            None
        };

        Ok(Sessao {
            usuario_id: usuario.id(),
            nome: usuario.nome().to_string(),
            perfil: usuario.perfil(),
            organizador_id,
        })
    }

    /// Algoritmo 2 — cadastrar cliente (validações na ordem do fluxograma, depois unicidade).
    pub async fn cadastrar_cliente(&self, dados: &CadastroCliente) -> Resultado<Usuario> {
        let usuario = self.fabrica.criar_cliente(&dados.nome, &dados.cpf, &dados.email, &dados.senha)?;
        verificar_unicidade(self.usuarios.as_ref(), &usuario).await?;
        self.usuarios.inserir(&usuario).await
    }

    /// Cadastro feito pela própria empresa produtora ("Criar conta de organizador").
    /// Depois do cadastro ela entra pelo mesmo login e já pode criar eventos.
    pub async fn cadastrar_organizador(&self, dados: &CadastroOrganizador) -> Resultado<Organizador> {
        cadastro_organizador::registrar(self.usuarios.as_ref(), self.organizadores.as_ref(), &self.fabrica, dados).await
    }
}
