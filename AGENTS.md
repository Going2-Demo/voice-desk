# AGENTS.md — Voice Desk

App de desktop de ditado por voz para macOS 14+ e Windows 10/11, software livre. Feito em **Tauri 2**: núcleo em **Rust** (`src-tauri/`) e interface em **React + TypeScript + Vite** (`src/`). Sem back-end: dados em SQLite local; sincronização por pasta de nuvem do usuário, cifrada. Planejamento em `docs/produto/` (PRD), `docs/design/` (design e protótipo) e `docs/arquitetura/arquitetura.md` (stack, pastas, mapa de áreas, regras de conflito).

## Regras
- Código só por branch + PR com a chave do card no título (`ENG-42: ...`); merge squash feito só pelo DevSecOps (Igor).
- Commits em português: `tipo(escopo): descrição ENG-42`.
- Mexa só nas áreas da sua história (seção 10 da arquitetura). Pontos compartilhados seguem a tabela da seção 8.
- Código específico de Mac ou Windows só em `src-tauri/src/plataforma/`.
- Escrita em tabela sincronizável só por `configuracao::gravar()`.
- Nunca registrar em log texto ditado, contexto, áudio ou chave de API. Chaves só no cofre em arquivo cifrado do app (`chaves/cofre.rs`, ADR-10); nunca em texto puro.
- Nada de texto, prompt, ícone, som ou código do VoiceInk.
- `src/ipc/bindings.ts` é gerado: não edite; em conflito, regere.

## Onde ficam as coisas
- Telas: `src/modulos/<area>/` · janelas do gravador e do menu: `src/janelas/`
- Núcleo: `src-tauri/src/<area>/` · etapas do ditado: `src-tauri/src/ditado/etapas/`
- Migrações: `src-tauri/migracoes/AAAAMMDDHHMM_<area>_<descricao>.sql` (nunca edite uma já mesclada)
- Página do projeto: `site/` (HTML estático, sem build; `site/versoes.json` é a fonte da lista de versões, a primeira é a atual) · testes e2e: `tests/e2e/` · roteiro nativo: `tests/nativo/`
- Transcrição: `src-tauri/src/transcricao/` (contrato `Transcritor`, `groq.rs`, `flac.rs`, `config.rs` por modo, `falha.rs` grava o item "falhou"); a etapa `ditado/etapas/transcrever.rs` só orquestra. Provedor novo = arquivo novo e um braço em `fabrica_com_cofre`.
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
- Versão pública — suba a versão em `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` e no topo de `site/versoes.json` (e no HTML estático de `site/index.html`); só o BD autoriza a tag `vX.Y.Z`, que dispara `release.yml` (instaladores `voice-desk-mac.dmg` e `voice-desk-windows.exe`, nomes fixos usados pelos links da página). `pages.yml` publica `site/`.
- e2e — `BASE_URL=<url> pnpm exec playwright test` (suíte em `tests/e2e/`, mantida pela QA; ainda não criada)

## Convenções diferentes do padrão
- Nomes de domínio em português sem acento (`modo`, `historico_item`); datas ISO 8601 UTC; durações em `_ms` inteiros; IDs UUID v7.
- Erros dos comandos: `{ codigo, mensagem, campo? }`, mensagem já em português.

## Armadilhas conhecidas
- Windows: o binário de teste da lib precisa do manifesto Common Controls v6, senão `cargo test` falha com `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139). O `src-tauri/build.rs` o embute por argumento do linker (e o `tauri_build` fica sem o manifesto próprio); mantenha isso ao mexer no build. O plugin `dialog` não está instalado no F-00 por não ter uso ainda.
- App de bandeja: a janela `principal` nasce oculta e fechar a última janela não encerra o app (só `app.exit(0)`, pelo item Sair). Aberto no login, o sistema passa `--oculto` (plugin autostart) e nenhuma janela aparece. Para mudar o ícone use `bandeja::definir_estado`; o item de ditado do menu emite o evento `bandeja:alternar_ditado`.
- Mac: o Esc duplo (cancelar o ditado) escuta o teclado sem consumir a tecla (`plataforma/teclado`, handy-keys) e exige a permissão de Acessibilidade; sem ela o cancelamento fica indisponível e o app registra `esc_indisponivel` no log. O programa em foco vem do `active-win-pos-rs` (compila bindings do AppKit; precisa de Xcode CLT).
- O gravador é uma janela criada em `lib.rs` com `focusable(false)`; mexer nas flags dela pode devolver o foco ao Voice Desk (RF-01.2).
- Colagem (F-03): `ditado/colagem.rs` guarda a área de transferência, escreve o texto, simula Cmd/Ctrl+V (`plataforma/colagem`, enigo), espera 150 ms e devolve o conteúdo anterior (só texto e imagem). Sem Acessibilidade no Mac a simulação falha e o ditado termina `nao_colado` com o aviso no gravador. Não existe detecção de "nenhum campo de texto em foco": só se confere no roteiro nativo.
- Primeiro uso (F-06): `src-tauri/src/primeiro_uso/` guarda `concluido` e os passos pulados em `preferencia_local` (só nesta máquina); a janela principal vai a `/primeiro-uso` uma vez por execução enquanto não concluído (`ir-para-primeiro-uso.tsx`). Permissões: `plataforma/permissoes/` (Mac: `AXIsProcessTrusted` e `AVCaptureDevice`; Windows: chave `ConsentStore\microphone` do registro e entrada de áudio disponível; Acessibilidade não existe no Windows). O estado só se confere de verdade em máquina real (roteiro nativo). Mac, microfone: o `authorizationStatus` lido pelo processo pode ficar velho depois que o usuário liga o app em Ajustes; quando vem "negado", `macos.rs` confirma com `requestAccessForMediaType` (não abre diálogo se já decidido) — ENG-99.
- Vocabulário (F-07): `src-tauri/src/vocabulario/` (única escrita por `configuracao::gravar()`); `vocabulario::dica()` monta o `prompt` do provedor com as palavras mais recentes que cabem em 500 caracteres (limite de ~224 tokens do Groq) e `ditado/efeitos_tauri.rs` a põe no `Contexto`. A caixa rápida do menu é a janela `caixa-rapida` (`menu.html`, `src/janelas/menu/`), aberta pelo item do menu da bandeja.
- Substituições (F-08): `src-tauri/src/substituicoes/` (única escrita por `configuracao::gravar()`); `aplicar` casa palavra/expressão inteira sem diferenciar maiúsculas, numa passada só (original mais longo primeiro; o substituto não é reprocessado e é literal). A etapa `ditado/etapas/substituir.rs` usa as regras ativas que `efeitos_tauri.rs` põe em `Contexto.substituicoes` e transforma só `texto_final` (o transcrito fica intacto).
- Limpeza (F-09): `src-tauri/src/limpeza/` (`algoritmo.rs` puro; lista em `palavra_preenchimento` com sementes pela migração `202610280000`). Entrada da lista terminada em “…” só some quando o texto traz as reticências (hesitação). O bloco "Texto do ditado" (idioma, remover, formatar, Enter) grava na linha do modo ativo, criando o "Ditado" se não houver modo; a etapa `limpar` lê `Contexto.config_limpeza`. O modo navegador usa `src/modulos/limpeza/algoritmo.ts`, espelho do Rust: mude os dois juntos.
- Microfone (F-23): `src-tauri/src/audio/dispositivo.rs` guarda o nome do dispositivo em `preferencia_local` (`audio.microfone`, JSON; só nesta máquina, sem registro = padrão do sistema). `Captura::iniciar(preferido, ..)` abre o escolhido; se não estiver conectado cai no padrão e `Captura.aviso` vira parte da mensagem do gravador (`Ditado::mensagem_gravando`); sem dispositivo algum, erro `nenhum_microfone` e não grava. Só se confere com hardware de verdade no roteiro nativo.
- Pausar mídia (F-25): `plataforma/midia/` (`mod.rs` guarda o que o ditado pausou para só retomar isso; Mac: MediaRemote privado via `dlopen`, que o macOS recente pode recusar e então nada pausa; Windows: SMTC). Preferência local `midia.pausar_no_ditado` (padrão ligado). Os ganchos são `Efeitos::pausar_midia`/`retomar_midia` em `ditado/servico.rs` (retomar em parar, Esc e falha ao iniciar).
- (acrescente quando um agente errar algo de forma recorrente)
