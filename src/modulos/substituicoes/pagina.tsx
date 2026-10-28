import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useRef, useState } from "react";
import { consultas } from "../../ipc/consultas";
import { Interruptor } from "../../compartilhado/ui/interruptor";
import { api, type Substituicao } from "./api";
import { ErroSubstituicoes } from "./erro";
import "../../compartilhado/tema/componentes.css";
import "../../compartilhado/tema/formulario.css";
import "./substituicoes.css";

const mensagemDe = (e: unknown) =>
  e instanceof Error ? e.message : "Algo deu errado. Tente de novo.";

export default function Pagina() {
  const cliente = useQueryClient();
  const [original, setOriginal] = useState("");
  const [substituto, setSubstituto] = useState("");
  const [erro, setErro] = useState<{ campo?: string; mensagem: string } | null>(null);
  const [editando, setEditando] = useState<Substituicao | null>(null);
  const [aviso, setAviso] = useState("");
  const campoOriginal = useRef<HTMLInputElement>(null);

  const lista = useQuery({ queryKey: consultas.substituicoes.lista, queryFn: () => api.listar() });
  const recarregar = () => cliente.invalidateQueries({ queryKey: consultas.substituicoes.lista });

  const limparFormulario = () => {
    setOriginal("");
    setSubstituto("");
    setErro(null);
    setEditando(null);
    campoOriginal.current?.focus();
  };

  const salvar = useMutation({
    mutationFn: () =>
      editando
        ? api.editar(editando.id, original, substituto)
        : api.adicionar(original, substituto),
    onSuccess: () => {
      setAviso("Substituição salva.");
      limparFormulario();
      recarregar();
    },
    onError: (e) =>
      setErro({
        campo: e instanceof ErroSubstituicoes ? e.campo : undefined,
        mensagem: mensagemDe(e),
      }),
  });
  const ativar = useMutation({
    mutationFn: ({ s, ativa }: { s: Substituicao; ativa: boolean }) => api.ativar(s.id, ativa),
    onSuccess: (s) => {
      setAviso(s.ativa ? "Substituição ativada." : "Substituição desativada.");
      recarregar();
    },
    onError: (e) => setAviso(mensagemDe(e)),
  });
  const remover = useMutation({
    mutationFn: (s: Substituicao) => api.remover(s.id),
    onSuccess: () => {
      setAviso("Substituição removida.");
      if (editando) limparFormulario();
      recarregar();
    },
    onError: (e) => setAviso(mensagemDe(e)),
  });

  const enviar = (e: FormEvent) => {
    e.preventDefault();
    if (!original.trim()) {
      setErro({ campo: "original", mensagem: "Digite o que a transcrição costuma dizer." });
      return;
    }
    if (!substituto.trim()) {
      setErro({ campo: "substituto", mensagem: "Digite o texto que deve ser escrito no lugar." });
      return;
    }
    salvar.mutate();
  };

  const comecarEdicao = (s: Substituicao) => {
    setEditando(s);
    setOriginal(s.original);
    setSubstituto(s.substituto);
    setErro(null);
    campoOriginal.current?.focus();
  };

  const itens = lista.data;
  const vazio = itens?.length === 0;
  const erroOriginal = erro && erro.campo !== "substituto" ? erro.mensagem : "";
  const erroSubstituto = erro?.campo === "substituto" ? erro.mensagem : "";

  return (
    <section className="substituicoes formulario-vd">
      <h1>Substituições</h1>
      <p className="subtitulo">
        Troca uma palavra ou expressão por outra em todo ditado. Não diferencia maiúsculas de
        minúsculas.
      </p>
      {lista.isPending && <p role="status">Carregando…</p>}
      {lista.isError && (
        <p role="alert">
          Não consegui ler as substituições.{" "}
          <button type="button" className="btn" onClick={() => lista.refetch()}>
            Tentar de novo
          </button>
        </p>
      )}
      {itens && (
        <>
          <div className="cartao">
            <form onSubmit={enviar} noValidate>
              <div className="duas-col">
                <div className="campo">
                  <label htmlFor="s-de">Quando a transcrição disser</label>
                  <input
                    id="s-de"
                    ref={campoOriginal}
                    type="text"
                    autoComplete="off"
                    placeholder="Ex.: gou tu"
                    value={original}
                    aria-invalid={erroOriginal ? true : undefined}
                    aria-describedby="s-erro-de"
                    onChange={(e) => {
                      setOriginal(e.target.value);
                      setErro(null);
                    }}
                  />
                  <span className="erro-campo" id="s-erro-de" role="alert">
                    {erroOriginal}
                  </span>
                </div>
                <div className="campo">
                  <label htmlFor="s-para">Escrever</label>
                  <input
                    id="s-para"
                    type="text"
                    autoComplete="off"
                    placeholder="Ex.: Going2"
                    value={substituto}
                    aria-invalid={erroSubstituto ? true : undefined}
                    aria-describedby="s-erro-para"
                    onChange={(e) => {
                      setSubstituto(e.target.value);
                      setErro(null);
                    }}
                  />
                  <span className="erro-campo" id="s-erro-para" role="alert">
                    {erroSubstituto}
                  </span>
                </div>
              </div>
              <div className="acoes">
                <button type="submit" className="btn primario" disabled={salvar.isPending}>
                  {editando ? "Salvar substituição" : "Adicionar substituição"}
                </button>
                {editando && (
                  <button type="button" className="btn" onClick={limparFormulario}>
                    Cancelar edição
                  </button>
                )}
              </div>
            </form>
          </div>
          {vazio ? (
            <div className="vazio">
              <h2>Nenhuma substituição</h2>
              <p className="suave">
                Use para corrigir erros que se repetem, como siglas faladas por extenso.
              </p>
            </div>
          ) : (
            <div className="tabela-wrap">
              <table aria-label="Substituições">
                <thead>
                  <tr>
                    <th scope="col">Original</th>
                    <th scope="col">Substituto</th>
                    <th scope="col">Ativa</th>
                    <th scope="col">
                      <span className="so-leitor">Ações</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {itens.map((s) => (
                    <tr key={s.id} className={s.ativa ? undefined : "desativada"}>
                      <td>{s.original}</td>
                      <td>
                        <b>{s.substituto}</b>
                      </td>
                      <td>
                        <span className="so-leitor" id={`s-${s.id}-t`}>
                          Ativar “{s.original}”
                        </span>
                        <Interruptor
                          id={`s-${s.id}`}
                          ligado={s.ativa}
                          desabilitado={ativar.isPending}
                          aoMudar={(ativa) => ativar.mutate({ s, ativa })}
                        />
                      </td>
                      <td>
                        <div className="acoes">
                          <button
                            type="button"
                            className="btn"
                            aria-label={`Editar ${s.original}`}
                            onClick={() => comecarEdicao(s)}
                          >
                            Editar
                          </button>
                          <button
                            type="button"
                            className="btn"
                            aria-label={`Remover ${s.original}`}
                            onClick={() => remover.mutate(s)}
                          >
                            Remover
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <p className="so-leitor" role="status" aria-live="polite">
            {aviso}
          </p>
        </>
      )}
    </section>
  );
}
