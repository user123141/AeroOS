// AeroOS Spotlight Search (Ctrl+K / Cmd+K)

const SEARCH_ACTIONS = {
  'open-dashboard': () => openWindowById('dashboard'),
  'open-terminal':  () => openWindowById('terminal'),
  'open-timeline':  () => openWindowById('timeline'),
  'open-settings':  () => openWindowById('settings'),
  'toggle-theme':   () => window.toggleTheme && window.toggleTheme(),
  'toggle-dnd':     () => window.setDnd && window.setDnd(!window.getDnd()),
  'create-snapshot':() => window.send && window.send({ type: 'Snapshot', name: 'snap_' + Date.now() }),
  'stop-vm':        () => window.send && window.send({ type: 'Stop' }),
};

let searchItems = [];
let searchIndex = 0;

function openWindowById(id) {
  const win = document.getElementById('window-' + id);
  if (!win) return;
  win.classList.remove('hidden');
  win.classList.remove('minimized');
  if (win.style.left) {
    win.style.left = '';
    win.style.top = '';
    win.style.transform = '';
  }
  if (window.bringToFront) window.bringToFront(win);
  if (id === 'terminal' && window.initTerminal) window.initTerminal();
  if (id === 'timeline' && window.send) window.send({ type: 'ListSnapshots' });
}

function getSearchOverlay() {
  let o = document.getElementById('search-overlay');
  if (!o) {
    o = document.createElement('div');
    o.id = 'search-overlay';
    o.className = 'search-overlay';
    o.innerHTML = [
      '<div class="search-box">',
      '  <input type="text" class="search-input" id="search-input" placeholder="Search AeroOS..." autocomplete="off">',
      '  <div class="search-results" id="search-results"></div>',
      '  <div class="search-hint">',
      '    <span><kbd>&uarr;</kbd><kbd>&darr;</kbd> Navigate</span>',
      '    <span><kbd>Enter</kbd> Open</span>',
      '    <span><kbd>Esc</kbd> Close</span>',
      '  </div>',
      '</div>'
    ].join('\n');
    document.body.appendChild(o);

    const input = o.querySelector('#search-input');
    input.addEventListener('input', () => doSearch(input.value));
    input.addEventListener('keydown', handleSearchKey);
    o.addEventListener('click', (e) => { if (e.target === o) closeSearch(); });
  }
  return o;
}

function openSearch() {
  const o = getSearchOverlay();
  o.classList.add('on');
  const input = o.querySelector('#search-input');
  input.value = '';
  input.focus();
  doSearch('');
}

function closeSearch() {
  const o = document.getElementById('search-overlay');
  if (o) o.classList.remove('on');
}

function doSearch(query) {
  if (!window.send) return;
  window.send({ type: 'Search', query });
}

function renderSearchResults(items) {
  searchItems = items || [];
  searchIndex = 0;
  const box = document.getElementById('search-results');
  if (!box) return;
  box.innerHTML = '';
  if (items.length === 0) {
    box.innerHTML = '<div class="search-empty">No results</div>';
    return;
  }
  items.forEach((it, i) => {
    const div = document.createElement('div');
    div.className = 'search-item' + (i === 0 ? ' selected' : '');
    div.dataset.id = it.id;
    div.innerHTML =
      '<div><div class="search-item-title">' + escapeHtml(it.title) + '</div>' +
      '<div class="search-item-sub">' + escapeHtml(it.subtitle) + '</div></div>' +
      '<div class="search-item-cat">' + escapeHtml(it.category) + '</div>';
    div.addEventListener('click', () => activateSearchItem(i));
    box.appendChild(div);
  });
}

function activateSearchItem(i) {
  const it = searchItems[i];
  if (!it) return;
  closeSearch();
  // Snapshot restore
  if (it.id.startsWith('snap-')) {
    const name = it.id.substring(5);
    if (window.send) window.send({ type: 'Restore', name });
    if (window.showToast) window.showToast('Restoring snapshot: ' + name, 'info');
    return;
  }
  const action = SEARCH_ACTIONS[it.id];
  if (action) action();
}

function handleSearchKey(e) {
  if (e.key === 'Escape') { closeSearch(); return; }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    moveSelection(1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    moveSelection(-1);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    activateSearchItem(searchIndex);
  }
}

function moveSelection(dir) {
  const box = document.getElementById('search-results');
  if (!box) return;
  const items = box.querySelectorAll('.search-item');
  if (items.length === 0) return;
  items[searchIndex]?.classList.remove('selected');
  searchIndex = (searchIndex + dir + items.length) % items.length;
  items[searchIndex].classList.add('selected');
  items[searchIndex].scrollIntoView({ block: 'nearest' });
}

function escapeHtml(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

// Global keybinding: Ctrl+K or Cmd+K
window.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    openSearch();
  }
});

window.addEventListener('load', () => {
  // Hook: after WS response
  window.addEventListener('aero-search-results', (ev) => {
    renderSearchResults(ev.detail.items);
  });
});

window.openSearch = openSearch;
window.closeSearch = closeSearch;