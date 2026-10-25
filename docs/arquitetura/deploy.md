# Deploy em dev — Voice Desk

> Dono: Igor - DevSecOps · Última atualização: 2026-10-23 · Card: ENG-56
> Status: **v1 independente de stack.** Os comandos de build (seção "Pipeline") serão fechados quando a arquitetura (ENG-55) definir a stack.

## Decisão

O Voice Desk é um app de desktop (Mac e Windows) sem servidor próprio (PRD seções 10 e 11: sem servidor central, sem serviço pago). Não há aplicação para hospedar, então **o "deploy em dev" é gerar os instaladores de cada commit da `main` e deixá-los para a Marta baixar e testar nas máquinas reais**. Não usa Dokploy.

Destino: **GitHub Actions** (build em runner `macos-14` e `windows-latest`) + **GitHub Releases** (versões) + **GitHub Pages** (página do projeto, F-27).
Dev = pré-release automática `dev-<sha curto>` com os dois instaladores, marcada como *pre-release*. Versão pública = tag `vX.Y.Z` (fora do escopo do dev; só com aprovação do BD).

| Opção | Prós | Contras | Veredito |
|---|---|---|---|
| GitHub Actions + Releases + Pages | Mesmo lugar do código; runners Mac e Windows gratuitos em repo público; download sem cadastro (RF-27.1); sem custo; operável por `gh` | Repo privado consome minutos pagos (Mac conta 10x); download de release em repo privado exige login | **Escolhida** |
| Dokploy dev | Já operado por MCP | Só roda contêiner Linux; não gera app de Mac/Windows | Descartada (só serviria para hospedar a página) |
| Servidor próprio de arquivos / S3 | Controle total | Custo, manutenção, nada a mais que Releases | Descartada |

## Componentes
| Componente | Tipo | Nome | Build |
|---|---|---|---|
| Instalador Mac | artefato (.dmg) | `voice-desk-<versão>-mac.dmg` | runner `macos-14` |
| Instalador Windows | artefato (.exe/.msi, sem admin: RNF-12) | `voice-desk-<versão>-win.exe` | runner `windows-latest` |
| Página do projeto | site estático | GitHub Pages do repositório | `docs/site/` (a definir com o Otávio) |

## Como publicar (dev)
1. Pré-requisito: PR mergeado na `main`; nenhum outro workflow `build-dev` em execução (`gh run list --workflow build-dev --limit 3`).
2. O merge dispara `.github/workflows/build-dev.yml` (`on: push` na `main`): testes, build Mac e Windows em paralelo, publica a pré-release `dev-<sha>`. Se não disparar: `gh workflow run build-dev --ref main`.
3. Acompanhar na mesma execução: `sleep 90` e `gh run list --workflow build-dev --limit 1`, até `completed/success`; log só em falha (`gh run view <id> --log-failed | tail -30`).
4. Migrações de banco: não há banco de servidor; migração do banco local do app roda na abertura (responsabilidade da arquitetura).

Deploy automático por push: **ligado** (é só build; nada vai para usuário final). Publicação pública (`vX.Y.Z`) é manual e depende do BD.

## Pipeline (a fechar com a stack da ENG-55)
- Workflow `build-dev.yml`: jobs `test`, `build-mac`, `build-win`, `release-dev`; versão = `git describe` + sha; o app lê o commit da própria versão (tela "Sobre", RF-27.4).
- Workflow `release.yml` (tag `v*`): mesmos builds, release pública com changelog (RF-27.3).
- Workflow `pages.yml`: publica a página do projeto.
- Quem escreve os workflows: Igor, na primeira história de infraestrutura do E-01, via PR revisado.

## Variáveis e segredos
| Nome | Para quê | Onde está o valor |
|---|---|---|
| `GITHUB_TOKEN` | publicar release | automático do Actions |
| `APPLE_CERT_P12`, `APPLE_CERT_PASSWORD`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | assinar e notarizar no Mac | GitHub › Settings › Secrets (**não usados na v1**: o BD decidiu não ter custo de assinatura, pergunta 6) |
| `WIN_CERT_PFX`, `WIN_CERT_PASSWORD` | assinar no Windows | idem |
Sem certificado pago, o Mac (Gatekeeper) e o Windows (SmartScreen) mostram aviso na primeira abertura; a página do projeto traz as instruções para liberar. No macOS 15 o "botão direito › Abrir" não libera mais: o caminho é OK › Ajustes do Sistema › Privacidade e Segurança › Abrir Assim Mesmo, ou `xattr -dr com.apple.quarantine "/Applications/Voice Desk.app"` no Terminal (ENG-97). Decisão do BD em 2026-10-23: sem custo. Valores nunca vão para o repositório.

## URL de dev
- Pré-releases: `https://github.com/going2-demo/voice-desk/releases` (tag `dev-<sha>`)
- Página do projeto: `https://going2-demo.github.io/voice-desk/` (prevista)
- Usuário de teste: não existe conta; a Marta usa as próprias chaves de teste, guardadas no cofre do sistema.

## Saúde
- Não há `/health`. Verificação de um deploy dev: workflow `success`; os dois instaladores presentes na release (`gh release view dev-<sha> --json assets`); versão exibida em "Sobre" = sha do merge; em máquina real o app abre o primeiro uso (F-06).

## Rollback
1. Apagar a pré-release com defeito (`gh release delete dev-<sha> --cleanup-tag`); a anterior continua disponível.
2. Código: `git revert` do squash por PR `revert/<KEY>` e novo build.

## Primeiro deploy (provisionamento)
- [ ] Repositório público (necessário para distribuição aberta, RF-27.1/RNF-14, e para minutos de runner gratuitos) — **BD decide mais tarde**; hoje é privado. Enquanto privado, os builds funcionam, mas consomem minutos do plano e o download exige login.
- [ ] Habilitar GitHub Pages; escrever os 3 workflows (após a stack).
- [x] Assinatura de código: decidido, sem custo (sem assinatura paga).
Status: pendente
