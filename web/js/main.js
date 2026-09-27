let ws = null;
let sessionToken = '';
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
  ws = new WebSocket('ws://127.0.0.1:8081/?token=' + sessionToken);
  ws.onopen = () => send({ type: 'GetStatus' });
  ws.onmessage = e => handle(JSON.parse(e.data));
  ws.onclose = () => setTimeout(connectWS, 3000);
  ws.onerror = () => {};
}
function send(cmd) { if (ws && ws.readyState === 1) ws.send(JSON.stringify(cmd)); }
function handle(d) {
  switch (d.type) {
    case 'Status':
      document.getElementById('cpu-cores').textContent = d.cpu_cores;
      document.getElementById('ram-size').textContent = d.ram_mb + ' MB';
      document.getElementById('gpu-model').textContent = d.gpu;
      document.getElementById('vm-status').textContent = d.running ? 'Running' : 'Stopped';
      document.getElementById('status').textContent = 'System active';
      document.getElementById('cfg-cpu').textContent = d.cpu_cores;
      document.getElementById('cfg-ram').textContent = d.ram_mb;
      document.getElementById('cfg-gpu').textContent = d.gpu;
      break;
    case 'Stats':
      document.getElementById('snap-count').textContent = d.snapshots;
      document.getElementById('dirty-pages').textContent = d.dirty_pages;
      break;
    case 'TerminalOutput':
      if (window.term) window.term.write(d.data);
      break;
    case 'FileList':
      if (window.renderFiles) window.renderFiles(d.files);
      break;
  }
}
function initControls() {
  const w = document.getElementById('window-main');
  if (w) {
    w.querySelector('.close').onclick = () => w.style.display = 'none';
    w.querySelector('.minimize').onclick = () => {
      w.style.transform = 'translate(-50%,120%) scale(.9)';
      w.style.opacity = '0';
      setTimeout(() => { w.style.opacity = '1'; w.style.transform = 'translate(-50%,-50%)'; }, 300);
    };
    w.querySelector('.maximize').onclick = () => w.classList.toggle('maximized');
  }
  document.querySelector('.close-terminal')?.addEventListener('click', () => {
    document.getElementById('window-terminal').classList.add('hidden');
  });
  document.querySelector('.close-settings')?.addEventListener('click', () => {
    document.getElementById('window-settings').classList.add('hidden');
  });
  document.querySelector('.close-browser')?.addEventListener('click', () => {
    document.getElementById('window-browser').classList.add('hidden');
  });
  document.querySelector('.close-files')?.addEventListener('click', () => {
    document.getElementById('window-files').classList.add('hidden');
  });
}
function initDock() {
  document.querySelectorAll('.dock-item').forEach(i => {
    i.onclick = () => {
      const app = i.dataset.app;
      if (app === 'snapshots') document.getElementById('snapshots-panel').classList.toggle('hidden');
      else if (app === 'terminal') { document.getElementById('window-terminal').classList.remove('hidden'); if (window.initTerminal) window.initTerminal(); }
      else if (app === 'settings') document.getElementById('window-settings').classList.remove('hidden');
      else if (app === 'browser') { document.getElementById('window-browser').classList.remove('hidden'); if (window.initBrowser) window.initBrowser(); }
      else if (app === 'files') { document.getElementById('window-files').classList.remove('hidden'); if (window.initFiles) window.initFiles(); }
    };
  });
  document.getElementById('create-snapshot').onclick = () => {
    const n = prompt('Snapshot name:', 'snap_' + Date.now());
    if (n) send({ type: 'Snapshot', name: n });
  };
}