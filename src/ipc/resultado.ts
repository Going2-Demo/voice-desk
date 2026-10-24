type Resultado<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: { codigo: string; mensagem: string; campo?: string | null } };

/** Devolve o dado do comando ou lança um `Error` com a mensagem já em português do núcleo. */
export async function desembrulhar<T>(r: Promise<Resultado<T>>): Promise<T> {
  const resposta = await r;
  if (resposta.status === "error") throw new Error(resposta.error.mensagem);
  return resposta.data;
}
