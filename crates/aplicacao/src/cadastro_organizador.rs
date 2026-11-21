//! Cadastro de empresa organizadora, usado em dois lugares:
//! pela própria empresa (tela pública "Criar conta de organizador") e pelo SuperAdmin.

use dominio::Resultado;
use dominio::entidades::{NovoOrganizador, Organizador};
use dominio::portas::{RepositorioOrganizadores, RepositorioUsuarios};
use dominio::valores::{Cnpj, texto_obrigatorio};

use crate::fabrica::{FabricaUsuarios, verificar_unicidade};

/// Dados do formulário de cadastro de organizador.
#[derive(Debug, Clone, Default)]
pub struct CadastroOrganizador {
    pub razao_social: String,
    pub cnpj: String,
    pub contato: String,
    pub email: String,
    pub senha: String,
}

/// Valida (obrigatórios → CNPJ → e-mail → senha → unicidade) e grava login + organizador.
/// O organizador já começa ATIVO; o SuperAdmin pode bloqueá-lo depois.
pub(crate) async fn registrar(
    usuarios: &dyn RepositorioUsuarios,
    organizadores: &dyn RepositorioOrganizadores,
    fabrica: &FabricaUsuarios,
    dados: &CadastroOrganizador,
) -> Resultado<Organizador> {
    texto_obrigatorio(&dados.razao_social, "razão social")?;
    texto_obrigatorio(&dados.cnpj, "CNPJ")?;
    let cnpj = Cnpj::novo(&dados.cnpj)?;
    let novo = NovoOrganizador::novo(&dados.razao_social, cnpj.clone(), &dados.contato)?;
    let login = fabrica.criar_login_organizador(novo.razao_social(), &cnpj, &dados.email, &dados.senha)?;
    verificar_unicidade(usuarios, &login).await?;
    organizadores.inserir(&login, &novo).await
}
