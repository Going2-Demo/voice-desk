import type { Page } from "@playwright/test";

/**
 * Abre uma tela do app no modo simulado sem cair no primeiro uso (F-06): na primeira abertura o app
 * leva ao primeiro uso uma vez por execução; abrindo já nele e trocando a rota depois, a verificação
 * não repete. `rota` é o caminho do app, ex.: "/config/geral".
 */
export async function irPara(page: Page, rota: string) {
  await page.goto("/#/primeiro-uso");
  await page.getByRole("heading", { level: 1, name: "Primeiro uso" }).waitFor();
  await page.evaluate((r) => {
    window.location.hash = `#${r}`;
  }, rota);
}
