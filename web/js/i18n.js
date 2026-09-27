// AeroOS i18n — русский / английский

const AERO_I18N = {
  en: {
    dashboard:      "Dashboard",
    terminal:       "Terminal",
    timeline:       "Time Travel",
    settings:       "Settings",
    vm:             "Virtual Machine",
    cpu:            "CPU",
    ram:            "RAM",
    gpu:            "GPU",
    status:         "Status",
    running:        "Running",
    stopped:        "Stopped",
    snapshots:      "Snapshots",
    dirtyPages:     "Dirty pages",
    createSnapshot: "Create snapshot",
    restore:        "Restore",
    snapshotsTitle: "Snapshots",
    snapshotName:   "Snapshot name:",
    theme:          "Theme",
    themeDark:      "Dark",
    themeLight:     "Light",
    themeAero:      "Aero (glass)",
    blurIntensity:  "Blur intensity",
    accentColor:    "Accent color",
    language:       "Language",
    virtualMachines:"Virtual Machines",
    timelineHelp:   "Move the slider to restore any previous state instantly.",
    noSnapshots:    "no snapshots",
    now:            "now",
    welcome:        "Welcome to AeroOS",
    initializing:   "Initializing...",
    systemActive:   "System active",
    vmStopped:      "VM stopped (hypervisor off)",
    serial:         "Serial output",
    clearSerial:    "Clear",
  },
  ru: {
    dashboard:      "Панель",
    terminal:       "Терминал",
    timeline:       "Машина времени",
    settings:       "Настройки",
    vm:             "Виртуальная машина",
    cpu:            "Процессор",
    ram:            "ОЗУ",
    gpu:            "Видеокарта",
    status:         "Статус",
    running:        "Работает",
    stopped:        "Остановлена",
    snapshots:      "Снимки",
    dirtyPages:     "Изменённых страниц",
    createSnapshot: "Создать снимок",
    restore:        "Восстановить",
    snapshotsTitle: "Снимки",
    snapshotName:   "Имя снимка:",
    theme:          "Тема",
    themeDark:      "Тёмная",
    themeLight:     "Светлая",
    themeAero:      "Aero (стекло)",
    blurIntensity:  "Интенсивность размытия",
    accentColor:    "Акцентный цвет",
    language:       "Язык",
    virtualMachines:"Виртуальные машины",
    timelineHelp:   "Двигай ползунок, чтобы мгновенно откатить состояние назад.",
    noSnapshots:    "нет снимков",
    now:            "сейчас",
    welcome:        "Добро пожаловать в AeroOS",
    initializing:   "Инициализация...",
    systemActive:   "Система активна",
    vmStopped:      "VM остановлена (hypervisor выключен)",
    serial:         "Вывод ядра",
    clearSerial:    "Очистить",
  }
};

let currentLang = localStorage.getItem('aero-lang') || 'ru';

function t(key) {
  return (AERO_I18N[currentLang] && AERO_I18N[currentLang][key]) || key;
}

function applyTranslations() {
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    el.textContent = t(key);
  });
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.getAttribute('data-i18n-placeholder');
    el.placeholder = t(key);
  });
  document.documentElement.setAttribute('lang', currentLang);
}

function setLanguage(lang) {
  if (!AERO_I18N[lang]) return;
  currentLang = lang;
  localStorage.setItem('aero-lang', lang);
  applyTranslations();
}

window.t = t;
window.setLanguage = setLanguage;
window.getLanguage = () => currentLang;

window.addEventListener('load', applyTranslations);