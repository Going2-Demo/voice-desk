import { useEffect, useId, useState } from "react";
import type { PreferenciaSons } from "../../../ipc/bindings";
import "../../../compartilhado/tema/componentes.css";
import "../../../compartilhado/tema/formulario.css";
import { Interruptor } from "../../../compartilhado/ui/interruptor";
import { api } from "./api";

const MENSAGEM_FALHA = "Não foi possível salvar a preferência. Tente de novo.";
const TEMPO_TOAST_MS = 5000;

type Preferencia = Pick<PreferenciaSons, "ligado" | "som"> & {
  opcoes: { id: string; nome: string }[];
};

/** RF-24.1: liga ou desliga os sons de início e fim, escolhe entre 3 e ouve a prévia. */
export function Secao() {
  const id = useId();
  const [pref, setPref] = useState<Preferencia | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const [salvo, setSalvo] = useState(false);

  useEffect(() => {
    let ativo = true;
    api.sonsObter().then((r) => {
      if (!ativo) return;
      if (r.status === "ok") setPref(r.data);
      else setErro(r.error.mensagem);
    });
    return () => {
      ativo = false;
    };
  }, []);

  useEffect(() => {
    if (!salvo) return;
    const t = setTimeout(() => setSalvo(false), TEMPO_TOAST_MS);
    return () => clearTimeout(t);
  }, [salvo]);

  const salvar = async (ligado: boolean, som: string) => {
    setErro(null);
    const r = await api.sonsDefinir(ligado, som);
    if (r.status === "ok") {
      setPref(r.data);
      setSalvo(true);
    } else setErro(r.error.mensagem || MENSAGEM_FALHA);
  };

  const ouvir = async () => {
    if (!pref?.ligado) return;
    setErro(null);
    const r = await api.sonsPrevia(pref.som);
    if (r.status !== "ok") setErro(r.error.mensagem || MENSAGEM_FALHA);
  };

  const ligado = pref?.ligado ?? false;

  return (
    <section className="cartao formulario-vd" aria-labelledby={`${id}-titulo`}>
      <h2 id={`${id}-titulo`}>Durante o ditado</h2>
      <div className="linha-opcao">
        <div className="texto">
          <b id={`${id}-t`}>Sons de início e fim</b>
          <span className="dica">
            Um aviso curto antes de gravar e ao terminar. O som de início não entra na gravação.
          </span>
        </div>
        <Interruptor
          id={id}
          ligado={ligado}
          desabilitado={pref === null}
          aoMudar={(novo) => pref && salvar(novo, pref.som)}
        />
      </div>
      <div className="campo">
        <label htmlFor={`${id}-som`}>Som</label>
        <div className="acoes">
          <select
            id={`${id}-som`}
            value={pref?.som ?? ""}
            disabled={!ligado}
            onChange={(e) => salvar(true, e.target.value)}
          >
            {pref?.opcoes.map((o) => (
              <option key={o.id} value={o.id}>
                {o.nome}
              </option>
            ))}
          </select>
          <button type="button" className="btn" disabled={!ligado} onClick={ouvir}>
            ▶ Ouvir prévia
          </button>
        </div>
      </div>
      {erro && (
        <p role="alert" className="aviso-erro">
          {erro}
        </p>
      )}
      {salvo && (
        <div className="toasts">
          <div className="toast" role="status" aria-live="polite">
            Preferência salva
          </div>
        </div>
      )}
    </section>
  );
}
