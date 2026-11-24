use std::sync::Arc;

use dominio::entidades::{Perfil, Usuario};
use dominio::portas::{HashSenha, RepositorioUsuarios};
use dominio::valores::{Cnpj, Cpf, Email, texto_obrigatorio, validar_senha};
use dominio::{Erro, Resultado};

/// Padrão **Factory**: concentra a criação de usuários de cada perfil.
///
/// Valida o documento conforme o perfil (CPF para pessoas, CNPJ para empresas),
/// valida a senha e grava apenas o hash.
#[derive(Clone)]
pub struct FabricaUsuarios {
    hash: Arc<dyn HashSenha>,
}

impl FabricaUsuarios {
    pub fn new(hash: Arc<dyn HashSenha>) -> Self {
        FabricaUsuarios { hash }
    }

    /// Ordem das verificações igual ao fluxograma 2: obrigatórios → CPF → e-mail → senha.
    pub fn criar_cliente(&self, nome: &str, cpf: &str, email: &str, senha: &str) -> Resultado<Usuario> {
        self.criar_pessoa(nome, cpf, email, senha, Perfil::Cliente)
    }

    pub fn criar_superadmin(&self, nome: &str, cpf: &str, email: &str, senha: &str) -> Resultado<Usuario> {
        self.criar_pessoa(nome, cpf, email, senha, Perfil::SuperAdmin)
    }

    /// Usuário de login de uma empresa organizadora (documento = CNPJ).
    pub fn criar_login_organizador(
        &self,
        razao_social: &str,
        cnpj: &Cnpj,
        email: &str,
        senha: &str,
    ) -> Resultado<Usuario> {
        texto_obrigatorio(email, "e-mail")?;
        let email = Email::novo(email)?;
        validar_senha(senha)?;
        Usuario::novo(razao_social, cnpj.digitos(), email, Perfil::Organizador, self.hash.gerar(senha)?)
    }

    pub fn senha_confere(&self, senha: &str, usuario: &Usuario) -> bool {
        self.hash.verificar(senha, usuario.senha_hash())
    }

    fn criar_pessoa(&self, nome: &str, cpf: &str, email: &str, senha: &str, perfil: Perfil) -> Resultado<Usuario> {
        texto_obrigatorio(nome, "nome")?;
        texto_obrigatorio(cpf, "CPF")?;
        texto_obrigatorio(email, "e-mail")?;
        texto_obrigatorio(senha, "senha")?;
        let cpf = Cpf::novo(cpf)?;
        let email = Email::novo(email)?;
        validar_senha(senha)?;
        Usuario::novo(nome, cpf.digitos(), email, perfil, self.hash.gerar(senha)?)
    }
}

/// Regra 7.3: todo usuário deve ter cadastro único (e-mail e documento).
pub(crate) async fn verificar_unicidade(usuarios: &dyn RepositorioUsuarios, usuario: &Usuario) -> Resultado<()> {
    if usuarios.buscar_por_email(usuario.email()).await?.is_some() {
        return Err(Erro::regra("já existe um usuário com este e-mail"));
    }
    if usuarios.buscar_por_documento(usuario.documento()).await?.is_some() {
        return Err(Erro::regra("já existe um usuário com este CPF/CNPJ"));
    }
    Ok(())
}
