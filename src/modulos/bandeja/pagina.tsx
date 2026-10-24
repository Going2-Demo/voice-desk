import { useState } from "react";
import "../../compartilhado/tema/componentes.css";
import { Secao as IniciarComOSistema } from "../configuracoes/geral/secao-iniciar-com-o-sistema";
import "./bandeja.css";

const estados = [
  { id: "parado", rotulo: "Parado", simbolo: "🎙️", texto: "pronto para ditar" },
  { id: "gravando", rotulo: "Gravando", simbolo: "🔴", texto: "captando o microfone" },
  { id: "processando", rotulo: "Processando", simbolo: "⏳", texto: "transcrevendo e refinando" },
  { id: "erro", rotulo: "Erro", simbolo: "⚠️", texto: "o último ditado falhou" },
] as const;

type IdEstado = (typeof estados)[number]["id"];

/** Modos de exemplo: a lista real vem do menu do ícone quando houver modos (F-10). */
const modosExemplo = [
  { id: "ditado", rotulo: "Ditado", padrao: true },
  { id: "mensagem", rotulo: "Mensagem", padrao: false },
  { id: "email", rotulo: "E-mail", padrao: false },
];

const mac = typeof navigator !== "undefined" && /mac/i.test(navigator.userAgent);
const atalho = mac ? "⌃⌥Espaço" : "Ctrl+Alt+Espaço";
const lugar = mac ? "barra de menus" : "bandeja";

export default function Pagina() {
  const [estadoId, setEstadoId] = useState<IdEstado>("parado");
  const estado = estados.find((e) => e.id === estadoId) ?? estados[0];

  return (
    <section className="bandeja">
      <h1>Ícone e menu</h1>
      <p>
        O Voice Desk vive na {lugar} do sistema. O ícone mostra o estado e o menu dá acesso rápido
        sem abrir a janela.
      </p>
      <fieldset className="seg">
        <legend className="so-leitor">Estado do ícone</legend>
        {estados.map((e) => (
          <button
            key={e.id}
            type="button"
            aria-pressed={e.id === estadoId}
            onClick={() => setEstadoId(e.id)}
          >
            {e.rotulo}
          </button>
        ))}
      </fieldset>
      <div className="bandeja-demo">
        <div>
          <div className="barra-sistema">
            <span>Wi‑Fi</span>
            <span>🔋 82%</span>
            <span
              className="estado-icone"
              role="img"
              aria-label={`Ícone do Voice Desk: ${estado.rotulo.toLowerCase()}, ${estado.texto}`}
            >
              <b aria-hidden="true">{estado.simbolo}</b> Voice Desk
            </span>
          </div>
          <p className="dica">
            Dica do ícone: Voice Desk, {estado.rotulo.toLowerCase()}. O estado nunca depende só da
            cor: há símbolo e texto.
          </p>
        </div>
        <nav className="menu-bandeja" aria-label="Menu do Voice Desk">
          <div className="titulo-menu">Ditado</div>
          <div className="item">
            <span>{estadoId === "gravando" ? "Parar ditado" : "Iniciar ditado"}</span>
            <kbd>{atalho}</kbd>
          </div>
          <div className="titulo-menu">Modo padrão</div>
          <ul>
            {modosExemplo.map((m) => (
              <li key={m.id} className="item">
                <span>{m.rotulo}</span>
                <span aria-hidden="true">{m.padrao ? "✓" : ""}</span>
                {m.padrao && <span className="so-leitor">padrão</span>}
              </li>
            ))}
          </ul>
          <div className="sep" />
          <div className="item">Abrir histórico</div>
          <div className="item">Abrir configurações</div>
          <div className="item">Sair do Voice Desk</div>
        </nav>
      </div>
      <IniciarComOSistema />
    </section>
  );
}
