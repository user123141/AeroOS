// AeroOS Toast Notifications + Sound (Web Audio API)

let audioCtx = null;

function ensureAudioCtx() {
  if (!audioCtx) {
    try { audioCtx = new (window.AudioContext || window.webkitAudioContext)(); }
    catch (e) { console.warn('AudioContext not available', e); }
  }
  if (audioCtx && audioCtx.state === 'suspended') audioCtx.resume();
  return audioCtx;
}

function playSound(type, volume = 0.15) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;

  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.connect(gain);
  gain.connect(ctx.destination);
  gain.gain.value = volume;

  const t = ctx.currentTime;
  switch (type) {
    case 'success':
      osc.frequency.setValueAtTime(523.25, t);
      osc.frequency.setValueAtTime(783.99, t + 0.08);
      break;
    case 'error':
      osc.frequency.setValueAtTime(220, t);
      osc.frequency.setValueAtTime(180, t + 0.1);
      break;
    case 'warning':
      osc.frequency.setValueAtTime(440, t);
      break;
    case 'click':
      osc.frequency.setValueAtTime(1200, t);
      gain.gain.value = 0.05;
      break;
    default:
      osc.frequency.setValueAtTime(800, t);
  }
  osc.type = 'sine';
  osc.start(t);
  osc.stop(t + 0.15);
}

function showToast(message, type = 'info', duration = 3000) {
  let container = document.getElementById('toast-container');
  if (!container) {
    container = document.createElement('div');
    container.id = 'toast-container';
    document.body.appendChild(container);
  }

  const toast = document.createElement('div');
  toast.className = 'toast toast-' + type;
  const icon = document.createElement('span');
  icon.className = 'toast-icon';
  icon.textContent = type === 'success' ? '✓' : type === 'error' ? '✕' : type === 'warning' ? '!' : 'i';
  const msg = document.createElement('span');
  msg.className = 'toast-message';
  msg.textContent = message;
  toast.appendChild(icon);
  toast.appendChild(msg);
  container.appendChild(toast);

  // Click-to-close
  toast.style.cursor = 'pointer';
  toast.onclick = () => {
    toast.classList.add('toast-out');
    setTimeout(() => toast.remove(), 300);
  };

  const vol = parseFloat(localStorage.getItem('aero-volume') || '0.5');
  playSound(type, 0.15 * vol);

  setTimeout(() => {
    toast.classList.add('toast-out');
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

window.showToast = showToast;
window.playSound = playSound;