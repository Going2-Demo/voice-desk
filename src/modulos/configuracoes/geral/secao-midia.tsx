import { useEffect, useId, useState } from "react";
import "../../../compartilhado/tema/componentes.css";
import { Interruptor } from "../../../compartilhado/ui/interruptor";
import { api } from "./api";

const MENSAGEM_FALHA = "Não foi possível salvar a preferência. Tente de novo.";
const TEMPO_TOAST_MS = 5000;

/** RF-25.1: liga ou desliga a pausa da mídia durante o ditado (padrão: ligado). */
export function Secao() {
  const id = useId();
  const [ligado, setLigado] = useState<boolean | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const [salvo, setSalvo] = useState(false);

  useEffect(() => {
    let ativo = true;
    api.midiaPausarObter().then((r) => {
      if (!ativo) return;
      if (r.status === "ok") setLigado(r.data);
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

  const alterar = async (novo: boolean) => {
    setErro(null);
    const r = await api.midiaPausarDefinir(novo);
    if (r.status === "ok") {
      setLigado(r.data);
      setSalvo(true);
    } else setErro(r.error.mensagem || MENSAGEM_FALHA);
  };

  return (
    <section className="cartao" aria-labelledby={`${id}-titulo`}>
      <h2 id={`${id}-titulo`}>Mídia</h2>
      <div className="linha-opcao">
        <div className="texto">
          <b id={`${id}-t`}>Pausar a mídia</b>
          <span className="dica">
            Pausa música e vídeo durante o ditado e só retoma o que estava tocando.
          </span>
        </div>
        <Interruptor
          id={id}
          ligado={ligado ?? false}
          desabilitado={ligado === null}
          aoMudar={alterar}
        />
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
