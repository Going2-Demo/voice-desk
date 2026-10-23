/* Telas: Modos (F-10 a F-15), Vocabulário (F-07), Substituições (F-08), Limpeza (F-09), Histórico (F-16 a F-19). */
(function () {
  var VD = window.VD, S = VD.S, D = VD.D, esc = VD.esc;
  var $ = function (s, r) { return (r || document).querySelector(s); };
  var provNome = { groq: "Groq", local: "Whisper local", ollama: "Ollama" };

  /* ================= MODOS ================= */
  var ed = null, edNovo = false;
  function novoModo() {
    return { id: "novo", emoji: "✨", nome: "", padrao: false, idioma: "pt-BR", transc: "groq", modeloTransc: "whisper-large-v3-turbo", limpeza: true, formatacao: true,
      refina: false, ia: "groq", modeloIa: D.modelosIa.groq[0], prompt: "", ctxSel: false, ctxClip: false, enter: false, programas: { mac: [], win: [] }, atalho: { mac: "", win: "" }, gatilhos: [] };
  }
  function resumoModo(m) {
    var s = [];
    s.push('<span class="selo">' + (m.transc === "local" ? "🔒 Local" : "☁️ Groq") + "</span>");
    s.push('<span class="selo ' + (m.refina ? "info" : "") + '">' + (m.refina ? "IA: " + provNome[m.ia] : "Sem IA") + "</span>");
    if (m.enter) s.push('<span class="selo">↵ Envia Enter</span>');
    return s.join("");
  }
  VD.rota("modos", function () {
    var novo = '<a class="btn primario" href="#/modos/novo">＋ Novo modo</a>';
    return VD.cabecalho("Modos", "Um modo define como o ditado vira texto: idioma, provedor, limpeza, refinamento por IA e onde ele entra sozinho.", novo) +
      VD.estadoLista(function () {
        var h = '<div class="grade">';
        S.modos.forEach(function (m) {
          var progs = m.programas[S.plat];
          h += '<a class="cartao cartao-modo" href="#/modos/' + m.id + '"><div class="topo"><span class="emoji" aria-hidden="true">' + m.emoji + '</span><div><h3>' + esc(m.nome) + "</h3>" +
            (m.padrao ? '<span class="selo ok">Modo padrão</span>' : "") + '</div></div><div class="selos">' + resumoModo(m) + "</div>" +
            '<p class="dica">' + (progs.length ? "Entra em: " + esc(progs.join(", ")) : (m.padrao ? "Usado quando nenhum outro se aplica." : "Sem programa associado.")) + "</p>" +
            (m.atalho[S.plat] ? '<p class="dica">Atalho: <kbd>' + esc(m.atalho[S.plat]) + "</kbd></p>" : "") + "</a>";
        });
        return h + "</div>";
      }, function () {
        return VD.vazio("🧩", "Nenhum modo ainda", "O modo padrão sempre existe. Crie outros para e-mail, chat ou tradução.", '<a class="btn primario" href="#/modos/novo">Criar modo</a>');
      });
  });
  function lerForm() {
    var f = $("#form-modo"); if (!f || !ed) return;
    ed.nome = f.nome.value; ed.emoji = f.emoji.value; ed.idioma = f.idioma.value; ed.transc = f.transc.value; ed.modeloTransc = f.modeloTransc ? f.modeloTransc.value : ed.modeloTransc;
    ed.limpeza = f.limpeza.checked; ed.formatacao = f.formatacao.checked; ed.refina = f.refina.checked; ed.ia = f.ia.value; ed.modeloIa = f.modeloIa.value;
    ed.prompt = f.prompt.value; ed.ctxSel = f.ctxSel.checked; ed.ctxClip = f.ctxClip.checked; ed.enter = f.enter.checked; ed.atalho[S.plat] = f.atalho.value;
  }
  function editor(id) {
    if (!ed || ed._id !== id) {
      var base = id === "novo" ? novoModo() : VD.clone(VD.modoPorId(id) || novoModo());
      ed = base; ed._id = id; edNovo = id === "novo";
    }
    var m = ed, o = function (v, a) { return v === a ? " selected" : ""; };
    var progs = m.programas[S.plat];
    var h = '<p class="migalha"><a href="#/modos">← Modos</a></p>' + VD.cabecalho(edNovo ? "Novo modo" : m.emoji + " " + m.nome, "Mudanças valem para os próximos ditados" + (S.sync.ligada ? " e seguem para a outra máquina pela sincronização." : "."),
      edNovo ? "" : '<button type="button" class="btn" data-acao="modo-duplicar">Duplicar</button><button type="button" class="btn perigo" data-acao="modo-excluir"' + (m.padrao ? ' aria-disabled="true" title="O modo padrão não pode ser excluído. Escolha outro como padrão antes."' : "") + ">Excluir</button>");
    if (m.padrao) h += '<div class="aviso info"><span aria-hidden="true">ℹ️</span><div class="conteudo">Este é o modo padrão: não pode ser excluído enquanto for o padrão.</div></div>';
    h += '<form id="form-modo" data-envia="modo-salvar" novalidate><div class="cartao"><h2>Identidade</h2><div class="duas-col" style="margin-top:12px">' +
      '<div class="campo"><label for="m-nome">Nome do modo</label><input id="m-nome" name="nome" type="text" maxlength="40" value="' + esc(m.nome) + '" aria-describedby="m-nome-e" autocomplete="off"><span class="dica">Até 40 caracteres, único.</span><span class="erro-campo" id="m-nome-e" role="alert"></span></div>' +
      '<div class="campo"><label for="m-emoji">Emoji</label><input id="m-emoji" name="emoji" type="text" maxlength="4" value="' + esc(m.emoji) + '" style="max-width:100px"></div></div></div>' +
      '<div class="cartao"><h2>Transcrição</h2><div class="duas-col" style="margin-top:12px">' +
      '<div class="campo"><label for="m-idioma">Idioma do ditado</label><select id="m-idioma" name="idioma"><option value="pt-BR"' + o(m.idioma, "pt-BR") + '>Português do Brasil</option><option value="en"' + o(m.idioma, "en") + '>Inglês</option><option value="auto"' + o(m.idioma, "auto") + ">Detecção automática</option></select></div>" +
      '<div class="campo"><label for="m-transc">Provedor</label><select id="m-transc" name="transc" data-muda="modo-prov"><option value="groq"' + o(m.transc, "groq") + '>Groq (nuvem)</option><option value="local"' + o(m.transc, "local") + ">Whisper local (sem internet)</option></select></div>" +
      '<div class="campo"><label for="m-mt">Modelo</label><select id="m-mt" name="modeloTransc">' + (m.transc === "local" ? '<option selected>whisper-large-v3-turbo (local)</option>' : D.modelosTransc.map(function (x) { return "<option" + o(m.modeloTransc, x) + ">" + x + "</option>"; }).join("")) + "</select></div></div>" +
      (m.transc === "local" && S.modelo.estado !== "baixado" ? '<div class="aviso alerta" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo">O modelo local ainda não foi baixado. Este modo avisará antes de gravar. <a href="#/config/local">Baixar modelo</a></div></div>' : "") + "</div>" +
      '<div class="cartao"><h2>Limpeza do texto</h2><p class="dica">Sem IA, antes do refinamento.</p>' +
      VD.opcao("m-limpeza", "Remover palavras de preenchimento", "Tira “é…”, “hã”, “tipo”, “né” e outras da <a href='#/limpeza'>lista</a>.", m.limpeza, 'name="limpeza"') +
      VD.opcao("m-formatacao", "Formatar o texto", "Primeira letra maiúscula, ponto final e sem espaços duplos.", m.formatacao, 'name="formatacao"') + "</div>" +
      '<div class="cartao"><h2>Refinamento por IA</h2>' +
      VD.opcao("m-refina", "Refinar o texto com IA", "Envia o texto limpo ao provedor de IA com o prompt abaixo.", m.refina, 'name="refina" data-muda="modo-refina"') +
      '<div id="bloco-ia"' + (m.refina ? "" : " hidden") + '><div class="duas-col" style="margin-top:12px"><div class="campo"><label for="m-ia">Provedor de IA</label><select id="m-ia" name="ia" data-muda="modo-ia"><option value="groq"' + o(m.ia, "groq") + '>Groq (nuvem)</option><option value="ollama"' + o(m.ia, "ollama") + ">Ollama (nesta máquina)</option></select></div>" +
      '<div class="campo"><label for="m-mi">Modelo</label><select id="m-mi" name="modeloIa">' + D.modelosIa[m.ia].map(function (x) { return "<option" + o(m.modeloIa, x) + ">" + x + "</option>"; }).join("") + "</select><span class=\"dica\">Lista vinda do provedor.</span></div></div>" +
      '<div class="campo"><label for="m-prompt">Prompt</label><textarea id="m-prompt" name="prompt" maxlength="4000" data-entra="modo-contador" aria-describedby="m-prompt-d">' + esc(m.prompt) + '</textarea><span class="dica" id="m-prompt-d"><span id="contador">' + m.prompt.length + "</span> de 4.000 caracteres. Diga à IA como refinar; ela devolve só o texto final.</span></div>" +
      VD.opcao("m-ctxsel", "Usar o texto selecionado como contexto", "Lê a seleção do programa em foco (até 8.000 caracteres). Precisa da permissão de acessibilidade.", m.ctxSel, 'name="ctxSel"') +
      VD.opcao("m-ctxclip", "Usar a área de transferência como contexto", "Envia o que estava copiado no início do ditado (até 8.000 caracteres).", m.ctxClip, 'name="ctxClip"') + "</div></div>" +
      '<div class="cartao"><h2>Colagem</h2>' + VD.opcao("m-enter", "Enviar Enter depois de colar", "Útil em chats: o texto cola e a mensagem é enviada.", m.enter, 'name="enter"') + "</div>" +
      '<div class="cartao"><h2>Quando usar este modo</h2><p class="dica">Ordem de escolha: palavra-gatilho, atalho do modo, programa em foco, modo padrão.</p>' +
      '<div class="campo"><span class="rotulo">Programas neste ' + (S.plat === "mac" ? "Mac" : "Windows") + '</span><div class="lista-chips">' +
      (progs.length ? progs.map(function (p, i) { return '<span class="chip">' + esc(p) + '<button type="button" data-acao="modo-prog-rm" data-i="' + i + '" aria-label="Remover ' + esc(p) + '">×</button></span>'; }).join("") : '<span class="dica">Nenhum programa.</span>') + "</div>" +
      '<div class="acoes" style="margin-top:8px"><label class="so-leitor" for="m-addprog">Adicionar programa</label><select id="m-addprog" style="max-width:260px">' + D.programas[S.plat].filter(function (p) { return progs.indexOf(p) < 0; }).map(function (p) { return "<option>" + p + "</option>"; }).join("") + '</select><button type="button" class="btn" data-acao="modo-prog-add">Associar</button></div><span class="erro-campo" id="m-prog-e" role="alert"></span><span class="dica">Cada programa pertence a um único modo. A lista mostra os programas abertos ou instalados nesta máquina.</span></div>' +
      '<div class="campo"><label for="m-atalho">Atalho deste modo (opcional)</label><input id="m-atalho" name="atalho" type="text" value="' + esc(m.atalho[S.plat]) + '" placeholder="Ex.: ' + (S.plat === "mac" ? "⌥⇧E" : "Ctrl+Alt+E") + '" style="max-width:240px" aria-describedby="m-atalho-e"><span class="erro-campo" id="m-atalho-e" role="alert"></span><span class="dica">Precisa de pelo menos uma tecla modificadora, diferente do atalho global e dos outros modos. Acionar de novo para o ditado.</span></div>' +
      '<div class="campo"><span class="rotulo">Palavras-gatilho</span><div class="lista-chips">' + (m.gatilhos.length ? m.gatilhos.map(function (g, i) { return '<span class="chip">' + esc(g) + '<button type="button" data-acao="modo-gat-rm" data-i="' + i + '" aria-label="Remover gatilho ' + esc(g) + '">×</button></span>'; }).join("") : '<span class="dica">Nenhuma.</span>') + "</div>" +
      '<div class="acoes" style="margin-top:8px"><label class="so-leitor" for="m-addgat">Nova palavra-gatilho</label><input id="m-addgat" type="text" style="max-width:220px" placeholder="Ex.: e-mail"><button type="button" class="btn" data-acao="modo-gat-add">Adicionar</button></div><span class="erro-campo" id="m-gat-e" role="alert"></span><span class="dica">Se as 3 primeiras palavras do ditado contêm o gatilho, este modo é usado e a palavra some do texto.</span></div></div>' +
      '<div class="acoes" style="margin-top:20px"><button type="submit" class="btn primario">Salvar modo</button>' + (!edNovo && !m.padrao ? '<button type="button" class="btn" data-acao="modo-padrao">Tornar modo padrão</button>' : "") + '<a class="btn discreto" href="#/modos" data-acao="modo-cancela">Cancelar</a></div></form>';
    return h;
  }
  VD.rota("modos/:id", function (id) { return editor(id); });
  VD.acoes["modo-cancela"] = function () { ed = null; };
  VD.acoes["modo-prov"] = function () { lerForm(); VD.render(); };
  VD.acoes["modo-ia"] = function () { lerForm(); ed.modeloIa = D.modelosIa[ed.ia][0]; VD.render(); };
  VD.acoes["modo-refina"] = function (t) { $("#bloco-ia").hidden = !t.checked; };
  VD.acoes["modo-contador"] = function (t) { $("#contador").textContent = t.value.length; };
  VD.acoes["modo-prog-add"] = function () {
    lerForm(); var p = $("#m-addprog").value, e = $("#m-prog-e"); if (!p) return;
    var dono = S.modos.filter(function (m) { return m.id !== ed._id && m.programas[S.plat].indexOf(p) >= 0; })[0];
    if (dono) { e.textContent = p + " já pertence ao modo " + dono.nome + ". Remova de lá antes de associar aqui."; return; }
    ed.programas[S.plat].push(p); VD.render();
  };
  VD.acoes["modo-prog-rm"] = function (a) { lerForm(); ed.programas[S.plat].splice(+a.getAttribute("data-i"), 1); VD.render(); };
  VD.acoes["modo-gat-add"] = function () {
    lerForm(); var v = $("#m-addgat").value.trim().toLowerCase(), e = $("#m-gat-e"); if (!v) { e.textContent = "Digite a palavra-gatilho."; return; }
    var dono = S.modos.filter(function (m) { return m.id !== ed._id && m.gatilhos.indexOf(v) >= 0; })[0];
    if (dono || ed.gatilhos.indexOf(v) >= 0) { e.textContent = "“" + v + "” já é gatilho do modo " + (dono ? dono.nome : "atual") + ". Cada palavra pertence a um único modo."; return; }
    ed.gatilhos.push(v); VD.render();
  };
  VD.acoes["modo-gat-rm"] = function (a) { lerForm(); ed.gatilhos.splice(+a.getAttribute("data-i"), 1); VD.render(); };
  VD.acoes["modo-salvar"] = function () {
    lerForm(); var ok = true, n = $("#m-nome"), en = $("#m-nome-e"), at = $("#m-atalho"), ea = $("#m-atalho-e");
    n.removeAttribute("aria-invalid"); at.removeAttribute("aria-invalid"); en.textContent = ""; ea.textContent = "";
    var nome = ed.nome.trim();
    if (!nome) { en.textContent = "Dê um nome ao modo."; ok = false; }
    else if (S.modos.some(function (m) { return m.id !== ed._id && m.nome.toLowerCase() === nome.toLowerCase(); })) { en.textContent = "Já existe um modo chamado “" + nome + "”. Escolha outro nome."; ok = false; }
    if (!ok) { n.setAttribute("aria-invalid", "true"); n.focus(); }
    var a = ed.atalho[S.plat].trim();
    if (a && !/(⌘|⌥|⌃|⇧|ctrl|alt|shift|cmd)/i.test(a)) { ea.textContent = "O atalho precisa de uma tecla modificadora (" + (S.plat === "mac" ? "⌘, ⌥, ⌃ ou ⇧" : "Ctrl, Alt ou Shift") + ")."; at.setAttribute("aria-invalid", "true"); if (ok) at.focus(); ok = false; }
    if (!ok) return;
    var final = VD.clone(ed); delete final._id; final.nome = nome;
    if (edNovo) { final.id = "m" + Date.now(); S.modos.push(final); }
    else { S.modos = S.modos.map(function (m) { return m.id === ed._id ? Object.assign(final, { id: m.id }) : m; }); }
    ed = null; VD.toast("Modo “" + nome + "” salvo."); VD.vai("modos");
  };
  VD.acoes["modo-padrao"] = function () {
    lerForm(); S.modos.forEach(function (m) { m.padrao = m.id === ed._id; }); ed.padrao = true; VD.toast("“" + ed.nome + "” agora é o modo padrão."); VD.render();
  };
  VD.acoes["modo-duplicar"] = function () {
    lerForm(); var c = VD.clone(ed); delete c._id; c.id = "m" + Date.now(); c.nome = (c.nome + " (cópia)").slice(0, 40); c.padrao = false; c.programas = { mac: [], win: [] }; c.atalho = { mac: "", win: "" }; c.gatilhos = [];
    S.modos.push(c); ed = null; VD.toast("Modo duplicado, sem programas, atalho nem gatilhos."); VD.vai("modos/" + c.id);
  };
  VD.acoes["modo-excluir"] = function (a) {
    if (a.getAttribute("aria-disabled") === "true") { VD.toast("O modo padrão não pode ser excluído.", "alerta"); return; }
    var nome = ed.nome;
    VD.confirma("Excluir o modo “" + esc(nome) + "”?", "Os programas associados voltam a usar o modo padrão. Itens do histórico mantêm o nome do modo.", "Excluir modo", function () {
      S.modos = S.modos.filter(function (m) { return m.id !== ed._id; }); ed = null; VD.toast("Modo excluído."); VD.vai("modos");
    });
  };

  /* ================= VOCABULÁRIO ================= */
  var buscaVoc = "";
  VD.rota("vocabulario", function () {
    return VD.cabecalho("Vocabulário", "Nomes, siglas e termos que a transcrição deve escrever certo. Vale para todos os modos.",
      '<span class="selo">' + S.vocab.length + " de 500 palavras</span>") +
      VD.estadoLista(function () {
        var lista = S.vocab.filter(function (p) { return p.toLowerCase().indexOf(buscaVoc.toLowerCase()) >= 0; });
        return '<div class="cartao"><form data-envia="voc-add" class="acoes" novalidate style="align-items:flex-start"><div class="campo" style="flex:1;min-width:220px;margin:0"><label for="v-nova">Adicionar palavra</label><input id="v-nova" type="text" placeholder="Ex.: Oncoclínica Vale do Sol" autocomplete="off" aria-describedby="v-erro"><span class="erro-campo" id="v-erro" role="alert"></span></div><button class="btn primario" style="margin-top:26px">Adicionar</button></form></div>' +
          '<div class="campo" style="margin:20px 0 12px"><label for="v-busca">Buscar no vocabulário</label><input id="v-busca" type="search" value="' + esc(buscaVoc) + '" data-entra="voc-busca" placeholder="Digite para filtrar"></div>' +
          (lista.length ? '<ul class="lista-chips" style="list-style:none;padding:0">' + lista.map(function (p) { return '<li class="chip"><span>' + esc(p) + '</span><button type="button" data-acao="voc-edit" data-p="' + esc(p) + '" aria-label="Editar ' + esc(p) + '" style="font-size:13px">✎</button><button type="button" data-acao="voc-rm" data-p="' + esc(p) + '" aria-label="Remover ' + esc(p) + '">×</button></li>'; }).join("") + "</ul>" : '<p class="suave" role="status">Nenhuma palavra encontrada para “' + esc(buscaVoc) + "”.</p>");
      }, function () {
        return VD.vazio("📖", "Seu vocabulário está vazio", "Cadastre nomes de clientes, siglas e termos técnicos para a transcrição acertar de primeira.", '<form data-envia="voc-add" class="acoes" style="justify-content:center"><label class="so-leitor" for="v-nova">Primeira palavra</label><input id="v-nova" type="text" style="max-width:260px" placeholder="Ex.: Going2"><button class="btn primario">Adicionar</button></form><span class="erro-campo" id="v-erro" role="alert"></span>');
      });
  });
  VD.acoes["voc-busca"] = function (t) { buscaVoc = t.value; VD.render(); var i = $("#v-busca"); i.focus(); i.setSelectionRange(i.value.length, i.value.length); };
  VD.acoes["voc-add"] = function () {
    var i = $("#v-nova"), v = i.value.trim(), e = $("#v-erro"); i.removeAttribute("aria-invalid");
    if (!v) { e.textContent = "Digite uma palavra ou nome."; i.setAttribute("aria-invalid", "true"); return; }
    if (S.vocab.some(function (p) { return p.toLowerCase() === v.toLowerCase(); })) { e.textContent = "“" + v + "” já está no vocabulário."; i.setAttribute("aria-invalid", "true"); return; }
    if (S.vocab.length >= 500) { e.textContent = "O vocabulário aceita até 500 palavras."; return; }
    S.vocab.unshift(v); if (S.estado === "vazio") S.estado = "normal"; VD.toast("“" + v + "” adicionada."); VD.render(); $("#v-nova").focus();
  };
  VD.acoes["voc-rm"] = function (a) { var p = a.getAttribute("data-p"); S.vocab = S.vocab.filter(function (x) { return x !== p; }); VD.toast("“" + p + "” removida."); VD.render(); };
  VD.acoes["voc-edit"] = function (a) {
    var p = a.getAttribute("data-p");
    VD.modal("Editar palavra", '<div class="campo"><label for="ve">Palavra</label><input id="ve" type="text" value="' + esc(p) + '"></div>', [{ rot: "Cancelar" }, { rot: "Salvar", cls: "primario", acao: function (r) { var v = r.querySelector("#ve").value.trim(); if (!v) return false; S.vocab = S.vocab.map(function (x) { return x === p ? v : x; }); VD.render(); } }]);
  };

  /* ================= SUBSTITUIÇÕES ================= */
  var subEd = null;
  VD.rota("substituicoes", function () {
    return VD.cabecalho("Substituições", "Troca uma palavra ou expressão por outra em todo ditado, antes do refinamento por IA. Não diferencia maiúsculas de minúsculas.") +
      VD.estadoLista(function () {
        return '<div class="cartao"><form data-envia="sub-add" novalidate><div class="duas-col"><div class="campo"><label for="s-de">Quando a transcrição disser</label><input id="s-de" type="text" value="' + esc(subEd ? subEd.de : "") + '" placeholder="Ex.: gou tu"></div><div class="campo"><label for="s-para">Escrever</label><input id="s-para" type="text" value="' + esc(subEd ? subEd.para : "") + '" placeholder="Ex.: Going2"></div></div><span class="erro-campo" id="s-erro" role="alert"></span><div class="acoes"><button class="btn primario">' + (subEd ? "Salvar substituição" : "Adicionar substituição") + "</button>" + (subEd ? '<button type="button" class="btn" data-acao="sub-cancela">Cancelar edição</button>' : "") + "</div></form></div>" +
          '<div class="tabela-wrap" style="margin-top:20px"><table class="tabela-resp"><thead><tr><th>Original</th><th>Substituto</th><th>Ativa</th><th><span class="so-leitor">Ações</span></th></tr></thead><tbody>' +
          S.subs.map(function (s) { return "<tr><td>" + esc(s.de) + "</td><td><b>" + esc(s.para) + '</b></td><td><label class="interruptor"><input type="checkbox" role="switch" data-muda="sub-liga" data-id="' + s.id + '"' + (s.ativa ? " checked" : "") + '><span></span><span class="so-leitor">Ativar “' + esc(s.de) + '”</span></label></td><td><div class="acoes"><button class="btn pequeno" data-acao="sub-edit" data-id="' + s.id + '">Editar</button><button class="btn pequeno perigo" data-acao="sub-rm" data-id="' + s.id + '">Remover</button></div></td></tr>'; }).join("") + "</tbody></table></div>";
      }, function () { return VD.vazio("🔁", "Nenhuma substituição", "Use para corrigir erros que se repetem, como siglas faladas por extenso.", '<form data-envia="sub-add" class="duas-col" style="max-width:520px;margin:auto;text-align:left"><div class="campo"><label for="s-de">Quando disser</label><input id="s-de" type="text"></div><div class="campo"><label for="s-para">Escrever</label><input id="s-para" type="text"></div><button class="btn primario">Adicionar</button><span class="erro-campo" id="s-erro" role="alert"></span></form>'); });
  });
  VD.acoes["sub-add"] = function () {
    var de = $("#s-de").value.trim(), para = $("#s-para").value.trim(), e = $("#s-erro");
    if (!de || !para) { e.textContent = "Preencha os dois campos."; return; }
    if (S.subs.some(function (s) { return s.de.toLowerCase() === de.toLowerCase() && (!subEd || s.id !== subEd.id); })) { e.textContent = "Já existe uma substituição para “" + de + "”."; return; }
    if (subEd) { S.subs = S.subs.map(function (s) { return s.id === subEd.id ? { id: s.id, de: de, para: para, ativa: s.ativa } : s; }); subEd = null; }
    else S.subs.unshift({ id: Date.now(), de: de, para: para, ativa: true });
    if (S.estado === "vazio") S.estado = "normal"; VD.toast("Substituição salva."); VD.render();
  };
  VD.acoes["sub-liga"] = function (t) { var id = +t.getAttribute("data-id"); S.subs.forEach(function (s) { if (s.id === id) s.ativa = t.checked; }); VD.toast(t.checked ? "Substituição ativada." : "Substituição desativada."); };
  VD.acoes["sub-edit"] = function (a) { var id = +a.getAttribute("data-id"); subEd = S.subs.filter(function (s) { return s.id === id; })[0]; VD.render(); $("#s-de").focus(); };
  VD.acoes["sub-cancela"] = function () { subEd = null; VD.render(); };
  VD.acoes["sub-rm"] = function (a) { var id = +a.getAttribute("data-id"); S.subs = S.subs.filter(function (s) { return s.id !== id; }); VD.toast("Substituição removida."); VD.render(); };

  /* ================= LIMPEZA ================= */
  function limpa(t) {
    var r = t;
    S.fill.slice().sort(function (a, b) { return b.length - a.length; }).forEach(function (w) { var x = w.replace(/…/g, "").trim(); if (x) r = r.replace(new RegExp("(^|[\\s,.;])" + x.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "(?=[\\s,.;]|$)", "gi"), "$1"); });
    r = r.replace(/\s+,/g, ",").replace(/\s{2,}/g, " ").trim().replace(/^[,.;\s]+/, "");
    if (r) { r = r.charAt(0).toUpperCase() + r.slice(1); if (!/[.!?…]$/.test(r)) r += "."; }
    return r;
  }
  var testeLimpeza = "é… então assim, tipo, preciso falar com o Pereira né sobre o contrato";
  VD.rota("limpeza", function () {
    return VD.cabecalho("Limpeza do texto", "Etapa sem IA: tira as muletas da fala e arruma a pontuação. Cada modo liga ou desliga a remoção e a formatação.") +
      VD.estadoLista(function () {
        return '<div class="cartao"><h2>Palavras de preenchimento</h2><p class="dica">Em português e inglês. Edite a lista como quiser.</p><ul class="lista-chips" style="list-style:none;padding:0">' + S.fill.map(function (w, i) { return '<li class="chip"><span>' + esc(w) + '</span><button type="button" data-acao="fill-rm" data-i="' + i + '" aria-label="Remover ' + esc(w) + '">×</button></li>'; }).join("") + '</ul><form data-envia="fill-add" class="acoes" style="margin-top:12px"><label class="so-leitor" for="f-nova">Nova palavra</label><input id="f-nova" type="text" style="max-width:240px" placeholder="Ex.: basicamente"><button class="btn">Adicionar</button></form></div>' +
          '<div class="cartao"><h2>Teste a limpeza</h2><p class="dica">Digite uma frase como se tivesse falado e veja o resultado.</p><div class="campo"><label for="f-teste">Texto falado</label><textarea id="f-teste" data-entra="fill-teste" style="min-height:80px">' + esc(testeLimpeza) + '</textarea></div><span class="rotulo">Depois da limpeza</span><div class="texto-bloco" id="f-saida" aria-live="polite">' + esc(limpa(testeLimpeza)) + "</div></div>";
      }, function () { return VD.vazio("🧹", "Lista vazia", "Sem palavras de preenchimento, a limpeza só arruma a pontuação.", '<form data-envia="fill-add" class="acoes" style="justify-content:center"><label class="so-leitor" for="f-nova">Nova palavra</label><input id="f-nova" type="text" style="max-width:240px"><button class="btn primario">Adicionar</button></form>'); });
  });
  VD.acoes["fill-teste"] = function (t) { testeLimpeza = t.value; $("#f-saida").textContent = limpa(t.value); };
  VD.acoes["fill-add"] = function () { var v = $("#f-nova").value.trim(); if (!v || S.fill.indexOf(v) >= 0) return; S.fill.push(v); if (S.estado === "vazio") S.estado = "normal"; VD.render(); $("#f-nova").focus(); };
  VD.acoes["fill-rm"] = function (a) { S.fill.splice(+a.getAttribute("data-i"), 1); VD.render(); };

  /* ================= HISTÓRICO ================= */
  var busca = "", filtroEstado = "todos";
  var estLabel = { colado: ["ok", "✓ Colado"], "nao-colado": ["alerta", "Não colado"], falhou: ["erro", "Falhou"], cancelado: ["", "Cancelado"] };
  VD.rota("historico", function () {
    var acoes = '<button class="btn" data-acao="hist-csv">Exportar CSV</button><a class="btn" href="#/config/dados">Retenção…</a>';
    return VD.cabecalho("Histórico", "Nenhum ditado se perde. Textos e áudios ficam só nesta máquina e saem conforme o prazo de retenção.", acoes) +
      VD.estadoLista(function () {
        var q = busca.toLowerCase();
        var l = S.hist.filter(function (i) { return (filtroEstado === "todos" || i.estado === filtroEstado) && (!q || (i.transcrito + " " + i.final).toLowerCase().indexOf(q) >= 0); });
        var h = '<div class="acoes" style="margin-bottom:16px;align-items:flex-end"><div class="campo" style="flex:1;min-width:220px;margin:0"><label for="h-busca">Buscar no texto transcrito e no final</label><input id="h-busca" type="search" value="' + esc(busca) + '" data-entra="hist-busca" placeholder="Ex.: contrato"></div><div class="campo" style="margin:0"><label for="h-est">Estado</label><select id="h-est" data-muda="hist-filtro"><option value="todos">Todos</option><option value="colado"' + (filtroEstado === "colado" ? " selected" : "") + '>Colado</option><option value="nao-colado"' + (filtroEstado === "nao-colado" ? " selected" : "") + '>Não colado</option><option value="falhou"' + (filtroEstado === "falhou" ? " selected" : "") + ">Falhou</option></select></div></div>";
        if (!l.length) return h + '<div class="vazio" role="status"><div class="grande" aria-hidden="true">🔎</div><h2>Nada encontrado</h2><p>Nenhum ditado corresponde a “' + esc(busca) + '”. Tente outra palavra ou limpe o filtro.</p><button class="btn" data-acao="hist-limpa">Limpar busca</button></div>';
        return h + '<div class="tabela-wrap"><table class="tabela-resp"><thead><tr><th>Data</th><th>Modo</th><th>Texto final</th><th>Estado</th></tr></thead><tbody>' + l.map(function (i) {
          var e = estLabel[i.estado];
          return "<tr><td>" + i.data + "<br><span class=\"suave pequeno\">" + i.hora + " · " + esc(i.programa) + "</span></td><td>" + esc(i.modo) + '</td><td><a class="linha" href="#/historico/' + i.id + '"><span class="corte">' + esc(i.final || "(sem texto — falhou)") + '</span></a></td><td><span class="selo ' + e[0] + '">' + e[1] + "</span></td></tr>";
        }).join("") + "</tbody></table></div>";
      }, function () { return VD.vazio("🕘", "Nenhum ditado ainda", "Os ditados concluídos aparecem aqui, com o texto antes e depois do refinamento.", '<a class="btn primario" href="#/ditado">Fazer um ditado de teste</a>'); });
  });
  VD.acoes["hist-busca"] = function (t) { busca = t.value; VD.render(); var i = $("#h-busca"); i.focus(); i.setSelectionRange(i.value.length, i.value.length); };
  VD.acoes["hist-filtro"] = function (t) { filtroEstado = t.value; VD.render(); };
  VD.acoes["hist-limpa"] = function () { busca = ""; filtroEstado = "todos"; VD.render(); };
  VD.acoes["hist-csv"] = function () {
    VD.modal("Exportar histórico em CSV", '<div class="duas-col"><div class="campo"><label for="c-de">De</label><input id="c-de" type="text" value="01/10/2026"></div><div class="campo"><label for="c-ate">Até</label><input id="c-ate" type="text" value="23/10/2026"></div></div><p class="dica">O arquivo (UTF-8) traz data e hora, modo, programa, texto transcrito, texto final, provedores, duração do áudio e estado. O áudio não vai no CSV.</p>',
      [{ rot: "Cancelar" }, { rot: "Exportar", cls: "primario", acao: function () { VD.toast("voice-desk-historico-2026-10-23.csv salvo em Downloads."); } }]);
  };
  VD.rota("historico/:id", function (id) {
    var i = S.hist.filter(function (x) { return String(x.id) === id; })[0];
    if (!i) return VD.cabecalho("Item não encontrado") + '<a href="#/historico">Voltar ao histórico</a>';
    var e = estLabel[i.estado];
    var h = '<p class="migalha"><a href="#/historico">← Histórico</a></p>' + VD.cabecalho("Ditado de " + i.data + ", " + i.hora, "", '<span class="selo ' + e[0] + '">' + e[1] + "</span>");
    if (i.aviso) h += '<div class="aviso ' + (i.estado === "falhou" ? "erro" : "alerta") + '" role="status"><span aria-hidden="true">' + (i.estado === "falhou" ? "⚠️" : "ℹ️") + '</span><div class="conteudo">' + esc(i.aviso) + "</div></div>";
    h += '<dl class="meta"><div><dt>Modo</dt><dd>' + esc(i.modo) + "</dd></div><div><dt>Programa em foco</dt><dd>" + esc(i.programa) + "</dd></div><div><dt>Duração do áudio</dt><dd>" + i.dur + "</dd></div><div><dt>Transcrição</dt><dd>" + esc(i.transcProv) + "</dd></div><div><dt>Tempo de transcrição</dt><dd>" + i.tTransc + "</dd></div><div><dt>Refinamento</dt><dd>" + esc(i.iaProv) + "</dd></div><div><dt>Tempo de refinamento</dt><dd>" + i.tIa + "</dd></div><div><dt>Colagem</dt><dd>" + e[1].replace("✓ ", "") + "</dd></div></dl>";
    h += '<div class="duas-col"><div class="cartao"><h2>Texto final</h2><div class="texto-bloco" style="margin:12px 0">' + esc(i.final || "—") + '</div><button class="btn" data-acao="hist-copia" data-id="' + i.id + '" data-q="final"' + (i.final ? "" : ' aria-disabled="true"') + '>Copiar texto final</button></div><div class="cartao"><h2>Texto transcrito</h2><div class="texto-bloco" style="margin:12px 0">' + esc(i.transcrito || "—") + '</div><button class="btn" data-acao="hist-copia" data-id="' + i.id + '" data-q="transcrito"' + (i.transcrito ? "" : ' aria-disabled="true"') + ">Copiar texto transcrito</button></div></div>";
    h += '<div class="cartao"><h2>Áudio</h2>' + (i.audio ? '<div class="player" style="margin:12px 0"><button class="btn pequeno" data-acao="hist-play" aria-label="Ouvir o áudio">▶ Ouvir</button><div class="onda" aria-hidden="true"></div><span class="mono">0:00 / ' + i.dur + '</span></div><button class="btn primario" data-acao="hist-redo" data-id="' + i.id + '">Transcrever de novo…</button>' : '<p class="suave">O áudio deste ditado foi apagado pela retenção (' + S.cfg.retAudio + ' dias). O texto continua aqui. <a href="#/config/dados">Mudar prazos</a></p>') + "</div>";
    h += '<div class="acoes" style="margin-top:20px"><button class="btn perigo" data-acao="hist-excluir" data-id="' + i.id + '">Excluir este item</button></div>';
    return h;
  });
  VD.acoes["hist-copia"] = function (a) { if (a.getAttribute("aria-disabled") === "true") return; VD.toast("Texto copiado para a área de transferência."); };
  VD.acoes["hist-play"] = function () { VD.toast("Reproduzindo o áudio (simulado)."); };
  VD.acoes["hist-redo"] = function (a) {
    VD.modal("Transcrever de novo", '<div class="campo"><label for="r-modo">Usar o modo</label><select id="r-modo">' + S.modos.map(function (m) { return "<option>" + esc(m.emoji + " " + m.nome) + "</option>"; }).join("") + '</select></div><p class="dica">O resultado vai para a área de transferência. Nada é colado em nenhum programa.</p>',
      [{ rot: "Cancelar" }, { rot: "Transcrever", cls: "primario", acao: function () { VD.toast("Transcrição pronta e copiada. Cole onde quiser (" + VD.mod() + "+V)."); } }]);
  };
  VD.acoes["hist-excluir"] = function (a) {
    var id = a.getAttribute("data-id");
    VD.confirma("Excluir este ditado?", "O texto e o áudio deste item serão apagados desta máquina. Não dá para desfazer.", "Excluir item", function () { S.hist = S.hist.filter(function (x) { return String(x.id) !== id; }); VD.toast("Item excluído."); VD.vai("historico"); });
  };
})();
