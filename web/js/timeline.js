let snapshots = [];

window.updateTimeline = function (list) {
  snapshots = list.slice().sort();
  const sl = document.getElementById('timeline-slider');
  if (!sl) return;
  sl.max = Math.max(0, snapshots.length - 1);
  sl.value = snapshots.length - 1;
  updateLabels();
  sl.oninput = updateLabels;
};

function updateLabels() {
  const sl = document.getElementById('timeline-slider');
  const sel = document.getElementById('timeline-selected');
  const first = document.getElementById('timeline-first');
  const last = document.getElementById('timeline-last');
  if (!snapshots.length) {
    if (first) first.textContent = '—';
    if (sel) sel.textContent = 'no snapshots';
    if (last) last.textContent = '—';
    return;
  }
  if (first) first.textContent = snapshots[0];
  if (last) last.textContent = snapshots[snapshots.length - 1];
  if (sel) sel.textContent = snapshots[parseInt(sl.value, 10)] || '—';
}

window.addEventListener('load', () => {
  const btn = document.getElementById('timeline-restore');
  if (btn) {
    btn.onclick = () => {
      const sl = document.getElementById('timeline-slider');
      const name = snapshots[parseInt(sl.value, 10)];
      if (name) {
        send({ type: 'Restore', name });
        if (window.term) window.term.writeln(`\r\n[Time Travel] Restoring ${name}...`);
      }
    };
  }
});