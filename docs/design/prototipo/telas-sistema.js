/* Telas: Ditado e gravador (F-01 a F-03, F-26), Sincronização (F-20, F-21), Configurações (F-04, F-17, F-22 a F-25), Ícone e menu (F-05, F-07.3), Primeiro uso (F-06), Sobre (F-27.4). */
(function () {
  var VD = window.VD, S = VD.S, D = VD.D, esc = VD.esc;
  var $ = function (s, r) { return (r || document).querySelector(s); };

  /* ================= GRAVADOR ================= */
  VD.gravador = function (fase, modo, texto, tempo) {
    var barras = ""; for (var i = 0; i < 12; i++) barras += "<i></i>";
    return '<div class="gravador" data-fase="' + fase + '" role="status"><span class="ponto" aria-hidden="true"></span><span class="modo">' + esc(modo ? modo.emoji + " " + modo.nome : "") + '</span><span class="nivel" aria-hidden="true">' + barras + '</span><span class="estado">' + esc(texto) + "</span>" + (tempo ? '<span class="tempo" id="tempo">' + tempo + "</span>" : "") + "</div>";
  };
  var sim = { programa: "", via: "global", fala: "", cenario: "ok" };
  var texto = {
    padrao: { fala: "lembrar de renovar o certificado do domínio até o dia 30", final: "Lembrar de renovar o certificado do domínio até o dia 30." },
    email: { fala: "oi Ana então assim queria confirmar a reunião de quinta às 14h e saber se você manda a proposta antes", final: "Olá, Ana,\n\nGostaria de confirmar a reunião de quinta-feira, às 14h. Você consegue enviar a proposta antes?\n\nObrigado,\nBD", limpo: "Oi Ana queria confirmar a reunião de quinta às 14h e saber se você manda a proposta antes." },
    chat: { fala: "pode mandar o contrato da Tâmara Quintanilha ainda hoje", final: "pode mandar o contrato da Tâmara Quintanilha ainda hoje" },
    notas: { fala: "reunião de alinhamento primeiro prazo segundo custo terceiro treinamento", final: "- Prazo\n- Custo\n- Treinamento", limpo: "Reunião de alinhamento primeiro prazo segundo custo terceiro treinamento." },
    ingles: { fala: "preciso de mais uma semana para entregar o relatório", final: "I need one more week to deliver the report.", limpo: "Preciso de mais uma semana para entregar o relatório." }
  };
  var tx = function (m) { return texto[m.id] || texto.padrao; };
  function resolveModo() {
    var primeiras = sim.fala.toLowerCase().split(/\s+/).slice(0, 3).join(" ");
    if (primeiras) { var g = S.modos.filter(function (m) { return m.gatilhos.some(function (x) { return primeiras.indexOf(x) >= 0; }); })[0]; if (g) return { m: g, motivo: "palavra-gatilho “" + g.gatilhos.filter(function (x) { return primeiras.indexOf(x) >= 0; })[0] + "”" }; }
    if (sim.via !== "global") { var a = VD.modoPorId(sim.via); if (a) return { m: a, motivo: "atalho do modo " + a.nome }; }
    var p = S.modos.filter(function (m) { return m.programas[S.plat].indexOf(sim.programa) >= 0; })[0];
    if (p) return { m: p, motivo: "programa em foco: " + sim.programa };
    var pad = S.modos.filter(function (m) { return m.padrao; })[0] || S.modos[0];
    return { m: pad, motivo: "modo padrão (nenhuma regra se aplicou)" };
  }
  var D2 = S.ditado, timer = null, escUltimo = 0;
  function pinta() {
    var el = $("#sim-dinamico"); if (!el) return;
    var r = resolveModo(), m = D2.modo || r.m, h = "";
    var fase = D2.fase;
    var rotulos = { gravando: "Gravando… acione o atalho de novo para parar", processando: "Processando…", ok: "Pronto", aviso: D2.msg, erro: D2.msg, aguarde: "Aguarde: ainda processando" };
    if (fase !== "idle") h = VD.gravador(fase === "aguarde" ? "processando" : fase, m, rotulos[fase] || "", fase === "gravando" ? "0:" + ("0" + (D2.seg || 0)).slice(-2) : "");
    var pipe = "";
    if (D2.pipe && D2.pipe.length) pipe = '<ul class="passos-pipe" aria-label="Etapas do ditado">' + D2.pipe.map(function (p) { return '<li class="' + p.s + '">' + esc(p.t) + "</li>"; }).join("") + "</ul>";
    el.innerHTML = '<div class="programa-falso"><header><span>' + esc(sim.programa || "Programa em foco") + '</span><span class="suave">campo de texto</span></header><div class="campo-falso" aria-live="polite">' + esc(D2.campo || "") + (fase === "idle" || fase === "gravando" ? '<span class="cursor" aria-hidden="true"></span>' : "") + '</div></div><div class="gravador-pos">' + h + "</div>" + pipe +
      '<p class="dica" style="margin-top:12px">Modo aplicado ao próximo ditado: <b>' + esc(r.m.emoji + " " + r.m.nome) + "</b> — " + esc(r.motivo) + ".</p>";
    var b = $("#btn-atalho"); if (b) b.textContent = fase === "gravando" ? "Parar (acionar o atalho)" : "Acionar o atalho global";
  }
  function fim(fase, msg, ms) { D2.fase = fase; D2.msg = msg; pinta(); setTimeout(function () { if (D2.fase === fase) { D2.fase = "idle"; D2.pipe = []; D2.modo = null; pinta(); } }, ms || 2600); }
  function addHist(estado, m, extra) {
    var t = tx(m);
    S.hist.unshift(Object.assign({ id: Date.now(), data: "23/10/2026", hora: "10:" + ("0" + (10 + S.hist.length % 40)).slice(-2), modo: m.emoji + " " + m.nome, programa: sim.programa || "Bloco de Notas", estado: estado, transcrito: t.fala, final: estado === "colado" ? t.final : "", transcProv: m.transc === "local" ? "Whisper local · large-v3-turbo" : "Groq · " + m.modeloTransc, iaProv: m.refina ? "Groq · " + m.modeloIa : "—", dur: "0:0" + (D2.seg || 5), tTransc: "0,9 s", tIa: m.refina ? "1,5 s" : "—", audio: true }, extra || {}));
  }
  function comecar() {
    if (D2.fase === "processando") { D2.fase = "aguarde"; pinta(); setTimeout(function () { if (D2.fase === "aguarde") { D2.fase = "processando"; pinta(); } }, 1400); return; }
    if (D2.fase === "gravando") return parar();
    if (D2.fase !== "idle") return;
    var m = resolveModo().m; D2.modo = m; D2.campo = D2.campo || ""; D2.pipe = [];
    if (!S.perm.mic) { fim("erro", "Sem permissão do microfone. Abra Configurações › Permissões.", 4200); VD.toast("O Voice Desk não pode gravar sem acesso ao microfone.", "erro"); return; }
    if (m.transc === "local" && S.modelo.estado !== "baixado") { fim("erro", "Modelo local não baixado. Baixe em Configurações antes de gravar.", 4800); return; }
    if (!S.cfg.micConectado && S.cfg.mic !== "Padrão do sistema") VD.toast("“" + S.cfg.mic + "” não está conectado. Usando o microfone padrão do sistema.", "alerta");
    D2.seg = 0; D2.fase = "gravando"; pinta();
    timer = setInterval(function () { D2.seg++; var t = $("#tempo"); if (t) t.textContent = "0:" + ("0" + D2.seg).slice(-2); }, 1000);
  }
  function parar() {
    clearInterval(timer); var m = D2.modo, t = tx(m);
    if (sim.cenario === "sem_fala" && S.cfg.descartarSemFala) { fim("aviso", "Nenhuma fala detectada. Nada foi enviado nem colado.", 3200); return; }
    D2.fase = "processando"; D2.pipe = [{ t: "Transcrever", s: "atual" }, { t: "Limpar texto", s: "" }, { t: "Refinar com IA", s: m.refina ? "" : "pulado" }, { t: "Colar", s: "" }]; pinta();
    var etapa = function (i, aoFim) { setTimeout(function () { if (D2.fase !== "processando" && D2.fase !== "aguarde") return; D2.pipe[i].s = "feito"; if (D2.pipe[i + 1]) D2.pipe[i + 1].s = D2.pipe[i + 1].s === "pulado" ? "pulado" : "atual"; pinta(); aoFim(); }, 750); };
    etapa(0, function () {
      if (sim.cenario === "sem_internet" && m.transc === "groq") { D2.pipe[0].s = "pulado"; addHist("falhou", m, { transcrito: "", aviso: "Sem conexão com a internet. O áudio foi guardado; você pode transcrever de novo." }); fim("erro", "Falhou: sem conexão. O áudio ficou no histórico.", 4800); VD.toast("Transcrição falhou. Áudio guardado no Histórico para transcrever de novo.", "erro"); return; }
      etapa(1, function () {
        var seguir = function () {
          etapa(3, function () {
            var final = tx(m).final, aviso = null;
            if (m.refina && sim.cenario === "ia_falha") { final = tx(m).limpo || tx(m).final; aviso = "Colado sem refinamento: o provedor de IA não respondeu em 15 s."; }
            if (sim.cenario === "sem_colagem" || !S.perm.acess) {
              addHist("nao-colado", m, { aviso: "O programa em foco não aceitou a colagem. O texto ficou na área de transferência." });
              fim("aviso", "Texto copiado — cole com " + VD.mod() + "+V.", 4800); VD.toast("Texto copiado. Clique no campo certo e cole com " + VD.mod() + "+V.", "alerta"); return;
            }
            D2.campo = (D2.campo ? D2.campo + "\n" : "") + final + (m.enter ? "\n↵ (Enter enviado)" : "");
            addHist("colado", m, aviso ? { aviso: aviso, final: final } : {});
            if (aviso) { fim("aviso", aviso.replace("Colado sem refinamento: ", "Colado sem refinamento: "), 4200); VD.toast(aviso, "alerta"); } else fim("ok", "Texto colado", 2400);
          });
        };
        if (m.refina) { if (sim.cenario === "ia_falha") { D2.pipe[2].s = "pulado"; pinta(); } etapa(2, seguir); } else { D2.pipe[3].s = "atual"; pinta(); seguir(); }
      });
    });
  }
  VD.acoes["atalho"] = comecar;
  VD.acoes["sim-muda"] = function (t) { sim[t.getAttribute("data-k")] = t.value; pinta(); };
  VD.acoes["sim-limpa"] = function () { D2.campo = ""; pinta(); };
  document.addEventListener("keydown", function (e) {
    if (e.key !== "Escape" || D2.fase !== "gravando" || document.querySelector("#modal-raiz .modal")) return;
    var agora = Date.now();
    if (agora - escUltimo < 1500) { clearInterval(timer); fim("aviso", "Cancelado. Nada foi colado nem enviado.", 2200); VD.toast("Ditado cancelado: nada foi colado, enviado ao provedor nem guardado no histórico."); escUltimo = 0; }
    else { escUltimo = agora; VD.toast("Esc uma vez passa para o programa em foco. Aperte Esc de novo para cancelar o ditado.", "alerta"); }
  });
  VD.acoes["esc2"] = function () { if (D2.fase !== "gravando") return; clearInterval(timer); fim("aviso", "Cancelado. Nada foi colado nem enviado.", 2200); VD.toast("Ditado cancelado: nada foi colado, enviado ao provedor nem guardado no histórico."); };

  VD.rota("ditado", function () {
    var progs = D.programas[S.plat]; if (!sim.programa || progs.indexOf(sim.programa) < 0) sim.programa = S.plat === "mac" ? "Mail" : "Outlook";
    var opt = function (v, a, t) { return '<option value="' + esc(v) + '"' + (a === v ? " selected" : "") + ">" + esc(t || v) + "</option>"; };
    var h = VD.cabecalho("Testar ditado", "Simula o ditado de ponta a ponta: o atalho abre o gravador, você “fala”, o texto é limpo e colado no programa em foco. Em produção o app fica na " + (S.plat === "mac" ? "barra de menus" : "bandeja") + " e o gravador aparece sobre qualquer programa.",
      '<span class="selo info">Atalho global <kbd>' + esc(VD.atalhoGlobal()) + "</kbd></span>");
    h += '<div class="cartao"><h2>Cenário</h2><div class="duas-col" style="margin-top:12px"><div class="campo"><label for="s-prog">Programa em foco</label><select id="s-prog" data-k="programa" data-muda="sim-muda">' + progs.map(function (p) { return opt(p, sim.programa); }).join("") + opt("Outro programa", sim.programa) + '</select></div>' +
      '<div class="campo"><label for="s-via">Como o ditado começa</label><select id="s-via" data-k="via" data-muda="sim-muda">' + opt("global", sim.via, "Atalho global") + S.modos.filter(function (m) { return m.atalho[S.plat]; }).map(function (m) { return opt(m.id, sim.via, "Atalho do modo " + m.nome + " (" + m.atalho[S.plat] + ")"); }).join("") + '</select></div>' +
      '<div class="campo"><label for="s-fala">Primeiras palavras faladas (opcional)</label><input id="s-fala" type="text" data-k="fala" data-entra="sim-muda" value="' + esc(sim.fala) + '" placeholder="Ex.: e-mail, queria confirmar…"><span class="dica">Se contiver uma palavra-gatilho, ela vence o programa.</span></div>' +
      '<div class="campo"><label for="s-cen">O que acontece</label><select id="s-cen" data-k="cenario" data-muda="sim-muda">' + opt("ok", sim.cenario, "Tudo certo") + opt("sem_internet", sim.cenario, "Sem internet na transcrição") + opt("sem_colagem", sim.cenario, "O programa recusa a colagem") + opt("sem_fala", sim.cenario, "Gravação sem fala") + "</select></div></div></div>";
    h += '<div class="mesa" style="margin-top:16px"><div id="sim-dinamico"></div><div class="acoes" style="margin-top:16px"><button id="btn-atalho" type="button" class="btn primario" data-acao="atalho">Acionar o atalho global</button><button type="button" class="btn" data-acao="esc2">Esc duas vezes (cancelar)</button><button type="button" class="btn discreto" data-acao="sim-limpa">Limpar o campo</button></div><p class="dica" style="margin-top:8px">No teclado: durante a gravação, aperte <kbd>Esc</kbd> duas vezes em até 1,5 s para cancelar.</p></div>';
    h += '<div class="cartao" style="margin-top:16px"><h2>Todos os estados do gravador</h2><p class="dica">Referência visual: nome e emoji do modo sempre à vista; mensagens dizem o que aconteceu e o que fazer.</p><div style="display:flex;flex-direction:column;gap:12px;align-items:flex-start;margin-top:12px">' +
      [["gravando", "Gravando… acione o atalho de novo para parar", "0:07"], ["gravando", "Parando automaticamente em 30 s", "9:30"], ["processando", "Processando…"], ["processando", "Aguarde: ainda processando"], ["ok", "Texto colado"], ["aviso", "Texto copiado — cole com " + VD.mod() + "+V"], ["aviso", "Nenhuma fala detectada"], ["aviso", "Usando o microfone padrão: “Headset USB” desconectado"], ["erro", "Falhou: sem conexão. O áudio ficou guardado nesta máquina"]]
        .map(function (g, i) { return VD.gravador(g[0], S.modos[0], g[1], g[2]); }).join("") + "</div></div>";
    return h;
  });
  VD.aposRender = function (r) { if (r === "ditado") pinta(); };

  /* ================= SINCRONIZAÇÃO ================= */
  VD.rota("sincronizacao", function () {
    var off = !S.sync.ligada || S.estado === "vazio";
    var h = VD.cabecalho("Sincronização", "Configurou uma vez, vale no Mac e no Windows. Só a configuração viaja: modos, prompts, vocabulário, substituições, palavras de preenchimento e preferências de texto.");
    if (S.estado === "carregando") return h + VD.carregando(3);
    if (off) {
      h += VD.vazio("🔄", "Sincronização desligada nesta máquina", "Ligue para manter modos e vocabulário iguais na sua outra máquina. Chaves, histórico e áudio nunca saem daqui.", '<button class="btn primario" data-acao="sync-liga">Ligar a sincronização</button>');
    } else {
      var est = S.estado === "erro" ? "Erro" : S.sync.estado;
      var tipo = est === "Em dia" ? "ok" : est === "Sem conexão" ? "alerta" : est === "Erro" ? "erro" : "info";
      if (est === "Erro") h += '<div class="aviso erro" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo"><b>Não foi possível sincronizar.</b><br>O serviço de sincronização não respondeu. Suas alterações continuam salvas aqui e serão enviadas na próxima tentativa. Última sincronização: hoje, 09:40.<div class="acoes"><button class="btn pequeno" data-acao="sync-agora">Tentar agora</button></div></div></div>';
      if (est === "Sem conexão") h += '<div class="aviso alerta" role="status"><span aria-hidden="true">📡</span><div class="conteudo"><b>Sem internet.</b> 2 alterações aguardam envio e seguem sozinhas quando a conexão voltar.</div></div>';
      h += '<div class="cartao"><div class="acoes" style="justify-content:space-between"><div><h2>Estado</h2><p style="margin-top:6px"><span class="selo ' + tipo + '">' + (est === "Em dia" ? "✓ " : "") + est + '</span> <span class="suave pequeno">Última sincronização: hoje, 09:40</span></p></div><div class="acoes"><button class="btn" data-acao="sync-agora">Sincronizar agora</button><button class="btn" data-acao="sync-offline">' + (S.sync.estado === "Sem conexão" ? "Simular: voltou a internet" : "Simular: sem internet") + '</button></div></div></div>' +
        '<div class="cartao"><h2>Suas máquinas</h2><p class="dica">Só as máquinas do mesmo usuário. Em conflito, vale a alteração mais recente do mesmo item.</p><div class="tabela-wrap"><table class="tabela-resp"><thead><tr><th>Máquina</th><th>Última sincronização</th><th>Estado</th></tr></thead><tbody>' + D.maquinas.map(function (m) { return "<tr><td>" + (m.plat === "mac" ? "🍎 " : "🪟 ") + esc(m.nome) + (m.plat === S.plat ? ' <span class="selo info">Esta máquina</span>' : "") + "</td><td>" + m.ultima + '</td><td><span class="selo ok">' + m.estado + "</span></td></tr>"; }).join("") + '</tbody></table></div><div class="acoes" style="margin-top:12px"><button class="btn" data-acao="sync-conectar">Conectar outra máquina</button><button class="btn perigo" data-acao="sync-desliga">Desligar nesta máquina</button></div></div>';
    }
    h += '<div class="cartao"><h2>O que viaja e o que fica</h2><div class="duas-col" style="margin-top:8px"><div><b>Sincroniza</b><ul><li>Modos e prompts</li><li>Vocabulário e substituições</li><li>Palavras de preenchimento</li><li>Preferências de texto</li></ul></div><div><b>Nunca sai desta máquina</b><ul><li>Chaves de API (ficam no cofre do sistema)</li><li>Histórico e áudio</li><li>Atalhos, microfone e modelos baixados</li></ul></div></div></div>';
    h += '<div class="cartao"><h2>Exportar e importar</h2><p class="dica">Para compartilhar seus modos com alguém ou guardar uma cópia. O arquivo não leva chaves, histórico nem áudio.</p><div class="acoes"><button class="btn" data-acao="cfg-exporta">Exportar configuração</button><button class="btn" data-acao="cfg-importa">Importar configuração…</button></div></div>';
    return h;
  });
  VD.acoes["sync-liga"] = function () {
    VD.modal("Ligar a sincronização", '<p>Você vai conectar esta máquina à sua configuração. Em seguida, na outra máquina, escolha “Conectar outra máquina” e informe o código.</p><div class="campo" style="margin-top:12px"><label for="sc">Código desta configuração</label><input id="sc" type="text" readonly value="VD-4F7K-92QX" class="mono"></div>', [{ rot: "Cancelar" }, { rot: "Ligar", cls: "primario", acao: function () { S.sync.ligada = true; S.sync.estado = "Em dia"; if (S.estado === "vazio") S.estado = "normal"; VD.toast("Sincronização ligada. A configuração aparece na outra máquina em até 5 minutos."); VD.render(); } }]);
  };
  VD.acoes["sync-conectar"] = function () { VD.modal("Conectar outra máquina", '<p>Na outra máquina, abra Sincronização › Conectar a uma configuração existente e digite o código:</p><p style="margin:12px 0;font-size:24px" class="mono"><b>VD-4F7K-92QX</b></p><p class="dica">O código só liga máquinas do seu usuário. Ele vale por 10 minutos.</p>', [{ rot: "Fechar" }]); };
  VD.acoes["sync-agora"] = function () { S.sync.estado = "Sincronizando"; if (S.estado === "erro") S.estado = "normal"; VD.render(); setTimeout(function () { S.sync.estado = "Em dia"; if (location.hash.indexOf("sincronizacao") >= 0) VD.render(); VD.toast("Configuração em dia."); }, 1200); };
  VD.acoes["sync-offline"] = function () { S.sync.estado = S.sync.estado === "Sem conexão" ? "Em dia" : "Sem conexão"; VD.render(); };
  VD.acoes["sync-desliga"] = function () { VD.confirma("Desligar a sincronização nesta máquina?", "A configuração local fica como está e deixa de receber mudanças da outra máquina. Você pode ligar de novo quando quiser.", "Desligar", function () { S.sync.ligada = false; VD.render(); }); };
  VD.acoes["cfg-exporta"] = function () { VD.toast(D.exportacao.arquivo + " salvo em Downloads. Sem chaves, histórico nem áudio."); };
  VD.acoes["cfg-importa"] = function () {
    VD.modal("Importar configuração", '<p>Escolha o arquivo exportado por você ou por quem compartilhou os modos com você.</p><div class="acoes" style="margin-top:12px"><button class="btn" data-acao="imp-ok">Escolher “' + esc(D.importacao.arquivo) + '”</button><button class="btn" data-acao="imp-ruim">Escolher “planilha-de-clientes.xlsx”</button></div>', [{ rot: "Cancelar" }]);
  };
  VD.acoes["imp-ruim"] = function () {
    VD.fechaModal(); VD.modal("Arquivo recusado", '<div class="aviso erro" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo">Este arquivo não é uma configuração do Voice Desk. <b>Nada foi alterado.</b> Exporte a configuração pelo próprio Voice Desk e tente de novo.</div></div>', [{ rot: "Entendi" }]);
  };
  VD.acoes["imp-ok"] = function () {
    var a = D.importacao.atual, n = D.importacao.resumo;
    VD.fechaModal();
    VD.modal("Substituir a configuração desta máquina?", '<p>Arquivo: <b>' + esc(D.importacao.arquivo) + '</b></p><table class="diff"><thead><tr><th>Item</th><th>Agora</th><th>Depois</th></tr></thead><tbody><tr><td>Modos</td><td>' + a.modos + "</td><td>" + n.modos + "</td></tr><tr><td>Vocabulário</td><td>" + a.vocabulario + "</td><td>" + n.vocabulario + "</td></tr><tr><td>Substituições</td><td>" + a.substituicoes + "</td><td>" + n.substituicoes + "</td></tr><tr><td>Palavras de preenchimento</td><td>" + a.preenchimento + "</td><td>" + n.preenchimento + '</td></tr></tbody></table><p class="dica" style="margin-top:8px">Suas chaves, histórico, atalhos e microfone não mudam. O que existe hoje será substituído.</p>', [{ rot: "Cancelar" }, { rot: "Substituir configuração", cls: "primario", acao: function () { VD.toast("Configuração importada: 5 modos, 12 palavras, 5 substituições."); } }]);
  };

  /* ================= CONFIGURAÇÕES ================= */
  var ABAS = [["geral", "Geral"], ["atalhos", "Atalhos"], ["provedores", "Chave do Groq"], ["permissoes", "Permissões"]];
  function abas(at) { return '<nav class="abas" aria-label="Configurações">' + ABAS.map(function (a) { return '<a href="#/config/' + a[0] + '"' + (a[0] === at ? ' aria-current="page"' : "") + ">" + a[1] + "</a>"; }).join("") + "</nav>"; }
  var cab = function () { return VD.cabecalho("Configurações"); };
  VD.rota("config/geral", function () {
    var c = S.cfg;
    return cab() + abas("geral") + VD.estadoLista(function () {
      return '<div class="cartao"><h2>Sistema</h2>' + VD.opcao("c-inicia", "Iniciar com o sistema", "O Voice Desk abre sozinho na " + (S.plat === "mac" ? "barra de menus" : "bandeja") + " quando você liga a máquina.", c.iniciarComSistema, 'data-muda="cfg-liga" data-k="iniciarComSistema"') + "</div>" +
        '<div class="cartao"><h2>Microfone</h2><div class="campo"><label for="c-mic">Microfone de entrada</label><select id="c-mic" data-muda="cfg-mic">' + D.microfones.map(function (m) { return "<option" + (m === c.mic ? " selected" : "") + ">" + esc(m) + "</option>"; }).join("") + '</select></div>' +
        (c.mic !== "Padrão do sistema" ? '<p class="' + (c.micConectado ? "" : "aviso alerta") + '" style="padding:' + (c.micConectado ? 0 : "12px 16px") + '" role="status">' + (c.micConectado ? '<span class="selo ok">Conectado</span>' : "Este microfone não está conectado. Os ditados usam o padrão do sistema e o gravador avisa qual está em uso. Quando ele voltar, será usado de novo.") + "</p>" : "") +
        '<button type="button" class="btn pequeno" data-acao="cfg-mic-sim">Simular: ' + (c.micConectado ? "desconectar" : "reconectar") + " o microfone</button></div>" +
        '<div class="cartao"><h2>Texto do ditado</h2><p class="dica">Vale para todos os ditados.</p><div class="campo" style="max-width:320px"><label for="c-idioma">Idioma do ditado</label><select id="c-idioma" data-muda="cfg-idioma">' + [["pt-BR", "Português do Brasil"], ["en", "Inglês"], ["auto", "Detectar automaticamente"]].map(function (o) { return '<option value="' + o[0] + '"' + (o[0] === c.idioma ? " selected" : "") + ">" + o[1] + "</option>"; }).join("") + "</select></div>" +
        VD.opcao("c-limpeza", "Remover palavras de preenchimento", "Tira “é…”, “tipo”, “né” e parecidas. A lista é editável em Limpeza do texto.", c.limpeza, 'data-muda="cfg-liga" data-k="limpeza"') +
        VD.opcao("c-format", "Formatar o texto", "Maiúscula no início, pontuação e espaços arrumados.", c.formatacao, 'data-muda="cfg-liga" data-k="formatacao"') +
        VD.opcao("c-enter", "Apertar Enter depois de colar", "Desligado por padrão. Ligado, vale para todo programa, inclusive e-mail e documentos: ligue só se quiser enviar a mensagem na hora.", c.enter, 'data-muda="cfg-liga" data-k="enter"') + "</div>" +
        '<div class="cartao"><h2>Durante o ditado</h2>' + VD.opcao("c-sons", "Sons de início e fim", "Um aviso curto antes de gravar e ao terminar. O som de início não entra na gravação.", c.sons, 'data-muda="cfg-liga" data-k="sons"') +
        '<div class="campo" style="margin-top:8px"><label for="c-som">Som</label><div class="acoes"><select id="c-som" data-muda="cfg-som" style="max-width:220px"' + (c.sons ? "" : " disabled") + ">" + D.sons.map(function (s) { return "<option" + (s === c.som ? " selected" : "") + ">" + s + "</option>"; }).join("") + '</select><button type="button" class="btn" data-acao="cfg-previa"' + (c.sons ? "" : ' aria-disabled="true"') + ">▶ Ouvir prévia</button></div></div>" +
        VD.opcao("c-midia", "Pausar a mídia", "Pausa música e vídeo durante o ditado e só retoma o que estava tocando.", c.pausarMidia, 'data-muda="cfg-liga" data-k="pausarMidia"') +
        VD.opcao("c-sem-fala", "Descartar ditado sem fala", "Se nenhuma fala for detectada, não envia nada ao provedor, não cola e não guarda no histórico.", c.descartarSemFala, 'data-muda="cfg-liga" data-k="descartarSemFala"') + "</div>";
    }, function () { return ""; });
  });
  VD.acoes["cfg-liga"] = function (t) { S.cfg[t.getAttribute("data-k")] = t.checked; VD.toast("Preferência salva."); if (t.getAttribute("data-k") === "sons") VD.render(); };
  VD.acoes["cfg-idioma"] = function (t) { S.cfg.idioma = t.value; VD.toast("Idioma salvo."); };
  VD.acoes["cfg-mic"] = function (t) { S.cfg.mic = t.value; S.cfg.micConectado = true; VD.render(); };
  VD.acoes["cfg-mic-sim"] = function () { S.cfg.micConectado = !S.cfg.micConectado; VD.render(); };
  VD.acoes["cfg-som"] = function (t) { S.cfg.som = t.value; };
  VD.acoes["cfg-previa"] = function (a) { if (a.getAttribute("aria-disabled") !== "true") VD.toast("♪ Prévia do som “" + S.cfg.som + "” (simulada)."); };

  VD.rota("config/atalhos", function () {
    var h = cab() + abas("atalhos") + '<div class="cartao"><h2>Atalho global</h2><p class="dica">Funciona em qualquer programa. Acione para começar a gravar e de novo para parar.</p><form data-envia="atalho-salva" novalidate><div class="campo" style="max-width:340px"><label for="a-g">Atalho</label><input id="a-g" type="text" value="' + esc(VD.atalhoGlobal()) + '" aria-describedby="a-e a-d"><span class="erro-campo" id="a-e" role="alert"></span><span class="dica" id="a-d">Use pelo menos uma tecla modificadora (' + (S.plat === "mac" ? "⌘, ⌥, ⌃ ou ⇧" : "Ctrl, Alt, Shift ou Win") + "). Atalhos do sistema, como " + (S.plat === "mac" ? "⌘Espaço" : "Win+L") + ", são recusados.</span></div><button class=\"btn primario\">Salvar atalho</button></form></div>";
    h += '<div class="cartao"><h2>Outros atalhos</h2><div class="linha-opcao"><div class="texto"><b>Cancelar o ditado</b><span class="dica">Aperte duas vezes em até 1,5 s enquanto grava. Uma vez só passa para o programa em foco.</span></div><kbd>Esc</kbd> <kbd>Esc</kbd></div>';
    return h + "</div>";
  });
  VD.acoes["atalho-salva"] = function () {
    var i = $("#a-g"), e = $("#a-e"), v = i.value.trim(); i.removeAttribute("aria-invalid"); e.textContent = "";
    var emUso = S.plat === "mac" ? /^⌘\s*espaço$/i : /^win\+l$/i;
    if (!/(⌘|⌥|⌃|⇧|ctrl|alt|shift|win|cmd)/i.test(v)) e.textContent = "O atalho precisa de uma tecla modificadora. Só uma letra não vale.";
    else if (emUso.test(v)) e.textContent = "Este atalho já é usado pelo sistema. Escolha outra combinação.";
    if (e.textContent) { i.setAttribute("aria-invalid", "true"); i.focus(); return; }
    S.cfg.atalhoGlobal[S.plat] = v; VD.toast("Atalho salvo. Já vale, sem reiniciar o app.");
  };

  VD.rota("config/provedores", function () {
    var k = S.chaves.groq;
    return cab() + abas("provedores") + VD.estadoLista(function () {
      return '<div class="cartao"><h2>Groq <span class="selo ' + (k.salva ? (k.estado === "válida" ? "ok" : "erro") : "") + '">' + (k.salva ? (k.estado === "válida" ? "Chave válida" : "Chave inválida") : "Sem chave") + '</span></h2><p class="dica">Transcrição na nuvem. É o único provedor da v1 e sem chave não há ditado. O custo de uso é seu, na sua conta. A chave fica no cofre do ' + (S.plat === "mac" ? "Chaves do macOS" : "Windows") + "; nunca em arquivo.</p>" +
        (k.salva ? '<div class="linha-opcao"><div class="texto"><b class="mono">••••••••••••' + esc(k.fim) + '</b><span class="dica">Só os 4 últimos caracteres aparecem.</span></div><div class="acoes"><button class="btn" data-acao="chave-trocar">Trocar chave</button><button class="btn perigo" data-acao="chave-apagar">Apagar</button></div></div>' + (k.estado !== "válida" ? '<div class="aviso erro" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo">O Groq recusou esta chave. Confira se copiou inteira ou crie outra.</div></div>' : "")
          : '<form data-envia="chave-salva" novalidate><div class="campo" style="max-width:460px"><label for="k-chave">Chave de API</label><input id="k-chave" type="text" autocomplete="off" placeholder="gsk_…" aria-describedby="k-e"><span class="erro-campo" id="k-e" role="alert"></span></div><div class="acoes"><button class="btn primario">Salvar e testar</button><a class="btn discreto" href="https://console.groq.com/keys" target="_blank" rel="noopener">Onde consigo uma chave? ↗</a></div></form>') + "</div>";
    }, function () { return ""; });
  });
  VD.acoes["chave-salva"] = function () {
    var i = $("#k-chave"), e = $("#k-e"), v = i.value.trim(); i.removeAttribute("aria-invalid"); e.textContent = "";
    if (!v) { e.textContent = "Cole a chave de API do Groq."; i.setAttribute("aria-invalid", "true"); return; }
    VD.toast("Testando a chave no Groq…");
    setTimeout(function () { S.chaves.groq = { salva: true, fim: v.slice(-4).padStart(4, "x"), estado: /invalida/i.test(v) ? "inválida" : "válida" }; VD.toast(S.chaves.groq.estado === "válida" ? "Chave válida e salva no cofre do sistema." : "O Groq recusou a chave.", S.chaves.groq.estado === "válida" ? "" : "erro"); VD.render(); }, 900);
  };
  VD.acoes["chave-trocar"] = function () { S.chaves.groq.salva = false; VD.render(); };
  VD.acoes["chave-apagar"] = function () { VD.confirma("Apagar a chave do Groq?", "Ela sai do cofre do sistema. O ditado deixa de funcionar até você cadastrar outra chave.", "Apagar chave", function () { S.chaves.groq = { salva: false }; VD.render(); }); };
  VD.acoes["ollama-sim"] = function () { S.ollama.ativo = !S.ollama.ativo; VD.render(); };

  var dl = null;
  VD.rota("config/local", function () {
    var m = S.modelo;
    var h = cab() + abas("local") + '<div class="cartao"><h2>Whisper local <span class="selo ' + (m.estado === "baixado" ? "ok" : m.estado === "erro" ? "erro" : "") + '">' + { baixado: "Baixado", nao: "Não baixado", baixando: "Baixando…", pausado: "Pausado", erro: "Download interrompido" }[m.estado] + "</span></h2><p class=\"dica\">Transcrição na sua máquina: funciona sem internet e o áudio não sai daqui. Modelo: whisper-large-v3-turbo, cerca de 1,6 GB.</p>";
    if (m.estado === "baixando" || m.estado === "pausado" || m.estado === "erro") h += '<div class="progresso" role="progressbar" aria-valuenow="' + m.pct + '" aria-valuemin="0" aria-valuemax="100" aria-label="Progresso do download"><i style="width:' + m.pct + '%"></i></div><p class="dica" style="margin:6px 0 12px">' + m.pct + "% de 1,6 GB</p>";
    if (m.estado === "erro") h += '<div class="aviso erro" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo">A conexão caiu no meio do download. O arquivo parcial não é usado. Retome de onde parou ou recomece.</div></div>';
    h += '<div class="acoes">' + {
      nao: '<button class="btn primario" data-acao="dl-inicia">Baixar modelo</button>',
      baixando: '<button class="btn" data-acao="dl-pausa">Pausar</button><button class="btn" data-acao="dl-sim-erro">Simular: queda de conexão</button>',
      pausado: '<button class="btn primario" data-acao="dl-inicia">Retomar</button><button class="btn" data-acao="dl-reinicia">Recomeçar</button>',
      erro: '<button class="btn primario" data-acao="dl-inicia">Retomar</button><button class="btn" data-acao="dl-reinicia">Recomeçar</button>',
      baixado: '<button class="btn perigo" data-acao="dl-apaga">Apagar modelo</button>'
    }[m.estado] + "</div></div>";
    if (m.estado !== "baixado") h += '<div class="aviso alerta"><span aria-hidden="true">ℹ️</span><div class="conteudo">Enquanto o modelo não for baixado, modos com transcrição local avisam antes de gravar.</div></div>';
    return h;
  });
  function tickDl() { clearInterval(dl); dl = setInterval(function () { S.modelo.pct = Math.min(100, S.modelo.pct + 7); if (S.modelo.pct >= 100) { S.modelo.estado = "baixado"; clearInterval(dl); VD.toast("Modelo local pronto. Já dá para ditar sem internet."); } if (location.hash.indexOf("config/local") >= 0) VD.render(); }, 450); }
  VD.acoes["dl-inicia"] = function () { S.modelo.estado = "baixando"; tickDl(); VD.render(); };
  VD.acoes["dl-pausa"] = function () { clearInterval(dl); S.modelo.estado = "pausado"; VD.render(); };
  VD.acoes["dl-sim-erro"] = function () { clearInterval(dl); S.modelo.estado = "erro"; VD.render(); };
  VD.acoes["dl-reinicia"] = function () { S.modelo.pct = 0; S.modelo.estado = "baixando"; tickDl(); VD.render(); };
  VD.acoes["dl-apaga"] = function () { VD.confirma("Apagar o modelo local?", "Libera 1,6 GB. Modos com transcrição local deixam de funcionar até baixar de novo.", "Apagar modelo", function () { S.modelo = { estado: "nao", pct: 0 }; VD.render(); }); };

  VD.rota("config/dados", function () {
    var op = function (id, v) { return ["1", "7", "30", "90", "nunca"].map(function (x) { return '<option value="' + x + '"' + (x === v ? " selected" : "") + ">" + (x === "nunca" ? "Não apagar" : x + (x === "1" ? " dia" : " dias")) + "</option>"; }).join(""); };
    return cab() + abas("dados") + '<div class="cartao"><h2>Retenção</h2><p class="dica">Áudios e textos dos ditados podem conter dados de clientes. A limpeza roda ao abrir o app e depois a cada 24 h.</p><div class="duas-col"><div class="campo"><label for="r-audio">Apagar áudios depois de</label><select id="r-audio" data-muda="ret" data-k="retAudio">' + op("a", S.cfg.retAudio) + '</select></div><div class="campo"><label for="r-hist">Apagar itens do histórico depois de</label><select id="r-hist" data-muda="ret" data-k="retHist">' + op("h", S.cfg.retHist) + "</select></div></div></div>" +
      '<div class="cartao"><h2>Apagar agora</h2><p class="dica">Remove todo o histórico e todos os áudios desta máquina. Não dá para desfazer.</p><button class="btn perigo" data-acao="apaga-tudo">Apagar todo o histórico e áudios…</button></div>' +
      '<div class="cartao"><h2>Registro de erros</h2><p class="dica">Guarda eventos e tempos por 7 dias, até 20 MB — sem texto ditado, sem áudio e sem chaves.</p><button class="btn" data-acao="abre-log">Abrir pasta do registro</button></div>';
  });
  VD.acoes["ret"] = function (t) { S.cfg[t.getAttribute("data-k")] = t.value; VD.toast("Prazo salvo. Vale na próxima limpeza."); };
  VD.acoes["apaga-tudo"] = function () { VD.confirma("Apagar todo o histórico e os áudios?", "Serão apagados " + S.hist.length + " ditados e seus áudios desta máquina. Não dá para desfazer.", "Apagar tudo", function () { S.hist = []; VD.toast("Histórico e áudios apagados."); VD.render(); }); };
  VD.acoes["abre-log"] = function () { VD.toast("Abrindo a pasta do registro no " + (S.plat === "mac" ? "Finder" : "Explorador de Arquivos") + " (simulado)."); };

  VD.rota("config/permissoes", function () {
    var p = S.perm, mac = S.plat === "mac";
    var linha = function (k, tit, desc, nega, onde) { return '<div class="permissao"><div class="texto"><b>' + tit + "</b><span class=\"dica\">" + desc + '</span></div><span class="selo ' + (p[k] ? "ok" : "erro") + '">' + (p[k] ? "✓ Concedida" : "Negada") + "</span>" + (p[k] ? '<button class="btn pequeno" data-acao="perm-sim" data-k="' + k + '">Simular: negar</button>' : '<button class="btn pequeno primario" data-acao="perm-abre" data-k="' + k + '">Abrir ' + onde + "</button>") + "</div>" + (p[k] ? "" : '<div class="aviso alerta" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo"><b>O que deixa de funcionar:</b> ' + nega + "</div></div>"); };
    return cab() + abas("permissoes") + '<div class="cartao"><h2>Permissões do sistema</h2><p class="dica">Você pode conceder ou revisar cada uma depois, a qualquer momento.</p>' +
      linha("mic", "Microfone", "Para gravar a sua voz.", "nenhum ditado grava; o gravador mostra “Sem permissão do microfone”.", mac ? "Ajustes do Sistema" : "Configurações do Windows") +
      linha("acess", mac ? "Acessibilidade" : "Colar e ler a seleção", mac ? "Para colar o texto no programa em foco e ler o texto selecionado." : "O Windows pode bloquear a colagem em programas executados como administrador.", "o texto não é colado; fica na área de transferência com um aviso, e a seleção é ignorada como contexto.", mac ? "Ajustes do Sistema" : "Configurações do Windows") + "</div>";
  });
  VD.acoes["perm-sim"] = function (a) { S.perm[a.getAttribute("data-k")] = false; VD.render(); };
  VD.acoes["perm-abre"] = function (a) { S.perm[a.getAttribute("data-k")] = true; VD.toast("Permissão concedida (simulado)."); VD.render(); };

  /* ================= ÍCONE E MENU ================= */
  var estIc = "parado";
  VD.rota("bandeja", function () {
    var mac = S.plat === "mac", icones = { parado: "🎙️", gravando: "🔴", processando: "⏳", erro: "⚠️" };
    var h = VD.cabecalho("Ícone e menu", "O Voice Desk vive na " + (mac ? "barra de menus" : "bandeja do sistema") + ". O ícone mostra o estado e o menu dá acesso rápido sem abrir a janela.");
    h += '<div class="seg" role="group" aria-label="Estado do ícone" style="margin-bottom:16px;border-color:var(--cor-borda-forte)">' + ["parado", "gravando", "processando", "erro"].map(function (e) { return '<button type="button" style="color:var(--cor-texto)' + (estIc === e ? ";background:var(--cor-primaria);color:var(--cor-sobre-primaria)" : "") + '" data-acao="ic-estado" data-e="' + e + '" aria-pressed="' + (estIc === e) + '">' + e[0].toUpperCase() + e.slice(1) + "</button>"; }).join("") + "</div>";
    h += '<div class="bandeja-demo"><div style="display:flex;flex-direction:column;gap:0"><div class="barra-sistema"><span>Wi‑Fi</span><span>🔋 82%</span><span class="estado-icone" aria-label="Ícone do Voice Desk: ' + estIc + '"><b aria-hidden="true">' + icones[estIc] + "</b> Voice Desk</span><span>" + (mac ? "sex 23 out 10:12" : "10:12") + "</span></div></div>" +
      '<div class="menu-bandeja" role="menu" aria-label="Menu do Voice Desk"><div class="titulo-menu">Ditado</div><button role="menuitem" data-acao="ic-ditado">' + (estIc === "gravando" ? "Parar ditado" : "Iniciar ditado") + "<kbd>" + esc(VD.atalhoGlobal()) + '</kbd></button><div class="sep" role="separator"></div><div class="titulo-menu">Adicionar ao vocabulário</div><form class="caixa-rapida" data-envia="ic-voc"><label class="so-leitor" for="ic-v">Nova palavra</label><div class="acoes" style="flex-wrap:nowrap"><input id="ic-v" type="text" placeholder="Ex.: Hortolândia"><button class="btn pequeno primario">Adicionar</button></div></form><div class="sep" role="separator"></div><a role="menuitem" class="btn discreto" style="display:flex;justify-content:flex-start;width:100%" href="#/config/geral">Abrir configurações</a><button role="menuitem" data-acao="ic-sair">Sair do Voice Desk</button></div></div>';
    h += "<p class=\"dica\" style=\"margin-top:16px\">O ícone muda para refletir parado, gravando, processando ou erro; a cor nunca é o único sinal (há símbolo e texto na dica do ícone).</p>";
    return h;
  });
  VD.acoes["ic-estado"] = function (a) { estIc = a.getAttribute("data-e"); VD.render(); };
  VD.acoes["ic-ditado"] = function () { VD.vai("ditado"); };
  VD.acoes["ic-modo"] = function (a) { var id = a.getAttribute("data-id"); S.modos.forEach(function (m) { m.padrao = m.id === id; }); VD.render(); VD.toast("Modo padrão alterado."); };
  VD.acoes["ic-voc"] = function () { var i = $("#ic-v"), v = i.value.trim(); if (!v) return; if (S.vocab.indexOf(v) < 0) S.vocab.unshift(v); i.value = ""; VD.toast("“" + v + "” adicionada ao vocabulário, sem abrir as configurações."); };
  VD.acoes["ic-sair"] = function () { VD.confirma("Sair do Voice Desk?", "O atalho global deixa de funcionar até você abrir o app de novo.", "Sair", function () { VD.toast("O app seria encerrado (simulado)."); }); };

  /* ================= PRIMEIRO USO ================= */
  var passo = 0, pulou = {}, nomesPasso = ["Boas-vindas", "Permissões", "Atalho", "Chave do Groq", "Teste"];
  VD.rota("primeiro-uso", function () {
    var mac = S.plat === "mac", h = VD.cabecalho("Primeiro uso", "Em até 10 minutos você faz o primeiro ditado. Qualquer passo pode ser pulado e retomado depois em Configurações.");
    h += '<ol class="trilha" aria-label="Passos">' + nomesPasso.map(function (n, i) { return '<li' + (i === passo ? ' aria-current="step"' : "") + (i < passo ? ' class="feito"' : "") + ">" + (i < passo ? "✓ " : i + 1 + ". ") + n + "</li>"; }).join("") + "</ol>";
    var corpo = "";
    if (passo === 0) corpo = "<h2>Bem-vindo ao Voice Desk</h2><p style=\"margin:8px 0 16px\" class=\"suave\">Aperte um atalho em qualquer programa, fale e solte. O texto aparece onde o cursor estava. Gratuito, de código aberto, e a sua configuração e as suas chaves ficam com você.</p><div class=\"aviso info\"><span aria-hidden=\"true\">🔒</span><div class=\"conteudo\">Seu áudio e seus textos ficam nesta máquina. Só vão ao Groq, para a transcrição.</div></div>";
    if (passo === 1) corpo = "<h2>Permissões</h2><p class=\"dica\">Pedimos só o necessário e explicamos para quê.</p>" +
      '<div class="permissao"><div class="texto"><b>Microfone</b><span class="dica">Para gravar a sua voz.</span></div><span class="selo ' + (S.perm.mic ? "ok" : "") + '">' + (S.perm.mic ? "✓ Concedida" : "Pendente") + '</span><button class="btn pequeno primario" data-acao="pu-perm" data-k="mic"' + (S.perm.mic ? " disabled" : "") + ">Conceder</button></div>" +
      '<div class="permissao"><div class="texto"><b>' + (mac ? "Acessibilidade" : "Colar em outros programas") + '</b><span class="dica">' + (mac ? "Para colar o texto no programa em foco e ler a seleção." : "O Windows pode pedir confirmação na primeira colagem.") + '</span></div><span class="selo ' + (S.perm.acess ? "ok" : "") + '">' + (S.perm.acess ? "✓ Concedida" : "Pendente") + '</span><button class="btn pequeno primario" data-acao="pu-perm" data-k="acess"' + (S.perm.acess ? " disabled" : "") + ">Conceder</button></div>" +
      (!S.perm.mic || !S.perm.acess ? '<div class="aviso alerta" style="margin-top:12px"><span aria-hidden="true">⚠️</span><div class="conteudo">Sem estas permissões, o ditado não grava ou o texto não é colado. Você pode conceder depois em Configurações › Permissões.</div></div>' : "");
    if (passo === 2) corpo = '<h2>Escolha o atalho</h2><p class="dica">Combinação que você aciona em qualquer programa.</p><div class="campo" style="max-width:320px;margin-top:12px"><label for="pu-at">Atalho global</label><input id="pu-at" type="text" value="' + esc(VD.atalhoGlobal()) + '"></div><p class="dica">Você pode mudar quando quiser em Configurações › Atalhos.</p>';
    if (passo === 3) corpo = '<h2>Chave do Groq</h2><p class="dica">O Voice Desk transcreve pelo Groq. Sem chave não há ditado.</p><div class="cartao" style="box-shadow:none"><h3>☁️ Groq (nuvem) — rápido e preciso</h3><p class="dica">Precisa de uma chave de API gratuita ou paga, criada por você no site do Groq.</p><div class="campo" style="max-width:420px;margin-top:8px"><label for="pu-k">Chave de API do Groq</label><input id="pu-k" type="text" placeholder="gsk_…"></div><a href="https://console.groq.com/keys" target="_blank" rel="noopener">Como obter uma chave ↗</a></div>';
    if (passo === 4) corpo = '<h2>Faça um ditado de teste</h2><p class="dica">Clique na caixa, acione o atalho <kbd>' + esc(VD.atalhoGlobal()) + '</kbd> e fale uma frase. (No protótipo, use o botão.)</p><div class="campo" style="margin-top:12px"><label for="pu-teste">Caixa de teste</label><textarea id="pu-teste" placeholder="O texto vai aparecer aqui"></textarea></div><button class="btn primario" data-acao="pu-testa">Simular ditado de teste</button>';
    h += '<div class="cartao">' + corpo + '</div><div class="acoes" style="margin-top:16px;justify-content:space-between"><div class="acoes">' + (passo > 0 ? '<button class="btn" data-acao="pu-volta">Voltar</button>' : "") + '</div><div class="acoes">' + (passo > 0 && passo < 4 ? '<button class="btn discreto" data-acao="pu-pula">Pular este passo</button>' : "") + '<button class="btn primario" data-acao="pu-segue">' + (passo === 4 ? "Concluir" : "Continuar") + "</button></div></div>";
    return h;
  });
  VD.acoes["pu-segue"] = function () { if (passo === 4) { passo = 0; VD.toast("Tudo pronto. O Voice Desk fica na " + (S.plat === "mac" ? "barra de menus" : "bandeja") + "."); VD.vai("ditado"); return; } passo++; VD.render(); };
  VD.acoes["pu-volta"] = function () { passo = Math.max(0, passo - 1); VD.render(); };
  VD.acoes["pu-pula"] = function () { pulou[passo] = true; passo++; VD.render(); VD.toast("Passo pulado. Você retoma em Configurações."); };
  VD.acoes["pu-perm"] = function (a) { S.perm[a.getAttribute("data-k")] = true; VD.render(); };
  VD.acoes["pu-testa"] = function () { $("#pu-teste").value = "Funcionou! Este é o meu primeiro ditado com o Voice Desk."; VD.toast("Primeiro ditado concluído."); };

  /* ================= SOBRE ================= */
  VD.rota("sobre", function () {
    return VD.cabecalho("Sobre o Voice Desk") + '<div class="cartao"><div class="meta" style="margin-top:0"><div><dt>Versão instalada</dt><dd>' + D.versao + '</dd></div><div><dt>Licença</dt><dd>Código aberto (licença a definir)</dd></div><div><dt>Plataforma</dt><dd>' + (S.plat === "mac" ? "macOS" : "Windows") + '</dd></div></div><p style="margin:12px 0"><a href="projeto.html">Página do projeto: instaladores, código-fonte e licença ↗</a></p><div class="aviso info"><span aria-hidden="true">ℹ️</span><div class="conteudo">O Voice Desk é um projeto independente e <b>não tem relação com o VoiceInk</b>.</div></div></div>';
  });
  VD.iniciar();
})();
