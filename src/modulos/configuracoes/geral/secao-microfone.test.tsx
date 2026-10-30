import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { obter, definir } = vi.hoisted(() => ({ obter: vi.fn(), definir: vi.fn() }));
vi.mock("./api", () => ({
  api: { audioMicrofoneEstado: obter, audioDefinirMicrofone: definir },
}));

import { Secao } from "./secao-microfone";

const estado = (escolhido: string | null, conectado = true) => ({
  status: "ok" as const,
  data: { dispositivos: ["Interno", "Headset USB"], escolhido, conectado },
});

beforeEach(() => {
  obter.mockReset().mockResolvedValue(estado(null));
  definir.mockReset();
});

describe("escolha do microfone (F-23)", () => {
  it("lista o padrão do sistema e os dispositivos conectados (RF-23.1)", async () => {
    render(<Secao />);
    const caixa = await screen.findByRole("combobox", { name: /microfone de entrada/i });
    await waitFor(() => expect(caixa).toBeEnabled());
    const nomes = screen.getAllByRole("option").map((o) => o.textContent);
    expect(nomes).toEqual(["Padrão do sistema", "Interno", "Headset USB"]);
    expect(caixa).toHaveValue("");
  });

  it("grava a escolha e confirma", async () => {
    definir.mockResolvedValue(estado("Headset USB"));
    render(<Secao />);
    const caixa = await screen.findByRole("combobox", { name: /microfone de entrada/i });
    await waitFor(() => expect(caixa).toBeEnabled());
    fireEvent.change(caixa, { target: { value: "Headset USB" } });
    expect(definir).toHaveBeenCalledWith("Headset USB");
    expect(await screen.findByText("Microfone salvo")).toBeInTheDocument();
    expect(screen.getByText("Conectado")).toBeInTheDocument();
  });

  it("volta ao padrão do sistema enviando nulo", async () => {
    obter.mockResolvedValue(estado("Headset USB"));
    definir.mockResolvedValue(estado(null));
    render(<Secao />);
    const caixa = await screen.findByRole("combobox", { name: /microfone de entrada/i });
    await waitFor(() => expect(caixa).toHaveValue("Headset USB"));
    fireEvent.change(caixa, { target: { value: "" } });
    expect(definir).toHaveBeenCalledWith(null);
  });

  it("avisa quando o microfone escolhido está desconectado (RF-23.2)", async () => {
    obter.mockResolvedValue({
      status: "ok",
      data: { dispositivos: ["Interno"], escolhido: "Headset USB", conectado: false },
    });
    render(<Secao />);
    expect(await screen.findByText(/não está conectado/i)).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Headset USB (desconectado)" })).toBeInTheDocument();
  });

  it("mostra o erro do núcleo e mantém a escolha anterior", async () => {
    definir.mockResolvedValue({
      status: "error",
      error: { codigo: "microfone_desconectado", mensagem: "Este microfone não está conectado." },
    });
    render(<Secao />);
    const caixa = await screen.findByRole("combobox", { name: /microfone de entrada/i });
    await waitFor(() => expect(caixa).toBeEnabled());
    fireEvent.change(caixa, { target: { value: "Interno" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("não está conectado");
    expect(caixa).toHaveValue("");
  });
});
