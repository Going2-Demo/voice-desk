import { describe, expect, it } from "vitest";
import { limpar } from "./algoritmo";

const lista = ["é…", "hã", "hum", "né", "tipo", "uh", "um…", "you know"];

describe("limpeza do modo navegador (espelho do núcleo)", () => {
  it("modo Chat: remoção e formatação ligadas", () => {
    expect(limpar("hã… chego em dez minutos", lista, true, true)).toBe("Chego em dez minutos.");
  });
  it("Ditado literal: tudo desligado não altera nada", () => {
    expect(limpar("é…  eu acho que sim", lista, false, false)).toBe("é…  eu acho que sim");
  });
  it("tipo na lista: “é tipo assim” vira “É assim.”", () => {
    expect(limpar("é tipo assim", lista, true, true)).toBe("É assim.");
  });
  it("formatação tira espaços duplos e põe ponto", () => {
    expect(limpar("olá  tudo bem", [], false, true)).toBe("Olá tudo bem.");
  });
  it("interrogação da muleta passa para a palavra anterior", () => {
    expect(limpar("você vai, né?", lista, true, true)).toBe("Você vai?");
  });
  it("palavra inteira apenas", () => {
    expect(limpar("humor humano um dia", lista, true, false)).toBe("humor humano um dia");
  });
});
