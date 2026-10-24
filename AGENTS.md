# AGENTS.md — Voice Desk

App de desktop de ditado por voz para macOS 14+ e Windows 10/11, software livre. Feito em **Tauri 2**: núcleo em **Rust** (`src-tauri/`) e interface em **React + TypeScript + Vite** (`src/`). Sem back-end: dados em SQLite local; sincronização por pasta de nuvem do usuário, cifrada. Planejamento em `docs/produto/` (PRD), `docs/design/` (design e protótipo) e `docs/arquitetura/arquitetura.md` (stack, pastas, mapa de áreas, regras de conflito).

## Regras
- Código só por branch + PR com a chave do card no título (`ENG-42: ...`); merge squash feito só pelo DevSecOps (Igor).
- Commits em português: `tipo(escopo): descrição ENG-42`.
- Mexa só nas áreas da sua história (seção 10 da arquitetura). Pontos compartilhados seguem a tabela da seção 8.
- Código específico de Mac ou Windows só em `src-tauri/src/plataforma/`.
- Escrita em tabela sincronizável só por `configuracao::gravar()`.
- Nunca registrar em log texto ditado, contexto, áudio ou chave de API. Chaves só no cofre do sistema.
- Nada de texto, prompt, ícone, som ou código do VoiceInk.
- `src/ipc/bindings.ts` é gerado: não edite; em conflito, regere.

## Onde ficam as coisas
- Telas: `src/modulos/<area>/` · janelas do gravador e do menu: `src/janelas/`
- Núcleo: `src-tauri/src/<area>/` · etapas do ditado: `src-tauri/src/ditado/etapas/`
- Migrações: `src-tauri/migracoes/AAAAMMDDHHMM_<area>_<descricao>.sql` (nunca edite uma já mesclada)
- Página do projeto: `site/` · testes e2e: `tests/e2e/` · roteiro nativo: `tests/nativo/`
- Deploy em dev: `docs/arquitetura/deploy.md` (Igor)

## Rodar e verificar
- TODO (após F-00): instalar — `pnpm install`
- TODO (após F-00): rodar o app — `pnpm tauri dev`
- TODO (após F-00): interface no navegador, núcleo simulado — `pnpm dev:simulado`
- TODO (após F-00): testes — `cargo test --manifest-path src-tauri/Cargo.toml` e `pnpm test`
- TODO (após F-00): lint e tipos — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `pnpm biome ci`, `pnpm typecheck`
- TODO (após F-00): instaladores — `pnpm tauri build`
- TODO (após F-00): e2e — `BASE_URL=<url> pnpm exec playwright test` (suíte em `tests/e2e/`, mantida pela QA)

## Convenções diferentes do padrão
- Nomes de domínio em português sem acento (`modo`, `historico_item`); datas ISO 8601 UTC; durações em `_ms` inteiros; IDs UUID v7.
- Erros dos comandos: `{ codigo, mensagem, campo? }`, mensagem já em português.

## Armadilhas conhecidas
- (vazio; acrescente quando um agente errar algo de forma recorrente)
