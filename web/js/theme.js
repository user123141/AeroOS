const THEMES = ['dark', 'light', 'aero'];
let themeIdx = 0;

function applyTheme(name) {
  document.documentElement.setAttribute('data-theme', name);
  localStorage.setItem('aero-theme', name);
}

window.toggleTheme = function () {
  themeIdx = (themeIdx + 1) % THEMES.length;
  applyTheme(THEMES[themeIdx]);
  const sel = document.getElementById('theme-select');
  if (sel) sel.value = THEMES[themeIdx];
};

window.addEventListener('load', () => {
  const saved = localStorage.getItem('aero-theme') || 'dark';
  applyTheme(saved);
  themeIdx = THEMES.indexOf(saved);

  const themeSel = document.getElementById('theme-select');
  if (themeSel) {
    themeSel.value = saved;
    themeSel.onchange = () => { applyTheme(themeSel.value); themeIdx = THEMES.indexOf(themeSel.value); };
  }

  const langSel = document.getElementById('lang-select');
  if (langSel) {
    langSel.value = window.getLanguage ? window.getLanguage() : 'ru';
    langSel.onchange = () => {
      if (window.setLanguage) {
        window.setLanguage(langSel.value);
        window.dispatchEvent(new Event('aero-lang-changed'));
      }
    };
  }

  const blur = document.getElementById('blur-slider');
  if (blur) {
    blur.oninput = () => document.documentElement.style.setProperty('--blur', blur.value + 'px');
  }

  const accent = document.getElementById('accent-picker');
  if (accent) {
    accent.oninput = () => {
      document.documentElement.style.setProperty('--accent', accent.value);
      document.documentElement.style.setProperty('--accent-glow', accent.value + '66');
    };
  }
});