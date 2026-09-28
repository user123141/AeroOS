// AeroOS Clock

function updateClock() {
  const el = document.getElementById('dash-clock');
  if (!el) return;
  const now = new Date();
  const hh = String(now.getHours()).padStart(2, '0');
  const mm = String(now.getMinutes()).padStart(2, '0');
  el.textContent = hh + ':' + mm;
  el.title = now.toLocaleDateString('ru-RU', {
    weekday: 'long', year: 'numeric', month: 'long', day: 'numeric'
  });
}

window.addEventListener('load', () => {
  updateClock();
  setInterval(updateClock, 1000);
});