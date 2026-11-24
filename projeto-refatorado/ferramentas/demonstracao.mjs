// Percorre a aplicação como um usuário, tira os prints do relatório e grava um vídeo curto.
//
// Uso (a partir da raiz, com a aplicação no ar):
//   PLAYWRIGHT=$(npm root -g)/@playwright/cli/node_modules/playwright-core \
//     node ferramentas/demonstracao.mjs [url] [pasta-prints] [pasta-video]
import { createRequire } from "node:module";

const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT ?? "playwright-core");

const URL = process.argv[2] ?? "http://localhost:3000";
const PRINTS = process.argv[3] ?? "documentos/relatorio/prints";
const VIDEO = process.argv[4];
const PAUSA = 900;

const navegador = await chromium.launch({ executablePath: "/usr/bin/google-chrome" });
const contexto = await navegador.newContext({
  viewport: { width: 1280, height: 800 },
  locale: "pt-BR",
  ...(VIDEO ? { recordVideo: { dir: VIDEO, size: { width: 1280, height: 800 } } } : {}),
});
const p = await contexto.newPage();
const esperar = (ms = PAUSA) => p.waitForTimeout(ms);
const print = async (nome) => {
  await p.screenshot({ path: `${PRINTS}/${nome}.png`, fullPage: true });
  console.log("print:", nome);
};
const preencher = async (campos) => {
  for (const [nome, valor] of Object.entries(campos)) {
    const campo = p.locator(`[name="${nome}"]`).first();
    if ((await campo.evaluate((e) => e.tagName)) === "SELECT") await campo.selectOption(valor);
    else await campo.fill(valor);
  }
};
const enviar = async (texto) => {
  await p.getByRole("button", { name: texto }).first().click();
  await p.waitForLoadState("networkidle");
  await esperar();
};
const entrar = async (email, senha) => {
  await contexto.clearCookies();
  await p.goto(`${URL}/entrar`);
  await preencher({ email, senha });
  await enviar("Entrar");
};

// ---- público ----
await p.goto(URL);
await esperar();
await print("01-inicio");

await p.goto(`${URL}/cadastro/organizador`);
const org = { razao_social: "Aurora Produções", contato: "(71) 99999-0000", email: "aurora@eventos.com", senha: "aurora123" };
await preencher({ ...org, cnpj: "12.345.678/0001-00" });
await enviar("Criar conta");
await print("02-erro-cnpj-invalido");
await preencher({ ...org, cnpj: "12.ABC.345/01DE-35" });
await enviar("Criar conta");
await print("03-conta-de-organizador-criada");

// ---- organizador ----
await entrar("aurora@eventos.com", "aurora123");
const daqui30 = new Date(Date.now() + 30 * 864e5).toISOString().slice(0, 10);
await p.goto(`${URL}/organizador/eventos/novo`);
await preencher({
  nome: "Festival de Verão", descricao: "Música ao ar livre", local: "Farol da Barra",
  data: daqui30, horario: "18:00", capacidade: "200",
  cat_nome: "Arquibancada", cat_preco: "90,00", cat_assento: "sim", cat_fileiras: "4", cat_por_fileira: "10",
});
await print("04-cadastro-de-evento");
await enviar("Cadastrar evento");
await print("05-gerenciar-evento");

// ---- cliente: compra, comprovante, histórico e remarcação ----
await entrar("ana@email.com", "ana12345");
await print("06-eventos-disponiveis");
await p.locator('a[href^="/eventos/"]', { hasText: /Show de Rock|Comprar/ }).first().click();
await p.getByRole("link", { name: /^Cadeira/ }).first().click();
await esperar();
await p.locator('label:has(input[value="B-2"])').click();
await p.locator('input[name="tipo"][value="MEIA"]').check();
await esperar(500);
await print("07-compra-mapa-de-assentos");
await enviar("Continuar para o resumo");
await print("08-resumo-da-compra");
await enviar("Confirmar");
await print("09-comprovante");
await p.goto(`${URL}/meus-ingressos`);
await esperar();
await print("10-meus-ingressos");
await p.getByRole("link", { name: "Trocar assento" }).first().click();
await esperar();
await p.locator('label:has(input[value="C-3"])').click();
await esperar(500);
await print("11-remarcar-assento");
await enviar("Confirmar troca");
await print("12-assento-trocado");

// ---- organizador da demonstração: relatório ----
await entrar("contato@soleventos.com", "sol12345");
await p.locator('a[href$="/relatorio"]').first().click();
await esperar();
await print("13-relatorio-do-evento");

// ---- SuperAdmin ----
await entrar("admin@ingressos.com", "admin123");
await p.goto(`${URL}/admin/organizadores`);
await esperar();
await print("14-superadmin-organizadores");

// ---- erro tratado: login inválido ----
await entrar("ana@email.com", "senha-errada");
await print("15-erro-login-invalido");

await contexto.close();
await navegador.close();
