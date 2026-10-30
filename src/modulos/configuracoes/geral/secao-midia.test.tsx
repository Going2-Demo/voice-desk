import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { obter, definir } = vi.hoisted(() => ({ obter: vi.fn(), definir: vi.fn() }));
vi.mock("./api", () => ({ api: { midiaPausarObter: obter, midiaPausarDefinir: definir } }));

import { Secao } from "./secao-midia";

beforeEach(() => {
  obter.mockReset().mockResolvedValue({ status: "ok", data: true });
  definir.mockReset();
});

describe("pausar a mídia durante o ditado (RF-25.1)", () => {
  it("vem ligado por padrão", async () => {
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /pausar a mídia/i });
    await waitFor(() => expect(chave).toBeChecked());
  });

  it("desliga e grava a escolha", async () => {
    definir.mockResolvedValue({ status: "ok", data: false });
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /pausar a mídia/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(definir).toHaveBeenCalledWith(false);
    await waitFor(() => expect(chave).not.toBeChecked());
    expect(await screen.findByText("Preferência salva")).toBeInTheDocument();
  });

  it("mostra o erro e mantém o estado quando a gravação falha", async () => {
    definir.mockResolvedValue({
      status: "error",
      error: { codigo: "interno", mensagem: "Não foi possível salvar." },
    });
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /pausar a mídia/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(await screen.findByRole("alert")).toHaveTextContent("Não foi possível salvar.");
    expect(chave).toBeChecked();
  });
});
