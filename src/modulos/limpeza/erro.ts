/** Erro de comando já em português (`{ codigo, mensagem, campo? }`). */
export class ErroLimpeza extends Error {
  constructor(
    readonly codigo: string,
    mensagem: string,
    readonly campo?: string,
  ) {
    super(mensagem);
  }
}
