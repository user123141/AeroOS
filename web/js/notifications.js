// AeroOS Notifications v4.7.0 - Crystal

let audioCtx = null;
const MAX_VISIBLE = 3;
let dndEnabled = localStorage.getItem('aero-dnd') === '1';

const ICONS = {
  info:    '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
  success: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>',
  warning: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>',
  error:   '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>'
};

const TITLES = {
  info: 'Информация',
  success: 'Готово',
  warning: 'Внимание',
  error: 'Ошибка'
};

function ensureAudioCtx() {
  if (!audioCtx) {
    try { audioCtx = new (window.AudioContext || window.webkitAudioContext)(); }
    catch (e) {}
  }
  if (audioCtx && audioCtx.state === 'suspended') audioCtx.resume();
  return audioCtx;
}

function playSound(type, volume) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  volume = (typeof volume === 'number') ? volume : 0.12;

  const master = ctx.createGain();
  master.connect(ctx.destination);
  master.gain.value = volume;

  const t = ctx.currentTime;
  const notes = {
    success: [523.25, 783.99],
    error:   [246.94, 196.00],
    warning: [392.00, 493.88],
    info:    [659.25, 987.77],
    click:   [1046.50]
  }[type] || [523.25];

  notes.forEach((freq, i) => {
    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(freq, t + i * 0.06);
    const g = ctx.createGain();
    g.gain.setValueAtTime(0, t + i * 0.06);
    g.gain.linearRampToValueAtTime(0.8 / notes.length, t + i * 0.06 + 0.01);
    g.gain.exponentialRampToValueAtTime(0.001, t + i * 0.06 + 0.35);
    osc.connect(g);
    g.connect(master);
    osc.start(t + i * 0.06);
    osc.stop(t + i * 0.06 + 0.4);
  });
}

function ensureContainer() {
  let c = document.getElementById('toast-container');
  if (!c) {
    c = document.createElement('div');
    c.id = 'toast-container';
    document.body.appendChild(c);
  }
  return c;
}

function trimQueue() {
  const c = ensureContainer();
  const items = c.querySelectorAll('.toast:not(.toast-out)');
  if (items.length > MAX_VISIBLE) {
    for (let i = 0; i < items.length - MAX_VISIBLE; i++) {
      const old = items[i];
      old.classList.add('toast-out');
      setTimeout(() => old.remove(), 400);
    }
  }
}

function showToast(message, type, duration, title) {
  type = type || 'info';
  duration = (typeof duration === 'number') ? duration : 4000;

  if (dndEnabled && type !== 'error') return;

  const c = ensureContainer();
  const toast = document.createElement('div');
  toast.className = 'toast toast-' + type;

  const iconWrap = document.createElement('div');
  iconWrap.className = 'toast-icon-wrap';
  iconWrap.innerHTML = ICONS[type] || ICONS.info;

  const content = document.createElement('div');
  content.className = 'toast-content';

  const titleEl = document.createElement('div');
  titleEl.className = 'toast-title';
  titleEl.textContent = title || TITLES[type] || '';
  content.appendChild(titleEl);

  const desc = document.createElement('div');
  desc.className = 'toast-desc';
  desc.textContent = message;
  content.appendChild(desc);

  const closeBtn = document.createElement('button');
  closeBtn.className = 'toast-close';
  closeBtn.setAttribute('aria-label', 'Close');
  closeBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>';
  closeBtn.onclick = (e) => {
    e.stopPropagation();
    dismissToast(toast);
  };

  toast.appendChild(iconWrap);
  toast.appendChild(content);
  toast.appendChild(closeBtn);
  c.appendChild(toast);

  const vol = parseFloat(localStorage.getItem('aero-volume') || '0.5');
  if (vol > 0) playSound(type, 0.12 * vol);

  if (duration > 0) {
    setTimeout(() => dismissToast(toast), duration);
  }

  trimQueue();
  updateNotifBadge();
}

function dismissToast(toast) {
  if (!toast.parentNode) return;
  toast.classList.add('toast-out');
  setTimeout(() => {
    toast.remove();
    updateNotifBadge();
  }, 400);
}

function updateNotifBadge() {
  const badge = document.getElementById('notif-badge');
  if (!badge) return;
  const c = document.getElementById('toast-container');
  const count = c ? c.querySelectorAll('.toast:not(.toast-out)').length : 0;
  badge.classList.toggle('on', count > 0);
}

function setDnd(on) {
  dndEnabled = !!on;
  localStorage.setItem('aero-dnd', dndEnabled ? '1' : '0');
  const item = document.getElementById('topbar-dnd');
  if (item) item.classList.toggle('dnd-on', dndEnabled);
}

function getDnd() { return dndEnabled; }

window.showToast = showToast;
window.playSound = playSound;
window.setDnd = setDnd;
window.getDnd = getDnd;