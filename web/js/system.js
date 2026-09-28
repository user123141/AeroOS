// AeroOS System Controls: brightness + volume

function getBrightnessOverlay() {
  let o = document.getElementById('brightness-overlay');
  if (!o) {
    o = document.createElement('div');
    o.id = 'brightness-overlay';
    document.body.appendChild(o);
  }
  return o;
}

window.setBrightness = function (value) {
  // value: 0..100 (100 = max brightness)
  const dim = (100 - value) / 200; // max dim 50%
  const o = getBrightnessOverlay();
  o.style.opacity = String(dim);
  localStorage.setItem('aero-brightness', String(value));
};

window.getBrightness = function () {
  return parseInt(localStorage.getItem('aero-brightness') || '100', 10);
};

window.setVolume = function (value) {
  localStorage.setItem('aero-volume', String(value / 100));
};

window.getVolume = function () {
  return parseInt((parseFloat(localStorage.getItem('aero-volume') || '0.5')) * 100, 10);
};

window.addEventListener('load', () => {
  setBrightness(getBrightness());
});