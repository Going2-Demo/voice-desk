import { expect, type Page, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

const interruptor = (page: Page, nome: string) => page.getByRole("switch", { name: nome });
const REMOVER = "Remover palavras de preenchimento";
const FORMATAR = "Formatar o texto";

async function definir(page: Page, nome: string, ligado: boolean) {
  const sw = interruptor(page, nome);
  if ((await sw.getAttribute("aria-checked")) !== String(ligado)) await sw.click();
  await expect(sw).toHaveAttribute("aria-checked", String(ligado));
}

async function configurar(page: Page, remover: boolean, formatar: boolean) {
  await irPara(page, "/config/geral");
  await expect(page.getByRole("heading", { level: 2, name: "Texto do ditado" })).toBeVisible();
  await definir(page, REMOVER, remover);
  await definir(page, FORMATAR, formatar);
}

const saida = (page: Page) => page.locator("output");

async function digitar(page: Page, falado: string) {
  await page.evaluate(() => {
    window.location.hash = "#/limpeza";
  });
  await expect(page.getByRole("heading", { level: 1, name: "Limpeza do texto" })).toBeVisible();
  await page.getByLabel("Texto falado").fill(falado);
}

test('CA1 (RF-09.1/09.3): remoção e formatação ligadas → "Chego em dez minutos."', async ({
  page,
}) => {
  await configurar(page, true, true);
  await digitar(page, "hã… chego em dez minutos");
  await expect(saida(page)).toHaveText("Chego em dez minutos.");
});

test("CA2 (RF-09.1): tudo desligado → transcrição sem alteração", async ({ page }) => {
  await configurar(page, false, false);
  await digitar(page, "hã… chego em dez minutos");
  await expect(saida(page)).toHaveText("hã… chego em dez minutos");
  await expect(page.getByText(/estão desligadas em Configurações/)).toBeVisible();
});

test('RF-09.1: remoção desligada mantém "é… eu acho que sim" como a transcrição', async ({
  page,
}) => {
  await configurar(page, false, false);
  await digitar(page, "é… eu acho que sim");
  await expect(saida(page)).toHaveText("é… eu acho que sim");
});

test("RF-09.1: liga/desliga é separado — só formatar", async ({ page }) => {
  await configurar(page, false, true);
  await digitar(page, "hã… chego em dez minutos");
  await expect(saida(page)).toHaveText("Hã… chego em dez minutos.");
});

test("RF-09.1: liga/desliga é separado — só remover", async ({ page }) => {
  await configurar(page, true, false);
  await digitar(page, "hã… chego em dez minutos");
  await expect(saida(page)).toHaveText("chego em dez minutos");
});

test('RF-09.2: lista vem preenchida; "tipo" na lista → "É assim."', async ({ page }) => {
  await configurar(page, true, true);
  await page.evaluate(() => {
    window.location.hash = "#/limpeza";
  });
  const lista = page.getByRole("list", { name: "Palavras de preenchimento" });
  for (const p of ["tipo", "hã", "né", "you know"]) {
    await expect(lista.getByText(p, { exact: true })).toBeVisible();
  }
  await page.getByLabel("Texto falado").fill("é tipo assim");
  await expect(saida(page)).toHaveText("É assim.");
});

test('RF-09.2: remover "tipo" da lista faz a palavra voltar ao texto; adicionar passa a remover', async ({
  page,
}) => {
  await configurar(page, true, true);
  await page.evaluate(() => {
    window.location.hash = "#/limpeza";
  });
  await page.getByRole("button", { name: "Remover tipo" }).click();
  await page.getByLabel("Texto falado").fill("é tipo assim");
  await expect(saida(page)).toHaveText("É tipo assim.");
  await page.getByLabel("Nova palavra").fill("basicamente");
  await page.getByRole("button", { name: "Adicionar" }).click();
  await page.getByLabel("Texto falado").fill("basicamente funciona");
  await expect(saida(page)).toHaveText("Funciona.");
});

test("RF-09.2: palavra vazia é recusada", async ({ page }) => {
  await irPara(page, "/limpeza");
  await page.getByRole("button", { name: "Adicionar" }).click();
  await expect(page.getByRole("alert")).toBeVisible();
});

test('RF-09.3: "olá  tudo bem" → "Olá tudo bem."; pontuação final existente é mantida', async ({
  page,
}) => {
  await configurar(page, false, true);
  await digitar(page, "olá  tudo bem");
  await expect(saida(page)).toHaveText("Olá tudo bem.");
  await page.getByLabel("Texto falado").fill("tudo bem?");
  await expect(saida(page)).toHaveText("Tudo bem?");
});
