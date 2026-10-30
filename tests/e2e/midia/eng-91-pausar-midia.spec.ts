import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";

// ENG-91 / F-25 (modo simulado). A pausa real da mídia (Spotify, SMTC, MediaRemote) é nativa:
// ver tests/nativo/pendencias.md.

test.describe("F-25 pausar a mídia durante o ditado", () => {
  test("a seção Mídia existe em Configurações › Geral, com a pausa ligada por padrão (RF-25.1)", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    const secao = page.getByRole("region", { name: "Mídia" });
    await expect(secao).toBeVisible();
    await expect(secao.getByText("Pausar a mídia")).toBeVisible();
    await expect(secao.getByRole("switch")).toBeChecked();
  });

  test("desligar e religar a pausa persiste na sessão (RF-25.1)", async ({ page }) => {
    await irPara(page, "/config/geral");
    const interruptor = page.getByRole("region", { name: "Mídia" }).getByRole("switch");
    await expect(interruptor).toBeChecked();
    await interruptor.click();
    await expect(interruptor).not.toBeChecked();

    await irPara(page, "/bandeja");
    await irPara(page, "/config/geral");
    await expect(page.getByRole("region", { name: "Mídia" }).getByRole("switch")).not.toBeChecked();

    await page.getByRole("region", { name: "Mídia" }).getByRole("switch").click();
    await expect(page.getByRole("region", { name: "Mídia" }).getByRole("switch")).toBeChecked();
  });
});
