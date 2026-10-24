import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";
import { instalarTauri } from "../ditado/tauri-mock";

const telas = [
  "/config/geral",
  "/config/atalhos",
  "/config/provedores",
  "/config/local",
  "/config/dados",
  "/config/permissoes",
];

test.describe("ENG-93 · sem rolagem horizontal abaixo de 720 px", () => {
  for (const largura of [390, 719]) {
    for (const tela of telas) {
      test(`${tela} em ${largura} px não rola na horizontal`, async ({ page }) => {
        await instalarTauri(page, {});
        await page.setViewportSize({ width: largura, height: 800 });
        await irPara(page, tela);
        await expect(page.getByRole("navigation", { name: "Principal" })).toBeVisible();
        const medidas = await page.evaluate(() => {
          const largura = (seletor: string) => document.querySelector(seletor)?.scrollWidth ?? 0;
          return {
            janela: document.documentElement.clientWidth,
            pagina: document.documentElement.scrollWidth,
            conteudo: largura("main.conteudo"),
            lateral: document.querySelector(".lateral")?.getBoundingClientRect().width ?? 0,
          };
        });
        expect(medidas.pagina).toBeLessThanOrEqual(medidas.janela);
        expect(medidas.conteudo).toBeLessThanOrEqual(medidas.janela);
        expect(medidas.lateral).toBeLessThanOrEqual(medidas.janela);
      });
    }
  }
});
