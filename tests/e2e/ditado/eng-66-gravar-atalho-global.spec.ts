import { expect, test } from "@playwright/test";
import { emitir, instalarTauri } from "./tauri-mock";

const atalhoPadrao = {
  combinacao: "Ctrl+Alt+Space",
  exibicao: "Ctrl+Alt+Espaço",
  padrao: "Ctrl+Alt+Space",
};
const estado = (fase: string, mensagem: string, segundos = 0) => ({ fase, mensagem, segundos });

test.describe("ENG-66 · F-01 gravador flutuante (interface, núcleo simulado)", () => {
  test("gravando: mostra estado, nível do áudio e tempo; depois processa e some", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await expect(page.getByRole("status")).toHaveCount(0);

    await emitir(page, "ditado-estado", estado("gravando", "Gravando…", 3));
    await expect(page.getByRole("status")).toHaveText("Gravando…");
    await expect(page.getByText("0:03")).toBeVisible();
    const nivel = page.getByRole("meter", { name: "Nível do áudio" });
    await emitir(page, "ditado-nivel", 0.6);
    await expect(nivel).toHaveAttribute("value", "60");

    await emitir(page, "ditado-estado", estado("processando", "Processando…"));
    await expect(page.getByRole("status")).toHaveText("Processando…");
    await expect(nivel).toHaveCount(0);
    await expect(page.getByText("0:03")).toHaveCount(0);
  });

  test("RF-01.5 aviso aos 9:30 e RF-01.6 'aguarde' durante o processamento", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(
      page,
      "ditado-estado",
      estado("aviso", "Faltam 30 s para o limite de 10 min", 570),
    );
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "aviso");
    await expect(page.getByRole("status")).toContainText("30 s");
    await emitir(page, "ditado-estado", estado("aviso", "Aguarde, ainda processando"));
    await expect(page.getByRole("status")).toContainText(/aguarde/i);
  });

  test("microfone sem permissão: mostra a mensagem e o caminho para conceder", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(
      page,
      "ditado-estado",
      estado("erro", "Sem acesso ao microfone. Conceda em Ajustes › Privacidade › Microfone."),
    );
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "erro");
    await expect(page.getByRole("status")).toContainText(/sem acesso ao microfone/i);
    await expect(page.getByRole("meter")).toHaveCount(0);
  });

  test("Esc duplo cancela: gravador some de vista sem texto a colar", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("gravando", "Gravando…", 1));
    await emitir(page, "ditado-estado", estado("ok", "Cancelado"));
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "ok");
  });
});

test.describe("ENG-66 · RF-01.1 atalho nas configurações", () => {
  test("mostra o atalho atual e salva um válido com aviso de que vale sem reiniciar", async ({
    page,
  }) => {
    await instalarTauri(page, {
      atalhos_obter: atalhoPadrao,
      atalhos_salvar: () => ({
        combinacao: "Ctrl+Shift+D",
        exibicao: "Ctrl+Shift+D",
        padrao: "Ctrl+Alt+Space",
      }),
    });
    await page.goto("/#/config/atalhos");
    const campo = page.getByLabel("Atalho");
    await expect(campo).toHaveValue("Ctrl+Alt+Espaço");
    await campo.fill("Ctrl+Shift+D");
    await page.getByRole("button", { name: "Salvar atalho" }).click();
    await expect(page.getByRole("status").filter({ hasText: "Atalho salvo" })).toContainText(
      "sem reiniciar",
    );
    await expect(campo).toHaveValue("Ctrl+Shift+D");
  });

  test("atalho recusado pelo núcleo mostra a mensagem e não marca como salvo", async ({ page }) => {
    await instalarTauri(page, {
      atalhos_obter: atalhoPadrao,
      atalhos_salvar: () => {
        throw { mensagem: "Escolha uma combinação com pelo menos uma tecla modificadora." };
      },
    });
    await page.goto("/#/config/atalhos");
    await page.getByLabel("Atalho").fill("A");
    await page.getByRole("button", { name: "Salvar atalho" }).click();
    await expect(page.getByRole("alert")).toContainText("modificadora");
    await expect(page.getByText("Atalho salvo")).toHaveCount(0);
  });

  test("campo vazio é recusado na tela, sem chamar o núcleo", async ({ page }) => {
    let chamadas = 0;
    await instalarTauri(page, {
      atalhos_obter: atalhoPadrao,
      atalhos_salvar: () => {
        chamadas++;
        return atalhoPadrao;
      },
    });
    await page.goto("/#/config/atalhos");
    await page.getByLabel("Atalho").fill("");
    await page.getByRole("button", { name: "Salvar atalho" }).click();
    await expect(page.getByRole("alert")).toContainText("Digite uma combinação");
    expect(chamadas).toBe(0);
  });

  test("informa o Esc duplo (RF-01.4) e a regra de tecla modificadora", async ({ page }) => {
    await instalarTauri(page, { atalhos_obter: atalhoPadrao });
    await page.goto("/#/config/atalhos");
    await expect(page.getByText("pelo menos uma tecla modificadora")).toBeVisible();
    await expect(page.getByText("Cancelar o ditado")).toBeVisible();
  });
});
