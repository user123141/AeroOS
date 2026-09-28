// AeroOS Top Menu Bar

function updateTopClock() {
  const el = document.getElementById('topbar-clock');
  if (!el) return;
  const now = new Date();
  const days = ['вс','пн','вт','ср','чт','пт','сб'];
  const hh = String(now.getHours()).padStart(2, '0');
  const mm = String(now.getMinutes()).padStart(2, '0');
  el.textContent = days[now.getDay()] + ' ' + hh + ':' + mm;
}

function openPopover(id) {
  document.querySelectorAll('.popover').forEach(p => {
    if (p.id !== id) p.classList.remove('on');
  });
  const p = document.getElementById(id);
  if (p) p.classList.toggle('on');
}

function closeAllPopovers() {
  document.querySelectorAll('.popover').forEach(p => p.classList.remove('on'));
}

window.addEventListener('load', () => {
  updateTopClock();
  setInterval(updateTopClock, 1000);

  document.getElementById('topbar-search')?.addEventListener('click', (e) => {
    e.stopPropagation();
    if (window.openSearch) window.openSearch();
  });

  document.getElementById('topbar-sound')?.addEventListener('click', (e) => {
    e.stopPropagation();
    openPopover('pop-sound');
  });

  document.getElementById('topbar-brightness')?.addEventListener('click', (e) => {
    e.stopPropagation();
    openPopover('pop-brightness');
  });

  document.getElementById('topbar-dnd')?.addEventListener('click', (e) => {
    e.stopPropagation();
    const on = !window.getDnd();
    window.setDnd(on);
    window.showToast(on ? 'Do Not Disturb ON' : 'Do Not Disturb OFF', 'info', 1500);
  });

  document.getElementById('topbar-notif')?.addEventListener('click', (e) => {
    e.stopPropagation();
    window.showToast('Notifications: ' + document.querySelectorAll('.toast').length, 'info', 2000);
  });

  // Clock -> calendar
  document.getElementById('topbar-clock')?.addEventListener('click', (e) => {
    e.stopPropagation();
    if (window.openCalendar) window.openCalendar();
  });
  document.getElementById('topbar-clock')?.style.setProperty('cursor', 'pointer');

  document.addEventListener('click', closeAllPopovers);

  // Sound slider in popover
  const soundSlider = document.getElementById('pop-sound-slider');
  const soundValue = document.getElementById('pop-sound-value');
  if (soundSlider) {
    soundSlider.value = window.getVolume();
    if (soundValue) soundValue.textContent = soundSlider.value + '%';
    soundSlider.oninput = () => {
      const v = parseInt(soundSlider.value, 10);
      window.setVolume(v);
      if (soundValue) soundValue.textContent = v + '%';
      if (window.showOsd) window.showOsd('volume', v);
    };
    soundSlider.onchange = () => {
      const v = parseInt(soundSlider.value, 10);
      if (window.send) window.send({ type: 'SetVolume', level: v });
    };
  }

  // Brightness slider in popover
  const brSlider = document.getElementById('pop-brightness-slider');
  const brValue = document.getElementById('pop-brightness-value');
  if (brSlider) {
    brSlider.value = window.getBrightness();
    if (brValue) brValue.textContent = brSlider.value + '%';
    brSlider.oninput = () => {
      const v = parseInt(brSlider.value, 10);
      window.setBrightness(v);
      if (brValue) brValue.textContent = v + '%';
      if (window.showOsd) window.showOsd('brightness', v);
    };
    brSlider.onchange = () => {
      const v = parseInt(brSlider.value, 10);
      if (window.send) window.send({ type: 'SetBrightness', level: v });
    };
  }
});