// AeroOS main.js — окна, drag, focus, WebSocket

const WS = 'ws://127.0.0.1:8081';
let ws = null;
let sessionToken = '';
let zCounter = 100;

// === WebSocket ===
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
  };
  ws.onmessage = e => handle(JSON.parse(e.data));
  ws.onclose = () => setTimeout(connectWS, 3000);
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
      if (window.term) window.term.write(d.data);
      break;
    case 'TerminalOutput':
      if (window.term) window.term.write(d.data);
      break;
  }
}

function setText(id, v) {
  const el = document.getElementById(id);
  if (el) el.textContent = v;
}

// === Drag & Drop окон ===
function bringToFront(win) {
  win.style.zIndex = ++zCounter;
}

function makeDraggable(win) {
  const header = win.querySelector('.window-header');
  if (!header) return;

  let dragging = false;
  let offsetX = 0, offsetY = 0;

  header.style.cursor = 'grab';

  header.addEventListener('mousedown', (e) => {
    if (e.target.closest('.window-controls')) return;
    if (e.button !== 0) return;

    dragging = true;
    header.style.cursor = 'grabbing';

    const rect = win.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;

    // Переводим в absolute позиционирование
    win.style.transform = 'none';
    win.style.left = rect.left + 'px';
    win.style.top = rect.top + 'px';
    win.style.right = 'auto';
    win.style.bottom = 'auto';
    win.style.margin = '0';

    bringToFront(win);
    e.preventDefault();
  });

  document.addEventListener('mousemove', (e) => {
    if (!dragging) return;
    const x = Math.max(0, Math.min(window.innerWidth - win.offsetWidth, e.clientX - offsetX));
    const y = Math.max(0, Math.min(window.innerHeight - win.offsetHeight, e.clientY - offsetY));
    win.style.left = x + 'px';
    win.style.top = y + 'px';
  });

  document.addEventListener('mouseup', () => {
    if (dragging) {
      dragging = false;
      header.style.cursor = 'grab';
    }
  });

  // Фокус при клике
  win.addEventListener('mousedown', () => bringToFront(win));
}

// === Window controls (close/min/max) ===
function initWindowControls(win) {
  win.querySelector('.ctrl-close')?.addEventListener('click', () => {
    win.classList.add('hidden');
  });

  win.querySelector('.ctrl-min')?.addEventListener('click', () => {
    win.classList.add('minimized');
    setTimeout(() => {
      win.classList.remove('minimized');
    }, 250);
  });

  win.querySelector('.ctrl-max')?.addEventListener('click', () => {
    win.classList.toggle('maximized');
  });

  win.querySelector('.ctrl-theme')?.addEventListener('click', () => {
    if (window.toggleTheme) window.toggleTheme();
  });
}

// === Dock ===
function initDock() {
  document.querySelectorAll('.dock-item').forEach(item => {
    item.addEventListener('click', () => {
      const app = item.dataset.app;
      const win = document.getElementById('window-' + app);
      if (!win) return;

      win.classList.remove('hidden');
      win.classList.remove('minimized');
      bringToFront(win);

      if (app === 'terminal' && window.initTerminal) window.initTerminal();
      if (app === 'timeline') send({ type: 'ListSnapshots' });
      if (app === 'dashboard') send({ type: 'ListVms' });
    });
  });
}

// === Init ===
document.addEventListener('DOMContentLoaded', async () => {
  await fetchToken();
  connectWS();

  // Все окна — draggable
  document.querySelectorAll('.window').forEach(win => {
    makeDraggable(win);
    initWindowControls(win);
  });

  initDock();

  // Periodic stats
  setInterval(() => send({ type: 'GetStats' }), 2000);
  setInterval(() => send({ type: 'ListVms' }), 3000);
});

// Re-apply translations when language changes
window.addEventListener('aero-lang-changed', () => {
  const status = document.getElementById('vm-status');
  if (status && status.textContent) {
    const running = status.textContent === 'Running' || status.textContent === 'Работает';
    status.textContent = running ? t('running') : t('stopped');
  }
});

// Expose
window.send = send;
window.bringToFront = bringToFront;