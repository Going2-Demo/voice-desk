/* Voice Desk — núcleo do protótipo: estado, rotas, toasts, modais e barra do protótipo. */
(function () {
  var D = window.DADOS;
  var clone = function (o) { return JSON.parse(JSON.stringify(o)); };
  var qs = new URLSearchParams(location.search);

  var S = {
    plat: qs.get("plat") === "win" ? "win" : "mac",
    estado: ["vazio", "carregando", "erro"].indexOf(qs.get("estado")) >= 0 ? qs.get("estado") : "normal",
    tema: qs.get("tema") === "escuro" ? "escuro" : "claro",
    modos: clone(D.modos), vocab: D.vocabulario.slice(), subs: clone(D.substituicoes), fill: D.preenchimento.slice(),
    hist: clone(D.historico),
    cfg: { iniciarComSistema: true, sons: true, som: "Gota", pausarMidia: true, descartarSemFala: true, mic: "Padrão do sistema", micConectado: true,
           retAudio: "7", retHist: "90", atalhoGlobal: { mac: "⌃⌥Espaço", win: "Ctrl+Alt+Espaço" } },
    chaves: { groq: { salva: true, fim: "a9F2", estado: "válida" } },
    ollama: { url: "http://localhost:11434", ativo: true },
    modelo: { estado: "baixado", pct: 0 },
    sync: { ligada: true, estado: "Em dia" },
    perm: { mic: true, acess: true },
    ditado: { fase: "idle" }
  };
  var VD = { S: S, D: D, rotas: [], clone: clone };
  window.VD = VD;

  VD.esc = function (t) { return String(t == null ? "" : t).replace(/[&<>"']/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]; }); };
  var esc = VD.esc;
  VD.mod = function () { return S.plat === "mac" ? "⌘" : "Ctrl"; };
  VD.modoPorId = function (id) { return S.modos.filter(function (m) { return m.id === id; })[0]; };
  VD.atalhoGlobal = function () { return S.cfg.atalhoGlobal[S.plat]; };

  /* ---------- toasts e modais ---------- */
  VD.toast = function (msg, tipo) {
    var el = document.createElement("div"); el.className = "toast" + (tipo ? " " + tipo : ""); el.textContent = msg;
    document.getElementById("toasts").appendChild(el);
    setTimeout(function () { el.remove(); }, 5200);
  };
  var ultimoFoco = null;
  VD.modal = function (titulo, corpoHtml, botoes) {
    ultimoFoco = document.activeElement;
    var raiz = document.getElementById("modal-raiz");
    var bs = (botoes || [{ rot: "Fechar" }]).map(function (b, i) {
      return '<button type="button" class="btn ' + (b.cls || "") + '" data-modal-btn="' + i + '">' + esc(b.rot) + "</button>";
    }).join("");
    raiz.innerHTML = '<div class="fundo-modal"><div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-t"><h2 id="modal-t">' + esc(titulo) + "</h2><div>" + corpoHtml + '</div><div class="acoes">' + bs + "</div></div></div>";
    var primeiro = raiz.querySelector("input,select,textarea,button");
    if (primeiro) primeiro.focus();
    raiz.querySelectorAll("[data-modal-btn]").forEach(function (el) {
      el.addEventListener("click", function () {
        var b = (botoes || [{}])[+el.getAttribute("data-modal-btn")];
        var manter = b && b.acao ? b.acao(raiz) === false : false;
        if (!manter) VD.fechaModal();
      });
    });
  };
  VD.fechaModal = function () { document.getElementById("modal-raiz").innerHTML = ""; if (ultimoFoco && ultimoFoco.focus) ultimoFoco.focus(); };
  VD.confirma = function (titulo, texto, rotuloBotao, aoConfirmar) {
    VD.modal(titulo, '<p>' + texto + "</p>", [{ rot: "Cancelar" }, { rot: rotuloBotao, cls: "perigo cheio", acao: aoConfirmar }]);
  };
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && document.querySelector("#modal-raiz .modal")) VD.fechaModal();
  });

  /* ---------- peças reutilizáveis ---------- */
  VD.interruptor = function (id, ligado, rotulo, extra) {
    return '<label class="interruptor"><input type="checkbox" id="' + id + '" role="switch" ' + (ligado ? "checked " : "") + (extra || "") + '><span></span><span class="so-leitor">' + esc(rotulo) + "</span></label>";
  };
  VD.opcao = function (id, titulo, descricao, ligado, extra) {
    return '<div class="linha-opcao"><div class="texto"><b id="' + id + '-t">' + esc(titulo) + '</b><span class="dica">' + descricao + "</span></div>" + VD.interruptor(id, ligado, titulo, extra) + "</div>";
  };
  VD.vazio = function (emoji, titulo, texto, botao) {
    return '<div class="vazio"><div class="grande" aria-hidden="true">' + emoji + "</div><h2>" + esc(titulo) + "</h2><p>" + esc(texto) + "</p>" + (botao || "") + "</div>";
  };
  VD.carregando = function (n) {
    var h = '<div role="status" aria-label="Carregando"><span class="so-leitor">Carregando…</span>';
    for (var i = 0; i < (n || 4); i++) h += '<div class="esqueleto"></div>';
    return h + "</div>";
  };
  VD.erroBloco = function (msg) {
    return '<div class="aviso erro" role="alert"><span aria-hidden="true">⚠️</span><div class="conteudo"><b>Não foi possível carregar.</b><br>' + esc(msg) + '<div class="acoes"><button type="button" class="btn pequeno" data-acao="tentar-de-novo">Tentar de novo</button></div></div></div>';
  };
  VD.cabecalho = function (titulo, texto, acoes) {
    return '<div class="cabecalho"><div><h1>' + esc(titulo) + "</h1>" + (texto ? '<p class="suave">' + texto + "</p>" : "") + '</div><div class="acoes">' + (acoes || "") + "</div></div>";
  };
  VD.estadoLista = function (fnNormal, fnVazio, msgErro) {
    if (S.estado === "carregando") return VD.carregando(5);
    if (S.estado === "erro") return VD.erroBloco(msgErro || "Confira a conexão e tente de novo.");
    if (S.estado === "vazio") { var v = fnVazio(); return v === "" ? fnNormal() : v; }
    return fnNormal();
  };

  /* ---------- navegação ---------- */
  var MENU = [
    { grupo: "Ditado", itens: [
      { r: "ditado", ic: "🎙️", t: "Testar ditado" },
      { r: "modos", ic: "🧩", t: "Modos" },
      { r: "historico", ic: "🕘", t: "Histórico" } ] },
    { grupo: "Texto", itens: [
      { r: "vocabulario", ic: "📖", t: "Vocabulário" },
      { r: "substituicoes", ic: "🔁", t: "Substituições" },
      { r: "limpeza", ic: "🧹", t: "Limpeza do texto" } ] },
    { grupo: "Máquinas", itens: [
      { r: "sincronizacao", ic: "🔄", t: "Sincronização" } ] },
    { grupo: "App", itens: [
      { r: "config/geral", ic: "⚙️", t: "Configurações" },
      { r: "bandeja", ic: "🔔", t: "Ícone e menu" },
      { r: "primeiro-uso", ic: "👋", t: "Primeiro uso" },
      { r: "sobre", ic: "ℹ️", t: "Sobre" } ] }
  ];
  function rotaAtual() { return (location.hash || "#/ditado").replace(/^#\/?/, "") || "ditado"; }
  function desenhaLateral(r) {
    var h = '<div class="marca"><span aria-hidden="true">🎙️</span> Voice Desk</div>';
    MENU.forEach(function (g) {
      h += '<div class="grupo-titulo">' + g.grupo + "</div>";
      g.itens.forEach(function (i) {
        var ativo = r === i.r || r.split("/")[0] === i.r.split("/")[0];
        h += '<a class="item" href="#/' + i.r + '"' + (ativo ? ' aria-current="page"' : "") + '><span class="icone" aria-hidden="true">' + i.ic + "</span>" + i.t + "</a>";
      });
    });
    h += '<div class="rodape">Versão ' + D.versao + " · " + (S.plat === "mac" ? "macOS" : "Windows") + "</div>";
    document.getElementById("lateral").innerHTML = h;
  }
  VD.rota = function (padrao, fn) { VD.rotas.push({ padrao: padrao, fn: fn }); };
  VD.render = function () {
    document.documentElement.setAttribute("data-tema", S.tema);
    document.documentElement.setAttribute("data-plat", S.plat);
    var r = rotaAtual(), partes = r.split("/"), html = null;
    for (var i = 0; i < VD.rotas.length && html === null; i++) {
      var p = VD.rotas[i].padrao.split("/");
      if (p.length !== partes.length) continue;
      var args = [], ok = true;
      for (var j = 0; j < p.length; j++) { if (p[j].charAt(0) === ":") args.push(partes[j]); else if (p[j] !== partes[j]) { ok = false; break; } }
      if (ok) html = VD.rotas[i].fn.apply(null, args);
    }
    if (html === null) html = VD.cabecalho("Tela não encontrada") + '<p>Volte ao <a href="#/ditado">início</a>.</p>';
    desenhaLateral(r);
    document.getElementById("principal").innerHTML = html;
    document.querySelectorAll("[data-proto]").forEach(function (b) { b.setAttribute("aria-pressed", String(S[b.getAttribute("data-proto")] === b.getAttribute("data-v"))); });
    if (VD.aposRender) VD.aposRender(r);
  };
  VD.vai = function (r) { if (location.hash === "#/" + r) VD.render(); else location.hash = "#/" + r; };

  /* ---------- ações globais por delegação ---------- */
  VD.acoes = {};
  document.addEventListener("click", function (e) {
    var p = e.target.closest("[data-proto]");
    if (p) { S[p.getAttribute("data-proto")] = p.getAttribute("data-v"); VD.render(); return; }
    var a = e.target.closest("[data-acao]");
    if (a && VD.acoes[a.getAttribute("data-acao")]) { VD.acoes[a.getAttribute("data-acao")](a, e); }
  });
  VD.acoes["tentar-de-novo"] = function () { S.estado = "carregando"; VD.render(); setTimeout(function () { S.estado = "normal"; VD.render(); }, 900); };
  document.addEventListener("change", function (e) { var t = e.target.closest("[data-muda]"); if (t && VD.acoes[t.getAttribute("data-muda")]) VD.acoes[t.getAttribute("data-muda")](t, e); });
  document.addEventListener("submit", function (e) { var f = e.target.closest("[data-envia]"); if (f) { e.preventDefault(); if (VD.acoes[f.getAttribute("data-envia")]) VD.acoes[f.getAttribute("data-envia")](f, e); } });
  document.addEventListener("input", function (e) { var t = e.target.closest("[data-entra]"); if (t && VD.acoes[t.getAttribute("data-entra")]) VD.acoes[t.getAttribute("data-entra")](t, e); });
  window.addEventListener("hashchange", function () { VD.render(); var m = document.getElementById("principal"); if (m) m.focus(); });

  VD.iniciar = function () { VD.render(); };
})();
