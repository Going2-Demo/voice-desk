import { render, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ApiPrimeiroUso } from "./api";
import { IrParaPrimeiroUso } from "./ir-para-primeiro-uso";

const api = (concluido: boolean): ApiPrimeiroUso => ({
  estado: vi.fn().mockResolvedValue({ concluido, pulados: [] }),
  pular: vi.fn(),
  cumprir: vi.fn(),
  concluir: vi.fn(),
});

function desenhar(a: ApiPrimeiroUso, inicial = "/modos") {
  const { container } = render(
    <MemoryRouter initialEntries={[inicial]}>
      <IrParaPrimeiroUso api={a} />
      <Routes>
        <Route path="/modos" element={<p>modos</p>} />
        <Route path="/primeiro-uso" element={<p>primeiro uso</p>} />
      </Routes>
    </MemoryRouter>,
  );
  return container;
}

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});
afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
});

describe("primeira abertura", () => {
  it("leva ao primeiro uso quando ainda não foi concluído", async () => {
    const c = desenhar(api(false));
    await waitFor(() => expect(c).toHaveTextContent("primeiro uso"));
  });

  it("não aparece de novo depois de concluído", async () => {
    const a = api(true);
    const c = desenhar(a);
    await waitFor(() => expect(a.estado).toHaveBeenCalled());
    expect(c).toHaveTextContent("modos");
  });

  it("não consulta nada fora do app nem quando já está na tela", () => {
    const a = api(false);
    desenhar(a, "/primeiro-uso");
    expect(a.estado).not.toHaveBeenCalled();
  });

  it("se não conseguir ler o estado, deixa o usuário onde está", async () => {
    const a = api(false);
    a.estado = vi.fn().mockRejectedValue(new Error("falhou"));
    const c = desenhar(a);
    await waitFor(() => expect(a.estado).toHaveBeenCalled());
    expect(c).toHaveTextContent("modos");
  });
});
