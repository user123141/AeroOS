// AeroOS System Controls v4.4.0 - brightness + volume.

function getBrightnessOverlay() {
  let o = document.getElementById('brightness-overlay');
  if (!o) {
    o = document.createElement('div');
    o.id = 'brightness-overlay';
    document.body.appendChild(o);
  }
  return o;
}

// Fallback dimming via CSS (visual only). Real brightness goes through IPC.
window.setBrightnessVisual = function (value) {
  const dim = (100 - value) / 250;
  const o = getBrightnessOverlay();
  o.style.opacity = String(dim);
};

window.setBrightness = function (value) {
  value = Math.max(0, Math.min(100, value));
  localStorage.setItem('aero-brightness', String(value));
  window.setBrightnessVisual(value);
};

window.getBrightness = function () {
  return parseInt(localStorage.getItem('aero-brightness') || '100', 10);
};

window.setVolume = function (value) {
  value = Math.max(0, Math.min(100, value));
  localStorage.setItem('aero-volume', String(value / 100));
};

window.getVolume = function () {
  return parseInt((parseFloat(localStorage.getItem('aero-volume') || '0.5')) * 100, 10);
};

window.addEventListener('load', () => {
  window.setBrightnessVisual(window.getBrightness());
});