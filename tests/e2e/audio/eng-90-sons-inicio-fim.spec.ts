import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

test.describe("ENG-90 · F-24 sons de início e fim (interface, núcleo simulado)", () => {
  test("RF-24.1 seção 'Durante o ditado': sons ligados por padrão, com 3 sons próprios", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    await expect(page.getByRole("heading", { level: 2, name: "Durante o ditado" })).toBeVisible();
    const interruptor = page.getByRole("switch", { name: "Sons de início e fim" });
    await expect(interruptor).toBeChecked();
    const som = page.getByLabel("Som", { exact: true });
    await expect(som.getByRole("option")).toHaveCount(3);
    await expect(som.getByRole("option")).toHaveText([/Gota/, /Pulso/, /Clique suave/]);
    await expect(page.getByRole("button", { name: /Ouvir prévia/ })).toBeEnabled();
  });

  test("RF-24.1 critério sons desligados: escolha e prévia ficam desabilitadas e a preferência é salva", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    const interruptor = page.getByRole("switch", { name: "Sons de início e fim" });
    await interruptor.setChecked(false, { force: true });
    await expect(page.getByText("Preferência salva")).toBeVisible();
    await expect(interruptor).not.toBeChecked();
    await expect(page.getByLabel("Som", { exact: true })).toBeDisabled();
    await expect(page.getByRole("button", { name: /Ouvir prévia/ })).toBeDisabled();
  });

  test("RF-24.1 critério sons ligados: escolher outro som e ouvir a prévia, sem erro", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    const som = page.getByLabel("Som", { exact: true });
    await som.selectOption({ index: 1 });
    await expect(page.getByText("Preferência salva")).toBeVisible();
    const escolhido = await som.inputValue();
    await page.getByRole("button", { name: /Ouvir prévia/ }).click();
    await expect(page.getByRole("alert")).toHaveCount(0);
    await expect(som).toHaveValue(escolhido);
  });

  test("RF-24.1 a preferência persiste ao reabrir a tela (mesma sessão)", async ({ page }) => {
    await irPara(page, "/config/geral");
    const som = page.getByLabel("Som", { exact: true });
    await som.selectOption({ index: 2 });
    await expect(page.getByText("Preferência salva")).toBeVisible();
    const escolhido = await som.inputValue();
    await page.evaluate(() => {
      window.location.hash = "#/modos";
    });
    await page.evaluate(() => {
      window.location.hash = "#/config/geral";
    });
    await expect(page.getByLabel("Som", { exact: true })).toHaveValue(escolhido);
    const interruptor = page.getByRole("switch", { name: "Sons de início e fim" });
    await interruptor.setChecked(false, { force: true });
    await expect(page.getByText("Preferência salva")).toBeVisible();
    await page.evaluate(() => {
      window.location.hash = "#/modos";
    });
    await page.evaluate(() => {
      window.location.hash = "#/config/geral";
    });
    await expect(page.getByRole("switch", { name: "Sons de início e fim" })).not.toBeChecked();
  });
});
