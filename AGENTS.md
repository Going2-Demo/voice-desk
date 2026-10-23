# AGENTS.md — Voice Desk

Contexto curto e verificado para os agentes. Mantenha atualizado no mesmo PR que mudar o que está aqui.

## Produto
Engenharia reversa do Voice LNK, para Mac e Windows · Documentação em `docs/`.

## Stack
A definir pelo Arquiteto em `docs/arquitetura/arquitetura.md`.

## Comandos
| Ação | Comando |
|---|---|
| Instalar | a definir |
| Rodar local | a definir |
| Lint | a definir |
| Testes unitários | a definir |
| Build | a definir |
| Testes e2e | `BASE_URL=<url> npx playwright test` (suíte em `tests/e2e/`, mantida pela QA) |

## Convenções
- Código por branch + PR com a chave do card no título (`ENG-42: ...`); merge squash feito só pelo DevSecOps.
- Commits em português: `tipo(escopo): descrição ENG-42`.
- Segredos só em variáveis de ambiente; nomes em `.env.example`. Deploy em dev: `docs/arquitetura/deploy.md`.
