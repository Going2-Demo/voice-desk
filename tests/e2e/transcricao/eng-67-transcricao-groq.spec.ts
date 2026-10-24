import { expect, test } from "@playwright/test";
import { emitir, instalarTauri } from "../ditado/tauri-mock";

const aviso = "Cadastre a chave do Groq em Configurações › Provedores e chaves.";

test.describe("ENG-67 · F-02 transcrição na nuvem (interface do gravador, núcleo simulado)", () => {
  test("sem chave do Groq: avisa antes de gravar e oferece o atalho para as chaves", async ({
    page,
  }) => {
    let aberto = 0;
    await instalarTauri(page, {
      ditado_abrir_chaves: () => {
        aberto += 1;
        return null;
      },
    });
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", {
      fase: "erro",
      mensagem: aviso,
      segundos: 0,
      acao: "abrir_chaves",
    });
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "erro");
    await expect(page.getByRole("status")).toContainText("Cadastre a chave do Groq");
    await expect(page.getByRole("meter")).toHaveCount(0);
    await page.getByRole("button", { name: "Abrir chaves" }).click();
    await expect.poll(() => aberto).toBe(1);
  });

  test("RF-02.3 falha de transcrição mostra o motivo, sem atalho de chaves", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", {
      fase: "erro",
      mensagem: "Sem conexão com o provedor.",
      segundos: 0,
    });
    await expect(page.getByRole("status")).toContainText("Sem conexão com o provedor");
    await expect(page.getByRole("button")).toHaveCount(0);
  });
});
