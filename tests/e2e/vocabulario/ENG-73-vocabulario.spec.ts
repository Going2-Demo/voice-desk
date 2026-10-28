import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

const lista = (page: import("@playwright/test").Page) =>
  page.getByRole("list", { name: "Palavras do vocabulário" });
const adicionar = async (page: import("@playwright/test").Page, palavra: string) => {
  await page.getByLabel(/Adicionar palavra|Primeira palavra/).fill(palavra);
  await page.getByRole("button", { name: "Adicionar", exact: true }).click();
};

test.beforeEach(async ({ page }) => {
  await irPara(page, "/vocabulario");
  await expect(page.getByRole("heading", { level: 1, name: "Vocabulário" })).toBeVisible();
});

test("RF-07.1: palavra adicionada aparece na lista e na busca", async ({ page }) => {
  await adicionar(page, "Zyxwvuta");
  await expect(lista(page).getByText("Zyxwvuta")).toBeVisible();
  await page.getByLabel("Buscar no vocabulário").fill("zyxw");
  await expect(lista(page).getByRole("listitem")).toHaveCount(1);
  await page.getByLabel("Buscar no vocabulário").fill("naoexiste123");
  await expect(page.getByText(/Nenhuma palavra encontrada/)).toBeVisible();
});

test("RF-07.1: duplicata sem diferenciar maiúsculas é recusada com aviso", async ({ page }) => {
  await adicionar(page, "going2");
  await expect(page.getByRole("alert").filter({ hasText: "já está no vocabulário" })).toBeVisible();
  await expect(lista(page).getByText("Going2", { exact: true })).toHaveCount(1);
});

test("RF-07.1: editar e remover palavra", async ({ page }) => {
  await page.getByRole("button", { name: "Editar LGPD" }).click();
  await page.getByLabel("Palavra", { exact: true }).fill("LGPD Nova");
  await page.getByRole("button", { name: "Salvar" }).click();
  await expect(lista(page).getByText("LGPD Nova")).toBeVisible();
  await page.getByRole("button", { name: "Remover LGPD Nova" }).click();
  await expect(lista(page).getByText("LGPD Nova")).toHaveCount(0);
});

test("RF-07.1: palavra vazia é recusada", async ({ page }) => {
  await page.getByRole("button", { name: "Adicionar", exact: true }).click();
  await expect(page.getByText("Digite uma palavra ou nome.")).toBeVisible();
});

test("RF-07.4: a 501ª palavra é recusada com 'Limite de 500 palavras.'", async ({ page }) => {
  test.setTimeout(120_000);
  // semente tem 7 palavras; completa as 500 pela própria tela
  for (let i = 0; i < 493; i++) await adicionar(page, `palavra${i}`);
  await expect(page.getByText("500 de 500 palavras")).toBeVisible();
  await adicionar(page, "palavra501");
  await expect(page.getByText("Limite de 500 palavras.")).toBeVisible();
});

test("Critério: vocabulário vazio mostra estado vazio e segue funcionando", async ({ page }) => {
  for (const p of ["Going2", "Hortolândia", "Kubernetes", "LGPD", "Dokploy", "Voice Desk", "SLA"])
    await page.getByRole("button", { name: `Remover ${p}` }).click();
  await expect(page.getByRole("heading", { name: "Seu vocabulário está vazio" })).toBeVisible();
  await adicionar(page, "Primeira");
  await expect(lista(page).getByText("Primeira")).toBeVisible();
});

test("RF-07.3: caixa rápida do menu adiciona e recusa vazio", async ({ page }) => {
  await page.goto("/menu.html");
  await page.getByRole("button", { name: "Adicionar" }).click();
  await expect(page.getByText("Digite uma palavra ou nome.")).toBeVisible();
  await page.getByLabel("Adicionar ao vocabulário").fill("Hortolândia Norte");
  await page.getByRole("button", { name: "Adicionar" }).click();
  await expect(page.getByText("“Hortolândia Norte” adicionada ao vocabulário.")).toBeVisible();
});
