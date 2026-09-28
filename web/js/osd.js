// AeroOS OSD - volume/brightness overlay (Apple-style)

const OSD_ICONS = {
  volume: {
    low:  '<svg viewBox="0 0 24 24"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/></svg>',
    mid:  '<svg viewBox="0 0 24 24"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>',
    high: '<svg viewBox="0 0 24 24"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>',
    muted:'<svg viewBox="0 0 24 24"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>'
  },
  brightness: '<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>'
};

let osdEl = null;
let osdHideTimer = null;

function ensureOsd() {
  if (osdEl) return osdEl;
  osdEl = document.createElement('div');
  osdEl.className = 'osd';
  osdEl.innerHTML = [
    '<div class="osd-icon" id="osd-icon"></div>',
    '<div class="osd-bar"><div class="osd-fill" id="osd-fill"></div></div>',
    '<div class="osd-value" id="osd-value">0%</div>'
  ].join('');
  document.body.appendChild(osdEl);
  return osdEl;
}

function showOsd(kind, level) {
  const el = ensureOsd();
  const iconEl = el.querySelector('#osd-icon');
  const fillEl = el.querySelector('#osd-fill');
  const valueEl = el.querySelector('#osd-value');

  level = Math.max(0, Math.min(100, level));
  fillEl.style.width = level + '%';
  valueEl.textContent = level + '%';

  if (kind === 'volume') {
    el.classList.remove('muted');
    if (level === 0) {
      iconEl.innerHTML = OSD_ICONS.volume.muted;
      el.classList.add('muted');
    } else if (level < 33) {
      iconEl.innerHTML = OSD_ICONS.volume.low;
    } else if (level < 66) {
      iconEl.innerHTML = OSD_ICONS.volume.mid;
    } else {
      iconEl.innerHTML = OSD_ICONS.volume.high;
    }
  } else if (kind === 'brightness') {
    el.classList.remove('muted');
    iconEl.innerHTML = OSD_ICONS.brightness;
  }

  el.classList.add('on');
  if (osdHideTimer) clearTimeout(osdHideTimer);
  osdHideTimer = setTimeout(() => el.classList.remove('on'), 1500);
}

window.showOsd = showOsd;