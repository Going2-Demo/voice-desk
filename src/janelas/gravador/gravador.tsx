import { useEffect, useState } from "react";
import { type Estado, type PonteDitado, ponteReal } from "../../ipc/ponte-ditado";
import "./gravador.css";

const BARRAS = 9;

export function formatarTempo(segundos: number): string {
  const m = Math.floor(segundos / 60);
  const s = String(segundos % 60).padStart(2, "0");
  return `${m}:${s}`;
}

/** Pílula escura do gravador (design 2.2): estado, nível do áudio, mensagem e tempo. */
export function Gravador({ ponte = ponteReal }: { ponte?: PonteDitado }) {
  const [estado, setEstado] = useState<Estado | null>(null);
  const [nivel, setNivel] = useState(0);

  useEffect(() => {
    let vivo = true;
    const cancelar: Array<() => void> = [];
    const guardar = (c: () => void) => (vivo ? cancelar.push(c) : c());
    void ponte
      .aoEstado((e) => {
        if (e.fase !== "gravando") setNivel(0);
        setEstado(e);
      })
      .then(guardar);
    void ponte.aoNivel(setNivel).then(guardar);
    return () => {
      vivo = false;
      for (const c of cancelar) c();
    };
  }, [ponte]);

  if (!estado) return null;
  const gravando = estado.fase === "gravando";
  return (
    <div className="gravador" data-fase={estado.fase}>
      <span className="ponto" aria-hidden="true" />
      {gravando && (
        <>
          <meter
            className="so-leitor"
            aria-label="Nível do áudio"
            min={0}
            max={100}
            value={Math.round(nivel * 100)}
          />
          <span className="nivel" aria-hidden="true">
            {Array.from({ length: BARRAS }, (_, i) => {
              const ativa = nivel * BARRAS > i;
              return (
                <i
                  // biome-ignore lint/suspicious/noArrayIndexKey: barras fixas, sem identidade própria
                  key={i}
                  style={{
                    height: ativa ? `${6 + ((i % 3) + 1) * nivel * 5 + nivel * 10}px` : "4px",
                  }}
                />
              );
            })}
          </span>
        </>
      )}
      <span className="estado" role="status">
        {estado.mensagem}
      </span>
      {estado.acao === "abrir_chaves" && (
        <button type="button" className="acao" onClick={() => void ponte.abrirChaves()}>
          Abrir chaves
        </button>
      )}
      {gravando && <span className="tempo">{formatarTempo(estado.segundos)}</span>}
    </div>
  );
}
