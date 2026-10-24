import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

const rotas = ["/config/geral", "/modos", "/provedores", "/config/chaves"];

test.describe("ENG-94 casca recolhe a lateral abaixo de 720 px", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  for (const rota of rotas) {
    test(`sem rolagem horizontal em ${rota}`, async ({ page }) => {
      await irPara(page, rota);
      await page.waitForLoadState("networkidle");
      const { sw, cw } = await page.evaluate(() => ({
        sw: document.documentElement.scrollWidth,
        cw: document.documentElement.clientWidth,
      }));
      expect(sw).toBeLessThanOrEqual(cw);
    });
  }

  test("lateral vira barra horizontal no topo, com links de 44 px", async ({ page }) => {
    await irPara(page, "/config/geral");
    const nav = page.getByRole("navigation").first();
    const nb = await nav.boundingBox();
    expect(nb?.width ?? 999).toBeLessThanOrEqual(390);
    expect(nb?.y ?? 999).toBeLessThan(120);
    const links = nav.getByRole("link");
    const n = await links.count();
    expect(n).toBeGreaterThan(0);
    for (let i = 0; i < n; i++) {
      const b = await links.nth(i).boundingBox();
      expect(b?.height ?? 0).toBeGreaterThanOrEqual(44);
    }
  });

  test("botões das telas com altura mínima de 44 px", async ({ page }) => {
    await irPara(page, "/config/chaves");
    for (const b of await page.getByRole("button").all()) {
      if (!(await b.isVisible())) continue;
      expect((await b.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
    }
  });
});

test("em janela larga a lateral continua fixa à esquerda", async ({ page }) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await irPara(page, "/config/geral");
  const nb = await page.getByRole("navigation").first().boundingBox();
  expect(nb?.x).toBe(0);
  expect(nb?.width ?? 0).toBeLessThan(300);
});
