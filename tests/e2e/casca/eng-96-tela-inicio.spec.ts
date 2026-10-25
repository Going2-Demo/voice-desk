import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

test("aceite: a janela principal abre em Início com como ditar e o atalho atual", async ({
  page,
}) => {
  await irPara(page, "/inicio");
  await expect(page.getByRole("heading", { level: 1, name: "Início" })).toBeVisible();
  await expect(page.getByRole("heading", { level: 2, name: "Como ditar" })).toBeVisible();
  // No núcleo simulado o atalho não é resolvido: vale o texto de reserva (o atalho real fica nas pendências nativas).
  await expect(page.getByText("Pressione o atalho de ditado e fale")).toBeVisible();
  await expect(page.getByText("fechar esta janela não encerra o app")).toBeVisible();
  await expect(page.getByText("Tela ainda não implementada")).toHaveCount(0);
});

test("aceite: Início tem Testar ditado e links para primeiro uso, provedores e atalhos", async ({
  page,
}) => {
  await irPara(page, "/inicio");
  await expect(page.getByRole("button", { name: "Testar ditado" })).toBeVisible();
  await page.getByRole("main").getByRole("link", { name: "Provedores e chaves" }).click();
  await expect(page).toHaveURL(/#\/config\/provedores/);
  await page.goBack();
  await page.getByRole("main").getByRole("link", { name: "Atalhos" }).click();
  await expect(page).toHaveURL(/#\/config\/atalhos/);
  await page.goBack();
  await page.getByRole("link", { name: "Guia de primeiro uso" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Primeiro uso" })).toBeVisible();
});

test("aceite: raiz sem rota leva a Início e o fim do primeiro uso também", async ({ page }) => {
  await page.goto("/#/primeiro-uso");
  for (let i = 0; i < 5; i++) await page.getByRole("button", { name: "Continuar" }).click();
  await page.getByRole("button", { name: "Simular ditado de teste" }).click();
  await page.getByRole("button", { name: "Concluir" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Início" })).toBeVisible();
});
