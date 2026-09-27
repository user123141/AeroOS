const WS = 'ws://127.0.0.1:8081';
let ws = null;
let sessionToken = '';
let status = {};

document.addEventListener('DOMContentLoaded', async () => {
  await fetchToken();
  connectWS();
  initControls();
  initDock();
  setInterval(() => send({ type: 'GetStats' }), 2000);
});

async function fetchToken() {
  try {
    const r = await fetch('/api/token');
    const j = await r.json();
    sessionToken = j.token;
  } catch (e) { console.error('Token fetch failed:', e); }
}

function connectWS() {
  if (!sessionToken) { setTimeout(connectWS, 500); return; }
  ws = new WebSocket(WS + '/?token=' + sessionToken);
  ws.onopen = () => { send({ type: 'GetStatus' }); send({ type: 'GetStats' }); };
  ws.onmessage = e => handle(JSON.parse(e.data));
  ws.onclose = () => setTimeout(connectWS, 3000);
  ws.onerror = () => {};
}

function send(cmd) { if (ws && ws.readyState === 1) ws.send(JSON.stringify(cmd)); }

function handle(d) {
  switch (d.type) {
    case 'Status':
      status = d;
      setText('cpu-cores', d.cpu_cores);
      setText('ram-size', d.ram_mb + ' MB');
      setText('gpu-model', d.gpu);
      setText('vm-status', d.running ? 'Running' : 'Stopped');
      break;
    case 'Stats':
      setText('snap-count', d.snapshots);
      setText('dirty-pages', d.dirty_pages);
      if (window.pushChart) window.pushChart(d.dirty_pages);
      break;
    case 'Snapshots':
      if (window.updateTimeline) window.updateTimeline(d.list);
      break;
    case 'TerminalOutput':
      if (window.term) window.term.write(d.data);
      break;
  }
}

function setText(id, v) { const el = document.getElementById(id); if (el) el.textContent = v; }

function initControls() {
  document.querySelectorAll('.close').forEach(b => b.onclick = e => e.target.closest('.window').style.display = 'none');
  document.querySelector('.close-terminal')?.addEventListener('click', () => document.getElementById('window-terminal').classList.add('hidden'));
  document.querySelector('.close-timeline')?.addEventListener('click', () => document.getElementById('window-timeline').classList.add('hidden'));
  document.querySelector('.close-settings')?.addEventListener('click', () => document.getElementById('window-settings').classList.add('hidden'));
  document.querySelector('.theme-toggle')?.addEventListener('click', () => window.toggleTheme && window.toggleTheme());
}

function initDock() {
  document.querySelectorAll('.dock-item').forEach(i => {
    i.onclick = () => {
      const app = i.dataset.app;
      document.querySelectorAll('.window').forEach(w => {
        if (w.id !== 'window-dashboard') w.classList.add('hidden');
      });
      if (app === 'terminal') { document.getElementById('window-terminal').classList.remove('hidden'); if (window.initTerminal) window.initTerminal(); }
      else if (app === 'timeline') { document.getElementById('window-timeline').classList.remove('hidden'); send({ type: 'ListSnapshots' }); }
      else if (app === 'settings') { document.getElementById('window-settings').classList.remove('hidden'); }
      else { document.getElementById('window-dashboard').style.display = 'flex'; }
    };
  });
}