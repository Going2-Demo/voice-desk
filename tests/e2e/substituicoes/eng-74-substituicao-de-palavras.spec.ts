import { expect, test, type Page } from "@playwright/test";
import { irPara } from "../helpers/navegar";

const tabela = (page: Page) => page.getByRole("table", { name: "Substituições" });
const adicionar = async (page: Page, original: string, substituto: string) => {
  await page.getByLabel("Quando a transcrição disser").fill(original);
  await page.getByLabel("Escrever", { exact: true }).fill(substituto);
  await page.getByRole("button", { name: "Adicionar substituição" }).click();
};

test.beforeEach(async ({ page }) => {
  await irPara(page, "/substituicoes");
  await expect(page.getByRole("heading", { level: 1, name: "Substituições" })).toBeVisible();
  await expect(tabela(page)).toBeVisible();
});

test("RF-08.1: criar substituição aparece na tabela e o formulário é limpo", async ({ page }) => {
  await adicionar(page, "pix", "PIX");
  await expect(tabela(page).getByRole("row", { name: /pix PIX/ })).toBeVisible();
  await expect(page.getByLabel("Quando a transcrição disser")).toHaveValue("");
  await expect(page.getByRole("status").filter({ hasText: "Substituição salva." })).toBeAttached();
});

test("RF-08.1: editar e remover substituição", async ({ page }) => {
  await page.getByRole("button", { name: "Editar gou tu" }).click();
  await page.getByLabel("Escrever", { exact: true }).fill("Going2 Tech");
  await page.getByRole("button", { name: "Salvar substituição" }).click();
  await expect(tabela(page).getByText("Going2 Tech")).toBeVisible();
  await page.getByRole("button", { name: "Remover gou tu" }).click();
  await expect(tabela(page).getByText("gou tu")).toHaveCount(0);
});

test("RF-08.1: desativar e reativar", async ({ page }) => {
  const chave = page.getByRole("switch", { name: "Ativar “gou tu”" });
  await expect(chave).toBeChecked();
  await chave.uncheck();
  await expect(chave).not.toBeChecked();
  await expect(tabela(page).getByRole("row", { name: /gou tu/ })).toHaveClass(/desativada/);
  await chave.check();
  await expect(chave).toBeChecked();
});

test("critério: original vazio é recusado com aviso", async ({ page }) => {
  await page.getByLabel("Escrever", { exact: true }).fill("PIX");
  await page.getByRole("button", { name: "Adicionar substituição" }).click();
  await expect(page.getByRole("alert").filter({ hasText: "Digite o que a transcrição" })).toBeVisible();
  await expect(tabela(page).getByText("PIX")).toHaveCount(0);
});

test("recusas: substituto vazio, duplicada (sem caixa) e texto longo", async ({ page }) => {
  await page.getByLabel("Quando a transcrição disser").fill("abc");
  await page.getByRole("button", { name: "Adicionar substituição" }).click();
  await expect(page.getByRole("alert").filter({ hasText: "Digite o texto que deve" })).toBeVisible();
  await adicionar(page, "GOU TU", "x");
  await expect(page.getByRole("alert").filter({ hasText: "Já existe uma substituição" })).toBeVisible();
  await adicionar(page, "a".repeat(201), "x");
  await expect(page.getByRole("alert").filter({ hasText: "até 200 caracteres" })).toBeVisible();
});
