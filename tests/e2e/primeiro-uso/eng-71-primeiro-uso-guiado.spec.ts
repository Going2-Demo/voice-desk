import { expect, type Page, test } from "@playwright/test";

const continuar = (page: Page) => page.getByRole("button", { name: "Continuar" });
const passo = (page: Page, titulo: string) =>
  expect(page.getByRole("heading", { level: 2, name: titulo })).toBeVisible();

async function abrir(page: Page, plat = "") {
  await page.goto(`/${plat}#/`);
  await expect(page.getByRole("heading", { level: 1, name: "Primeiro uso" })).toBeVisible();
}

test("aceite: primeira abertura leva ao primeiro uso e o passo a passo termina com o ditado de teste", async ({
  page,
}) => {
  await abrir(page);
  await passo(page, "Bem-vindo ao Voice Desk");
  await continuar(page).click();

  await passo(page, "Permissões");
  await continuar(page).click();
  await passo(page, "Escolha o atalho");
  await continuar(page).click();
  await passo(page, "Como transcrever?");
  await continuar(page).click();
  await passo(page, "Sincronização (opcional)");
  await continuar(page).click();

  await passo(page, "Faça um ditado de teste");
  await page.getByRole("button", { name: "Simular ditado de teste" }).click();
  await expect(page.getByLabel("Caixa de teste")).toHaveValue(/primeiro ditado/);
  await page.getByRole("button", { name: "Concluir" }).click();
});

test("RF-06.1: permissão pendente mostra explicação, o que deixa de funcionar e como conceder; conceder muda para concedida", async ({
  page,
}) => {
  await abrir(page);
  await continuar(page).click();
  await passo(page, "Permissões");
  await expect(page.getByText("Para gravar a sua voz.")).toBeVisible();
  await expect(page.getByRole("status", { name: "Microfone: Pendente" })).toBeVisible();
  await expect(page.getByRole("note")).toContainText("Sem o microfone o ditado não grava nada.");
  await expect(page.getByRole("note")).toContainText("Privacidade e segurança");
  await page.getByRole("button", { name: "Conceder microfone" }).click();
  await expect(page.getByRole("status", { name: "Microfone: Concedida" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Conceder microfone" })).toHaveCount(0);
});

test("RF-06.1: no Windows a Acessibilidade não aparece", async ({ page }) => {
  await abrir(page);
  await continuar(page).click();
  await passo(page, "Permissões");
  await expect(page.getByRole("status", { name: "Microfone: Pendente" })).toBeVisible();
  await expect(page.getByText("Acessibilidade", { exact: true })).toHaveCount(0);
});

test("RF-06.1: no Mac também pede Acessibilidade", async ({ page }) => {
  await abrir(page, "?plat=mac");
  await continuar(page).click();
  await expect(page.getByRole("status", { name: "Acessibilidade: Pendente" })).toBeVisible();
  await expect(page.getByRole("note").last()).toContainText("Ajustes do Sistema");
});

test("RF-06.2: pular um passo o marca como pulado e permite voltar a ele", async ({ page }) => {
  await abrir(page);
  await continuar(page).click();
  await continuar(page).click();
  await continuar(page).click();
  await passo(page, "Como transcrever?");
  await page.getByRole("button", { name: "Pular este passo" }).click();
  await passo(page, "Sincronização (opcional)");
  await expect(page.getByRole("list", { name: "Passos" })).toContainText("Provedor (pulado)");
  await page.getByRole("button", { name: "Voltar" }).click();
  await passo(page, "Como transcrever?");
});

test("RF-06.2 / aceite: pulando a chave do Groq o primeiro uso termina e Chaves segue acessível", async ({
  page,
}) => {
  await abrir(page);
  for (let i = 0; i < 3; i++) await continuar(page).click();
  await page.getByRole("button", { name: "Pular este passo" }).click();
  await continuar(page).click();
  await passo(page, "Faça um ditado de teste");
  await page.getByRole("button", { name: "Concluir" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Primeiro uso" })).toHaveCount(0);

  await page
    .getByRole("navigation", { name: "Principal" })
    .getByRole("link", { name: "Provedores e chaves" })
    .click();
  await expect(page.getByRole("heading", { name: "Provedores e chaves" })).toBeVisible();
});

test("RF-06.2: Configurações › Permissões continua acessível depois de pular", async ({ page }) => {
  await abrir(page);
  await page.getByRole("link", { name: "Permissões" }).click();
  await expect(page.getByRole("status", { name: "Microfone: Pendente" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Conceder microfone" })).toBeVisible();
});

test("RF-06.4: o passo da chave explica como obter uma e valida chave vazia e inválida", async ({
  page,
}) => {
  await abrir(page);
  for (let i = 0; i < 3; i++) await continuar(page).click();
  await passo(page, "Como transcrever?");
  await expect(page.getByText(/chave de API gratuita ou paga, criada por você/)).toBeVisible();
  await expect(page.getByRole("button", { name: /Como obter uma chave/ })).toBeVisible();
  await expect(page.getByRole("button", { name: "Usar transcrição local" })).toBeVisible();

  await page.getByRole("button", { name: "Salvar e testar chave" }).click();
  await expect(page.getByRole("alert")).toContainText("Cole a chave de API do Groq.");

  await page.getByLabel("Chave de API do Groq").fill("gsk_invalida_9999");
  await page.getByRole("button", { name: "Salvar e testar chave" }).click();
  await expect(page.getByText(/o Groq a recusou/)).toBeVisible();

  await page.getByLabel("Chave de API do Groq").fill("gsk_boa_1234");
  await page.getByRole("button", { name: "Salvar e testar chave" }).click();
  await expect(page.getByText("Chave guardada no cofre do sistema.")).toBeVisible();
});

test("RF-06.4: 'Usar transcrição local' pula a chave e segue para a sincronização", async ({
  page,
}) => {
  await abrir(page);
  for (let i = 0; i < 3; i++) await continuar(page).click();
  await page.getByRole("button", { name: "Usar transcrição local" }).click();
  await passo(page, "Sincronização (opcional)");
});

test("RF-06.2: sem conseguir ler o atalho (núcleo simulado) o passo avisa e o usuário segue adiante", async ({
  page,
}) => {
  await abrir(page);
  for (let i = 0; i < 2; i++) await continuar(page).click();
  await passo(page, "Escolha o atalho");
  await expect(page.getByText(/ajustar depois em Configurações › Atalhos/)).toBeVisible({
    timeout: 20_000,
  });
  await continuar(page).click();
  await passo(page, "Como transcrever?");
});

test("aceite: primeiro uso concluído não aparece de novo ao voltar ao início", async ({ page }) => {
  await abrir(page);
  for (let i = 0; i < 5; i++) await continuar(page).click();
  await page.getByRole("button", { name: "Concluir" }).click();
  await page.goto("/#/");
  await expect(page.getByRole("heading", { level: 1, name: "Primeiro uso" })).toHaveCount(0);
});
