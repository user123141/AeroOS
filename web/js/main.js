// AeroOS main.js v4.3.0 — drag, focus, toast, WS

const WS = 'ws://127.0.0.1:8081';
let ws = null;
let sessionToken = '';
let zCounter = 100;

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
  ws.onopen = () => {
    send({ type: 'GetStatus' });
    send({ type: 'GetStats' });
    send({ type: 'ListVms' });
    if (window.showToast) showToast('AeroOS connected', 'success', 2000);
  };
  ws.onmessage = e => handle(JSON.parse(e.data));
  ws.onclose = () => {
    if (window.showToast) showToast('Disconnected. Reconnecting...', 'warning', 2000);
    setTimeout(connectWS, 3000);
  };
  ws.onerror = () => {};
}

function send(cmd) {
  if (ws && ws.readyState === 1) ws.send(JSON.stringify(cmd));
}

function handle(d) {
  switch (d.type) {
    case 'Status':
      setText('cpu-cores', d.cpu_cores);
      setText('ram-size', d.ram_mb + ' MB');
      setText('gpu-model', d.gpu);
      setText('vm-status', d.running ? t('running') : t('stopped'));
      break;
    case 'Stats':
      setText('snap-count', d.snapshots);
      setText('dirty-pages', d.dirty_pages);
      if (window.pushChart) window.pushChart(d.dirty_pages);
      break;
    case 'VmsList':
      if (window.renderVmStats) window.renderVmStats(d.vms);
      break;
    case 'Snapshots':
      if (window.updateTimeline) window.updateTimeline(d.list);
      break;
    case 'Serial':
    case 'TerminalOutput':
      if (window.term) window.term.write(d.data);
      break;
    case 'SearchResults':
      window.dispatchEvent(new CustomEvent('aero-search-results', { detail: d }));
      break;
    case 'Ok':
      if (window.showToast) showToast(d.message, 'success');
      break;
    case 'Error':
      if (window.showToast) showToast(d.message, 'error');
      break;
  }
}

function setText(id, v) {
  const el = document.getElementById(id);
  if (el) el.textContent = v;
}

function bringToFront(win) {
  win.style.zIndex = ++zCounter;
}

function makeDraggable(win) {
  const header = win.querySelector('.window-header');
  if (!header) return;

  let dragging = false;
  let startX = 0, startY = 0, startLeft = 0, startTop = 0;

  header.addEventListener('mousedown', (e) => {
    if (e.target.closest('.window-controls')) return;
    if (e.button !== 0) return;

    const rect = win.getBoundingClientRect();

    // Switch from transform-centered to absolute px
    win.style.transform = 'none';
    win.style.left = rect.left + 'px';
    win.style.top  = rect.top  + 'px';

    dragging = true;
    startX = e.clientX;
    startY = e.clientY;
    startLeft = rect.left;
    startTop  = rect.top;
    win.classList.add('dragging');
    bringToFront(win);
    e.preventDefault();
  });

  document.addEventListener('mousemove', (e) => {
    if (!dragging) return;
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;
    const w = win.offsetWidth, h = win.offsetHeight;
    const nx = Math.max(0, Math.min(window.innerWidth  - w, startLeft + dx));
    const ny = Math.max(0, Math.min(window.innerHeight - h, startTop  + dy));
    win.style.left = nx + 'px';
    win.style.top  = ny + 'px';
  });

  document.addEventListener('mouseup', () => {
    if (dragging) {
      dragging = false;
      win.classList.remove('dragging');
    }
  });

  win.addEventListener('mousedown', () => bringToFront(win));
}

function initWindowControls(win) {
  win.querySelector('.ctrl-close')?.addEventListener('click', () => {
    win.classList.add('hidden');
    if (window.playSound) playSound('click');
  });
  win.querySelector('.ctrl-min')?.addEventListener('click', () => {
    win.classList.add('minimized');
    if (window.playSound) playSound('click');
  });
  win.querySelector('.ctrl-max')?.addEventListener('click', () => {
    win.classList.toggle('maximized');
    if (window.playSound) playSound('click');
  });
  win.querySelector('.ctrl-theme')?.addEventListener('click', () => {
    if (window.toggleTheme) window.toggleTheme();
  });
}

function initDock() {
  document.querySelectorAll('.dock-item').forEach(item => {
    item.addEventListener('click', () => {
      const app = item.dataset.app;
      const win = document.getElementById('window-' + app);
      if (!win) return;
      win.classList.remove('hidden');
      win.classList.remove('minimized');
      // Reset position to centered if it was moved
      if (win.style.left) {
        win.style.left = '';
        win.style.top = '';
        win.style.transform = '';
      }
      bringToFront(win);
      if (window.playSound) playSound('click');
      if (app === 'terminal' && window.initTerminal) window.initTerminal();
      if (app === 'timeline') send({ type: 'ListSnapshots' });
      if (app === 'dashboard') send({ type: 'ListVms' });
    });
  });
}

document.addEventListener('DOMContentLoaded', async () => {
  await fetchToken();
  connectWS();
  document.querySelectorAll('.window').forEach(win => {
    makeDraggable(win);
    initWindowControls(win);
  });
  initDock();
  setInterval(() => send({ type: 'GetStats' }), 2000);
  setInterval(() => send({ type: 'ListVms' }), 3000);
});

window.send = send;
window.bringToFront = bringToFront;