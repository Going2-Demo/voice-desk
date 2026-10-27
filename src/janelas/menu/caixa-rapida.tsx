import { type FormEvent, useState } from "react";
import { api } from "../../modulos/vocabulario/api";

/** Caixa rápida do menu do ícone (RF-07.3): adiciona uma palavra sem abrir as configurações. */
export function CaixaRapida({ aoConcluir }: { aoConcluir?: () => void }) {
  const [palavra, setPalavra] = useState("");
  const [erro, setErro] = useState("");
  const [feito, setFeito] = useState("");
  const [enviando, setEnviando] = useState(false);

  const enviar = async (e: FormEvent) => {
    e.preventDefault();
    if (!palavra.trim()) {
      setErro("Digite uma palavra ou nome.");
      return;
    }
    setEnviando(true);
    try {
      const nova = await api.adicionar(palavra);
      setFeito(`“${nova.palavra}” adicionada ao vocabulário.`);
      setPalavra("");
      setErro("");
      aoConcluir?.();
    } catch (e) {
      setFeito("");
      setErro(e instanceof Error ? e.message : "Algo deu errado. Tente de novo.");
    } finally {
      setEnviando(false);
    }
  };

  return (
    <form className="caixa-rapida" onSubmit={enviar} noValidate>
      <label htmlFor="caixa-palavra">Adicionar ao vocabulário</label>
      <div className="linha">
        <input
          id="caixa-palavra"
          type="text"
          autoComplete="off"
          placeholder="Ex.: Hortolândia"
          // biome-ignore lint/a11y/noAutofocus: a janela existe só para digitar a palavra
          autoFocus
          value={palavra}
          aria-invalid={erro ? true : undefined}
          aria-describedby="caixa-aviso"
          onChange={(e) => {
            setPalavra(e.target.value);
            setErro("");
            setFeito("");
          }}
        />
        <button type="submit" disabled={enviando}>
          Adicionar
        </button>
      </div>
      <p id="caixa-aviso" role={erro ? "alert" : "status"} className={erro ? "erro" : "ok"}>
        {erro || feito}
      </p>
    </form>
  );
}
