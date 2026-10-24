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
- Pré-requisitos: Node 24+, pnpm, Rust estável (`rustup`), CMake; no Mac, Xcode Command Line Tools.
- Instalar — `pnpm install`
- Rodar o app — `pnpm tauri dev` (regera `src/ipc/bindings.ts`)
- Interface no navegador, núcleo simulado — `pnpm dev:simulado` (porta 1420)
- Testes — `cargo test --manifest-path src-tauri/Cargo.toml` e `pnpm test`
- Lint e tipos — `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `pnpm biome ci`, `pnpm typecheck`
- Regerar `bindings.ts` sem abrir o app — `VOICE_DESK_REGERAR_BINDINGS=1 cargo test --manifest-path src-tauri/Cargo.toml bindings`
- Instaladores — `pnpm tauri build`
- Dados de desenvolvimento separados — `VOICE_DESK_PASTA_DADOS=/tmp/vd pnpm tauri dev`
- CI — `.github/workflows/ci.yml` roda tudo acima em `macos-14` e `windows-latest` em todo PR; `build-dev.yml` publica a pré-release `dev-<sha>` a cada merge na `main` (`docs/arquitetura/deploy.md`).
- e2e — `BASE_URL=<url> pnpm exec playwright test` (suíte em `tests/e2e/`, mantida pela QA; ainda não criada)

## Convenções diferentes do padrão
- Nomes de domínio em português sem acento (`modo`, `historico_item`); datas ISO 8601 UTC; durações em `_ms` inteiros; IDs UUID v7.
- Erros dos comandos: `{ codigo, mensagem, campo? }`, mensagem já em português.

## Armadilhas conhecidas
- Windows: o binário de teste da lib não leva o manifesto Common Controls v6; plugin com UI nativa (`tauri-plugin-dialog`/`rfd`) faz `cargo test` falhar com `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139). Por isso o `dialog` não está instalado no F-00: a história que o precisar (F-20/F-21) deve embutir o manifesto nos testes (`build.rs`, `cargo:rustc-link-arg-tests`) e validar no CI do Windows.
- (acrescente quando um agente errar algo de forma recorrente)
