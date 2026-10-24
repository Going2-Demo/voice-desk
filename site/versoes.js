/* Monta a tabela de versões a partir de versoes.json (RF-27.3). Sem o script, vale a tabela escrita no HTML. */
(() => {
  var corpo = document.getElementById("versoes-corpo");
  if (!corpo || !window.fetch) return;
  function data(iso) {
    var p = iso.split("-");
    return p[2] + "/" + p[1] + "/" + p[0];
  }
  function texto(el, t) {
    el.textContent = t;
    return el;
  }
  fetch("versoes.json")
    .then((r) => (r.ok ? r.json() : Promise.reject()))
    .then((lista) => {
      if (!Array.isArray(lista) || !lista.length) return;
      corpo.textContent = "";
      lista.forEach((v) => {
        var tr = document.createElement("tr");
        var a = document.createElement("td");
        var b = document.createElement("b");
        a.appendChild(texto(b, v.versao));
        var d = texto(document.createElement("td"), data(v.data));
        var m = document.createElement("td");
        m.textContent = v.mudancas.join(" ");
        tr.appendChild(a);
        tr.appendChild(d);
        tr.appendChild(m);
        corpo.appendChild(tr);
      });
      var atual = lista[0];
      var t = document.getElementById("versao-atual");
      if (t) t.textContent = atual.versao;
      var dt = document.getElementById("versao-data");
      if (dt) dt.textContent = data(atual.data);
    })
    .catch(() => {});
})();
