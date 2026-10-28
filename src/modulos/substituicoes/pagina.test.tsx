import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { readFileSync } from "node:fs";
import { beforeEach, describe, expect, it, vi } from "vitest";
vi.hoisted(() => {
  import.meta.env.VITE_IPC = "simulado";
});

import Pagina from "./pagina";
import { reiniciarSimulado } from "./simulado";

function desenhar() {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={cliente}>
      <Pagina />
    </QueryClientProvider>,
  );
}

const preencher = async (de: string, para: string) => {
  const campoDe = await screen.findByLabelText("Quando a transcrição disser");
  await userEvent.clear(campoDe);
  if (de) await userEvent.type(campoDe, de);
  const campoPara = screen.getByLabelText("Escrever");
  await userEvent.clear(campoPara);
  if (para) await userEvent.type(campoPara, para);
};

beforeEach(() =>
  reiniciarSimulado([
    ["gou tu", "Going2", true],
    ["pix", "PIX", false],
  ]),
);

describe("Substituições (F-08)", () => {
  it("lista os pares com original, substituto e estado", async () => {
    desenhar();
    const tabela = await screen.findByRole("table", { name: "Substituições" });
    expect(within(tabela).getByText("gou tu")).toBeInTheDocument();
    expect(within(tabela).getByText("Going2")).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: "Ativar “gou tu”" })).toBeChecked();
    expect(screen.getByRole("switch", { name: "Ativar “pix”" })).not.toBeChecked();
  });

  it("cria uma substituição e ela aparece na lista", async () => {
    desenhar();
    await preencher("vois desk", "Voice Desk");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar substituição" }));
    expect(await screen.findByText("vois desk")).toBeInTheDocument();
    expect(screen.getByText("Voice Desk")).toBeInTheDocument();
    expect(screen.getByLabelText("Quando a transcrição disser")).toHaveValue("");
  });

  it("original vazio é recusado com aviso e nada é salvo", async () => {
    desenhar();
    await preencher("", "PIX2");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar substituição" }));
    expect(
      await screen.findByText("Digite o que a transcrição costuma dizer."),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("row")).toHaveLength(3);
  });

  it("duplicada sem diferenciar maiúsculas é recusada", async () => {
    desenhar();
    await preencher("GOU TU", "outro");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar substituição" }));
    expect(
      await screen.findByText("Já existe uma substituição para “GOU TU”."),
    ).toBeInTheDocument();
  });

  it("ativa e desativa pelo interruptor", async () => {
    desenhar();
    const pix = await screen.findByRole("switch", { name: "Ativar “pix”" });
    await userEvent.click(pix);
    expect(await screen.findByRole("switch", { name: "Ativar “pix”" })).toBeChecked();
    await userEvent.click(screen.getByRole("switch", { name: "Ativar “pix”" }));
    expect(await screen.findByRole("switch", { name: "Ativar “pix”" })).not.toBeChecked();
  });

  it("edita um par e cancela a edição", async () => {
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Editar gou tu" }));
    expect(screen.getByLabelText("Quando a transcrição disser")).toHaveValue("gou tu");
    await preencher("gou tu", "Going2 Tech");
    await userEvent.click(screen.getByRole("button", { name: "Salvar substituição" }));
    expect(await screen.findByText("Going2 Tech")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Editar pix" }));
    await userEvent.click(screen.getByRole("button", { name: "Cancelar edição" }));
    expect(screen.getByRole("button", { name: "Adicionar substituição" })).toBeInTheDocument();
  });

  it("remove um par e mostra o estado vazio quando não sobra nenhum", async () => {
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Remover gou tu" }));
    await userEvent.click(await screen.findByRole("button", { name: "Remover pix" }));
    expect(await screen.findByText("Nenhuma substituição")).toBeInTheDocument();
  });

  it("o interruptor da tabela é block, senão colapsa a 0×0 (retorno 1 do QA)", () => {
    const css = readFileSync(new URL("./substituicoes.css", import.meta.url), "utf8");
    expect(css).toMatch(/\.substituicoes td \.interruptor\s*\{[^}]*display:\s*block/);
  });
});
