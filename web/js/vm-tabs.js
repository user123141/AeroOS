// Multi-VM tabs.
let vms = [{ id: 'default', name: 'AeroOS Default' }];
let activeVm = 'default';

function renderVmTabs() {
  const el = document.getElementById('vm-tabs');
  if (!el) return;
  el.innerHTML = '';
  vms.forEach(vm => {
    const t = document.createElement('button');
    t.className = 'vm-tab' + (vm.id === activeVm ? ' active' : '');
    t.textContent = vm.name;
    t.onclick = () => { activeVm = vm.id; renderVmTabs(); };
    el.appendChild(t);
  });
  const add = document.createElement('button');
  add.className = 'vm-tab add';
  add.textContent = '+';
  add.onclick = () => {
    const name = prompt('VM name:');
    if (name) { vms.push({ id: name.toLowerCase(), name }); renderVmTabs(); }
  };
  el.appendChild(add);
}
window.addEventListener('load', renderVmTabs);