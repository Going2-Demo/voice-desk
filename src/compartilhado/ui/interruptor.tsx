import "../tema/componentes.css";

type Props = {
  id: string;
  ligado: boolean;
  desabilitado?: boolean;
  aoMudar: (ligado: boolean) => void;
};

/** Interruptor `.interruptor` do design (trilho 46×28); o nome vem do elemento `${id}-t`. */
export function Interruptor({ id, ligado, desabilitado, aoMudar }: Props) {
  return (
    <label className="interruptor">
      <input
        id={id}
        type="checkbox"
        role="switch"
        aria-checked={ligado}
        aria-labelledby={`${id}-t`}
        checked={ligado}
        disabled={desabilitado}
        onChange={(e) => aoMudar(e.target.checked)}
      />
      <span />
    </label>
  );
}
