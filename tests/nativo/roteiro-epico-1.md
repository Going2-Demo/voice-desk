# Roteiro nativo — Épico 1

Pré-release: `dev-dca57c3` — https://github.com/Going2-Demo/voice-desk/releases/tag/dev-dca57c3
Instaladores: `voice-desk-dev-dca57c3-mac.dmg` (macOS) e `voice-desk-dev-dca57c3-win.exe` (Windows).

## Preparação

- Tenha uma chave real do Groq e um microfone funcionando.
- macOS: abra o app contornando o Gatekeeper (ENG-97). Conceda microfone e Acessibilidade quando pedido (itens 1–2).
- Para o item 1 (macOS e Windows), comece com o app recém-instalado, sem nenhuma permissão concedida.
- Em caso de falha, anexe print e o log do dia (`voice-desk.<data>.log`).

## Primeiro uso e permissões (macOS e Windows)

1. **Permissões (ENG-71, ENG-98)**: abra o app pela 1ª vez e vá em Permissões. Conceda microfone (e Acessibilidade no Mac). Esperado: estado de Pendente para Concedida; negada explica o que deixa de funcionar; "Abrir configurações do sistema" abre a tela certa; no Mac o diálogo do microfone aparece em português e o app consta em Privacidade e Segurança › Microfone.
2. **Indicadores atualizam sozinhos, só Mac (ENG-99, ENG-101)**: habilite o Voice Desk em Microfone e depois em Acessibilidade, voltando ao app sem reabrir. Esperado: cada linha fica verde/concedida em até ~3 s.
3. **Chave no cofre (ENG-69)**: salve a chave do Groq em Configurações › Provedores e chaves. Esperado: sem pedido de senha do sistema; `chaves.vdk` na pasta de dados sem a chave em texto puro; "válida" em até 10 s; reabra o app e a chave continua salva.
4. **Ditado de teste no primeiro uso (ENG-71)**: no último passo, clique na caixa, acione o atalho e fale. Esperado: texto aparece na caixa; com falha, aparece o motivo.
5. **Primeiro uso não reaparece (ENG-71)**: conclua, feche e reabra. Esperado: não abre de novo; continua em Configurações › Primeiro uso.

## Atalho e gravador (macOS e Windows)

6. **Gravador sem tirar o foco (ENG-66)**: com Bloco de Notas/TextEdit em foco, acione o atalho. Esperado: gravador em até 300 ms e o cursor continua no campo.
7. **Parar, bloquear e cancelar (ENG-66)**: fale e acione o atalho de novo (gravador vai a "processando" e some); acione o atalho durante o processamento (não abre nova gravação, indica "aguarde"); grave e aperte Esc duas vezes em 1 s (gravador some, nada colado) e depois Esc uma vez (chega ao programa em foco; Mac exige Acessibilidade).
8. **Trocar o atalho (ENG-66, ENG-96)**: em Atalhos, salve outro e use em outro programa. Esperado: funciona de imediato; só letra ou atalho do sistema é recusado; o Início mostra o atalho vigente e "Testar ditado" inicia a gravação.
9. **Limite de 10 min (ENG-66)**: grave sem parar. Esperado: aviso aos 9:30; aos 10:00 segue para o processamento.
10. **Microfone sem permissão (ENG-66)**: negue o microfone e acione o atalho. Esperado: "sem acesso ao microfone" com o caminho; nada gravado.

## Transcrição e colagem (macOS e Windows)

11. **Transcrição real (ENG-67, ENG-95)**: ditar ~20 s em português com editor em foco. Esperado: texto colado; histórico mostra provedor e modelo; log com `transcricao_concluida` e `colagem_resultado`.
12. **Inglês e silêncio (ENG-67, ENG-95)**: modo com idioma inglês, ditar uma frase (sai em inglês, sem tradução); acionar e parar sem falar (mostra "Não ouvi fala", nada colado).
13. **Sem chave e sem rede (ENG-67, ENG-69)**: apague a chave e dite (aviso "Cadastre a chave do Groq" com botão "Abrir chaves" que abre a tela). Recadastre, desligue a rede e dite (aviso "sem conexão com o provedor", nada colado, item "falhou" no histórico com áudio).
14. **Área de transferência e sem campo (ENG-68)**: copie "abc", dite e depois cole (cola "abc"; com imagem copiada antes, a imagem). Com foco sem campo de texto, dite (texto na área de transferência, aviso "Texto copiado; cole com Cmd+V/Ctrl+V").
15. **Enter depois de colar (ENG-68)**: em modo com Enter ligado, dite no chat (mensagem enviada); com desligado, fica na caixa.

## Só macOS

16. **Colar no Mail e no Slack (ENG-68)**: dite frase com acentos, emoji e quebra de linha. Esperado: texto na posição do cursor.
17. **Colagem automática por Cmd+V (ENG-100)**: com Acessibilidade concedida, dite no Teste do primeiro uso e num campo de outro app. Esperado: colado sem aviso "cole com Cmd+V"; se falhar, envie o log `colagem_falhou`.
18. **Acessibilidade negada (ENG-68)**: remova a permissão e dite. Esperado: aviso "Texto copiado; cole com Cmd+V"; colar manualmente funciona.

## Só Windows

19. **Colar no Outlook (ENG-68)**: corpo de e-mail novo, frase com acentos, emoji e quebra de linha. Esperado: texto na posição do cursor.
20. **Captura de áudio (ENG-95)**: dite 3 vezes seguidas e veja o log. Esperado: sem `erro_no_fluxo_de_audio` repetido.

## Ícone, menu e inicialização (macOS e Windows)

21. **Ícone e menu (ENG-70)**: abra o app instalado. Esperado: ícone na barra de menus/bandeja, sem janela principal; cada item do menu (ditado, modo padrão, histórico, configurações) executa; fechar a janela não encerra o app; o ícone muda ao gravar/processar/erro e volta ao normal.
22. **Sair e iniciar com o sistema (ENG-70)**: Menu › Sair, cancele (ícone fica); repita e confirme (ícone some e o atalho para). Com "iniciar com o sistema" ligado, reinicie a máquina: ícone em até 5 s, sem janela, atalho funciona; desligado, não abre sozinho.

## Como responder

Responda neste card, me mencionando, no formato: `1 ok · 2 falhou: <o que viu> · 3 não testei`.
