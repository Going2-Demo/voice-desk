import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/#/config/provedores");
  await expect(page.getByRole("heading", { name: "Provedores e chaves" })).toBeVisible();
});

const campo = (page: import("@playwright/test").Page) => page.getByLabel("Chave de API");
const salvar = (page: import("@playwright/test").Page) =>
  page.getByRole("button", { name: /Salvar e testar/ });

test("RF-04.2 / aceite: chave válida mostra 'válida' e máscara dos 4 últimos (RF-04.3)", async ({
  page,
}) => {
  await campo(page).fill("gsk_chave_boa_abcd");
  await salvar(page).click();
  await expect(page.getByRole("heading", { name: /Groq/ })).toContainText(/válida/i, {
    timeout: 10_000,
  });
  await expect(page.getByTestId("mascara")).toHaveText("••••abcd");
  await expect(page.getByText("gsk_chave_boa_abcd")).toHaveCount(0);
  await expect(campo(page)).toHaveCount(0);
});

test("RF-04.2: chave inventada fica inválida com o motivo do provedor", async ({ page }) => {
  await campo(page).fill("gsk_invalida_9999");
  await salvar(page).click();
  const alerta = page.getByRole("alert").filter({ hasText: "recusou" });
  await expect(alerta).toContainText("Invalid API Key", { timeout: 10_000 });
  await expect(page.getByRole("heading", { name: /Groq/ })).toContainText(/inválida/i);
});

test("RF-04.1: chave vazia é recusada com mensagem", async ({ page }) => {
  await salvar(page).click();
  await expect(page.getByText("Cole a chave de API do Groq.")).toBeVisible();
});

test("RF-04.1: trocar a chave atualiza a máscara", async ({ page }) => {
  await campo(page).fill("gsk_primeira_1111");
  await salvar(page).click();
  await expect(page.getByTestId("mascara")).toHaveText("••••1111");
  await page.getByRole("button", { name: "Trocar chave" }).click();
  await campo(page).fill("gsk_segunda_2222");
  await salvar(page).click();
  await expect(page.getByTestId("mascara")).toHaveText("••••2222");
});

test("RF-04.1: apagar pede confirmação, cancelar mantém e confirmar apaga", async ({ page }) => {
  await campo(page).fill("gsk_apagar_3333");
  await salvar(page).click();
  await expect(page.getByTestId("mascara")).toBeVisible();
  await page.getByRole("button", { name: "Apagar", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Apagar a chave do Groq?" })).toBeVisible();
  await page.getByRole("button", { name: "Cancelar" }).click();
  await expect(page.getByTestId("mascara")).toBeVisible();
  await page.getByRole("button", { name: "Apagar", exact: true }).click();
  await page.getByRole("button", { name: "Apagar chave" }).click();
  await expect(page.getByTestId("mascara")).toHaveCount(0);
  await expect(campo(page)).toBeVisible();
});

test("RF-04.3: o campo é mascarado e a chave não vai para localStorage/sessionStorage", async ({
  page,
}) => {
  await expect(campo(page)).toHaveAttribute("type", "password");
  await campo(page).fill("gsk_segredo_4444");
  await salvar(page).click();
  await expect(page.getByTestId("mascara")).toBeVisible();
  const guardado = await page.evaluate(() =>
    JSON.stringify([{ ...localStorage }, { ...sessionStorage }]),
  );
  expect(guardado).not.toContain("gsk_segredo_4444");
});

test("RF-04.4: link 'Onde consigo uma chave?' abre o console do Groq", async ({ page }) => {
  const popup = page.waitForEvent("popup");
  await page.getByRole("button", { name: /Onde consigo uma chave/ }).click();
  expect((await popup).url()).toContain("console.groq.com");
});
