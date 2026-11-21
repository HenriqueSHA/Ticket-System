use std::sync::Arc;

use dominio::entidades::{NovoOrganizador, Organizador, Perfil};
use dominio::portas::{RepositorioOrganizadores, RepositorioUsuarios};
use dominio::valores::{Cnpj, texto_obrigatorio};
use dominio::{Erro, Resultado};

use crate::fabrica::{FabricaUsuarios, verificar_unicidade};
use crate::sessao::Sessao;

/// Dados do formulário "Novo organizador".
#[derive(Debug, Clone, Default)]
pub struct CadastroOrganizador {
    pub razao_social: String,
    pub cnpj: String,
    pub contato: String,
    pub email: String,
    pub senha: String,
}

/// Algoritmo 7 — funções exclusivas do SuperAdmin.
pub struct ServicoAdministracao {
    usuarios: Arc<dyn RepositorioUsuarios>,
    organizadores: Arc<dyn RepositorioOrganizadores>,
    fabrica: FabricaUsuarios,
}

impl ServicoAdministracao {
    pub fn new(
        usuarios: Arc<dyn RepositorioUsuarios>,
        organizadores: Arc<dyn RepositorioOrganizadores>,
        fabrica: FabricaUsuarios,
    ) -> Self {
        ServicoAdministracao { usuarios, organizadores, fabrica }
    }

    pub async fn cadastrar_organizador(&self, sessao: &Sessao, dados: &CadastroOrganizador) -> Resultado<Organizador> {
        sessao.exigir(Perfil::SuperAdmin)?;
        texto_obrigatorio(&dados.razao_social, "razão social")?;
        texto_obrigatorio(&dados.cnpj, "CNPJ")?;
        let cnpj = Cnpj::novo(&dados.cnpj)?;
        let novo = NovoOrganizador::novo(&dados.razao_social, cnpj.clone(), &dados.contato)?;
        let login = self.fabrica.criar_login_organizador(novo.razao_social(), &cnpj, &dados.email, &dados.senha)?;
        verificar_unicidade(self.usuarios.as_ref(), &login).await?;
        self.organizadores.inserir(&login, &novo).await
    }

    pub async fn listar(&self, sessao: &Sessao) -> Resultado<Vec<Organizador>> {
        sessao.exigir(Perfil::SuperAdmin)?;
        self.organizadores.listar().await
    }

    pub async fn buscar(&self, sessao: &Sessao, id: i64) -> Resultado<Organizador> {
        sessao.exigir(Perfil::SuperAdmin)?;
        self.organizadores
            .buscar_por_id(id)
            .await?
            .ok_or_else(|| Erro::nao_encontrado("organizador não encontrado"))
    }

    pub async fn ativar(&self, sessao: &Sessao, id: i64) -> Resultado<Organizador> {
        let mut org = self.buscar(sessao, id).await?;
        org.ativar();
        self.organizadores.atualizar(&org).await?;
        Ok(org)
    }

    /// Organizador bloqueado não consegue mais entrar no sistema (Algoritmo 1).
    pub async fn bloquear(&self, sessao: &Sessao, id: i64) -> Resultado<Organizador> {
        let mut org = self.buscar(sessao, id).await?;
        org.bloquear();
        self.organizadores.atualizar(&org).await?;
        Ok(org)
    }

    pub async fn editar(&self, sessao: &Sessao, id: i64, razao_social: &str, contato: &str) -> Resultado<Organizador> {
        let mut org = self.buscar(sessao, id).await?;
        org.editar(razao_social, contato)?;
        self.organizadores.atualizar(&org).await?;
        Ok(org)
    }

    /// Remove só se não houver vendas — para não apagar o histórico dos clientes.
    pub async fn remover(&self, sessao: &Sessao, id: i64) -> Resultado<()> {
        let org = self.buscar(sessao, id).await?;
        if self.organizadores.possui_vendas(org.id()).await? {
            return Err(Erro::regra("organizador possui eventos com vendas. Bloqueie em vez de remover"));
        }
        self.organizadores.remover(&org).await
    }
}
