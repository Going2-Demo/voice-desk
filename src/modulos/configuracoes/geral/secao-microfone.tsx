import { useEffect, useId, useState } from "react";
import type { EstadoMicrofone } from "../../../ipc/bindings";
import "../../../compartilhado/tema/componentes.css";
import "../../../compartilhado/tema/formulario.css";
import { api } from "./api";

const PADRAO = "";
const TEMPO_TOAST_MS = 5000;

/** F-23: escolha do microfone de entrada; vale por máquina e não sincroniza. */
export function Secao() {
  const id = useId();
  const [estado, setEstado] = useState<EstadoMicrofone | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const [salvo, setSalvo] = useState(false);

  useEffect(() => {
    let ativo = true;
    api.audioMicrofoneEstado().then((r) => {
      if (!ativo) return;
      if (r.status === "ok") setEstado(r.data);
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

  const escolher = async (valor: string) => {
    setErro(null);
    const r = await api.audioDefinirMicrofone(valor === PADRAO ? null : valor);
    if (r.status === "ok") {
      setEstado(r.data);
      setSalvo(true);
    } else setErro(r.error.mensagem);
  };

  const escolhido = estado?.escolhido ?? null;
  // Um microfone escolhido e desconectado continua visível na lista, para o usuário ver o que ficou.
  const opcoes = estado?.dispositivos ?? [];
  const ausente = escolhido !== null && !opcoes.includes(escolhido);

  return (
    <section className="cartao formulario-vd" aria-labelledby={`${id}-titulo`}>
      <h2 id={`${id}-titulo`}>Microfone</h2>
      <div className="campo">
        <label htmlFor={id}>Microfone de entrada</label>
        <select
          id={id}
          value={escolhido ?? PADRAO}
          disabled={estado === null}
          onChange={(e) => escolher(e.target.value)}
        >
          <option value={PADRAO}>Padrão do sistema</option>
          {ausente && <option value={escolhido}>{escolhido} (desconectado)</option>}
          {opcoes.map((nome) => (
            <option key={nome} value={nome}>
              {nome}
            </option>
          ))}
        </select>
        <span className="dica">
          Vale só nesta máquina. Sem o microfone escolhido, o ditado usa o padrão do sistema.
        </span>
      </div>
      {estado && escolhido !== null && !estado.conectado && (
        <p role="status" className="aviso-alerta">
          Este microfone não está conectado. Os ditados usam o padrão do sistema e o gravador avisa
          qual está em uso. Quando ele voltar, será usado de novo.
        </p>
      )}
      {estado && escolhido !== null && estado.conectado && <p role="status">Conectado</p>}
      {erro && (
        <p role="alert" className="aviso-erro">
          {erro}
        </p>
      )}
      {salvo && (
        <div className="toasts">
          <div className="toast" role="status" aria-live="polite">
            Microfone salvo
          </div>
        </div>
      )}
    </section>
  );
}
