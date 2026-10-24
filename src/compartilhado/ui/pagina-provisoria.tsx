type Props = { titulo: string; area: string };

/** Conteúdo provisório criado no F-00; a história da área troca só o conteúdo da própria página. */
export function PaginaProvisoria({ titulo, area }: Props) {
  return (
    <section>
      <h1>{titulo}</h1>
      <p className="aviso-provisorio">Tela ainda não implementada (área: {area}).</p>
    </section>
  );
}
