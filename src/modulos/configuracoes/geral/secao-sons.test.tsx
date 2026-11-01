import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { obter, definir, previa } = vi.hoisted(() => ({
  obter: vi.fn(),
  definir: vi.fn(),
  previa: vi.fn(),
}));
vi.mock("./api", () => ({
  api: { sonsObter: obter, sonsDefinir: definir, sonsPrevia: previa },
}));

import { Secao } from "./secao-sons";

const OPCOES = [
  { id: "gota", nome: "Gota" },
  { id: "pulso", nome: "Pulso" },
  { id: "clique_suave", nome: "Clique suave" },
];
const pref = (ligado: boolean, som = "gota") => ({
  status: "ok" as const,
  data: { ligado, som, opcoes: OPCOES },
});

beforeEach(() => {
  obter.mockReset().mockResolvedValue(pref(true));
  definir.mockReset();
  previa.mockReset().mockResolvedValue({ status: "ok", data: null });
});

describe("sons de início e fim (RF-24.1)", () => {
  it("vem ligado e oferece pelo menos 3 sons", async () => {
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /sons de início e fim/i });
    await waitFor(() => expect(chave).toBeChecked());
    expect(screen.getAllByRole("option")).toHaveLength(3);
  });

  it("desliga e grava; com os sons desligados, escolha e prévia ficam desabilitadas", async () => {
    definir.mockResolvedValue(pref(false));
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /sons de início e fim/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(definir).toHaveBeenCalledWith(false, "gota");
    await waitFor(() => expect(chave).not.toBeChecked());
    expect(screen.getByRole("combobox", { name: "Som" })).toBeDisabled();
    expect(screen.getByRole("button", { name: /ouvir prévia/i })).toBeDisabled();
    expect(await screen.findByText("Preferência salva")).toBeInTheDocument();
  });

  it("escolher outro som grava a escolha", async () => {
    definir.mockResolvedValue(pref(true, "pulso"));
    render(<Secao />);
    const seletor = await screen.findByRole("combobox", { name: "Som" });
    await waitFor(() => expect(seletor).toBeEnabled());
    fireEvent.change(seletor, { target: { value: "pulso" } });
    expect(definir).toHaveBeenCalledWith(true, "pulso");
    await waitFor(() => expect(seletor).toHaveValue("pulso"));
  });

  it("a prévia toca o som escolhido", async () => {
    render(<Secao />);
    const botao = await screen.findByRole("button", { name: /ouvir prévia/i });
    await waitFor(() => expect(botao).toBeEnabled());
    fireEvent.click(botao);
    await waitFor(() => expect(previa).toHaveBeenCalledWith("gota"));
  });

  it("mostra o erro e mantém o estado quando a gravação falha", async () => {
    definir.mockResolvedValue({
      status: "error",
      error: { codigo: "interno", mensagem: "Não foi possível salvar." },
    });
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /sons de início e fim/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(await screen.findByRole("alert")).toHaveTextContent("Não foi possível salvar.");
    expect(chave).toBeChecked();
  });
});
