// Multi-VM tabs — connected to backend via WS

let vms = [];
let activeVm = 'default';

window.renderVmStats = function (list) {
  vms = list || [];
  const grid = document.getElementById('vm-stats-grid');
  if (!grid) return;
  grid.innerHTML = '';
  if (vms.length === 0) {
    grid.innerHTML = '<div class="vm-stat-card"><div class="vm-stat-name">' +
      (window.t ? t('noSnapshots') : 'No VMs') + '</div></div>';
    return;
  }
  vms.forEach(vm => {
    const card = document.createElement('div');
    card.className = 'vm-stat-card';
    const name = document.createElement('div');
    name.className = 'vm-stat-name';
    name.textContent = vm.name;
    card.appendChild(name);
    const rows = [
      ['ID', vm.id],
      ['State', vm.running ? 'Running' : 'Stopped', vm.running ? 'on' : 'off'],
      ['Snapshots', String(vm.snapshots)],
      ['Dirty', String(vm.dirty_pages)],
    ];
    rows.forEach(([label, value, cls]) => {
      const r = document.createElement('div');
      r.className = 'vm-stat-row';
      const l = document.createElement('span'); l.textContent = label;
      const v = document.createElement('span'); v.textContent = value;
      if (cls) v.className = cls;
      r.appendChild(l); r.appendChild(v);
      card.appendChild(r);
    });
    grid.appendChild(card);
  });
};