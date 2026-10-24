import { useEffect, useId, useState } from "react";
import { api } from "../../bandeja/api";

const MENSAGEM_FALHA = 'Não foi possível mudar "iniciar com o sistema". Tente de novo.';

/** RF-05.2: liga ou desliga a abertura do Voice Desk junto com o sistema (padrão: ligado). */
export function Secao() {
  const id = useId();
  const [ligado, setLigado] = useState<boolean | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  useEffect(() => {
    let ativo = true;
    api.bandejaIniciarComOSistema().then((r) => {
      if (!ativo) return;
      if (r.status === "ok") setLigado(r.data);
      else setErro(r.error.mensagem);
    });
    return () => {
      ativo = false;
    };
  }, []);

  const alterar = async (novo: boolean) => {
    setErro(null);
    const r = await api.bandejaDefinirIniciarComOSistema(novo);
    if (r.status === "ok") setLigado(r.data);
    else setErro(r.error.mensagem || MENSAGEM_FALHA);
  };

  return (
    <section className="cartao" aria-labelledby={`${id}-titulo`}>
      <h2 id={`${id}-titulo`}>Sistema</h2>
      <label htmlFor={id}>
        <input
          id={id}
          type="checkbox"
          checked={ligado ?? false}
          disabled={ligado === null}
          onChange={(e) => alterar(e.target.checked)}
        />{" "}
        Iniciar com o sistema
      </label>
      <p id={`${id}-ajuda`}>
        O Voice Desk abre sozinho no ícone da barra de menus ou da bandeja quando você entra na sua
        conta, sem abrir janela.
      </p>
      {erro && <p role="alert">{erro}</p>}
    </section>
  );
}
