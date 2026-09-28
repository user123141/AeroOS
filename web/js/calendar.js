// AeroOS Calendar with events

let calYear, calMonth;
let calEvents = JSON.parse(localStorage.getItem('aero-cal-events') || '{}');

function initCalendar() {
  const now = new Date();
  calYear = now.getFullYear();
  calMonth = now.getMonth();
}

function evKey(y, m, d) {
  return y + '-' + String(m + 1).padStart(2, '0') + '-' + String(d).padStart(2, '0');
}

function saveEvents() {
  localStorage.setItem('aero-cal-events', JSON.stringify(calEvents));
}

function renderCalendar() {
  const now = new Date();
  const today = now.getDate();
  const todayMonth = now.getMonth();
  const todayYear = now.getFullYear();

  const monthNames = ['Январь','Февраль','Март','Апрель','Май','Июнь','Июль','Август','Сентябрь','Октябрь','Ноябрь','Декабрь'];
  const dayNames = ['Пн','Вт','Ср','Чт','Пт','Сб','Вс'];

  const first = new Date(calYear, calMonth, 1);
  const startWeekday = (first.getDay() + 6) % 7;
  const daysInMonth = new Date(calYear, calMonth + 1, 0).getDate();
  const daysInPrev = new Date(calYear, calMonth, 0).getDate();

  const cells = [];
  for (let i = startWeekday - 1; i >= 0; i--) {
    cells.push({ day: daysInPrev - i, other: true });
  }
  for (let d = 1; d <= daysInMonth; d++) {
    cells.push({
      day: d,
      other: false,
      today: d === today && calMonth === todayMonth && calYear === todayYear,
      hasEvent: !!calEvents[evKey(calYear, calMonth, d)],
    });
  }
  while (cells.length % 7 !== 0) {
    cells.push({ day: cells.length - daysInMonth - startWeekday + 1, other: true });
  }

  const grid = document.getElementById('cal-grid');
  const title = document.getElementById('cal-title');
  if (title) title.textContent = monthNames[calMonth] + ' ' + calYear;
  if (!grid) return;

  grid.innerHTML = '';
  dayNames.forEach(d => {
    const el = document.createElement('div');
    el.className = 'cal-day-name';
    el.textContent = d;
    grid.appendChild(el);
  });
  cells.forEach(c => {
    const el = document.createElement('div');
    el.className = 'cal-day'
      + (c.other ? ' other-month' : '')
      + (c.today ? ' today' : '')
      + (c.hasEvent ? ' has-event' : '');
    el.textContent = c.day;
    if (!c.other) {
      el.onclick = (e) => {
        e.stopPropagation();
        const key = evKey(calYear, calMonth, c.day);
        const current = calEvents[key] || '';
        const v = prompt('Событие на ' + c.day + ':', current);
        if (v === null) return;
        if (v.trim() === '') {
          delete calEvents[key];
        } else {
          calEvents[key] = v;
        }
        saveEvents();
        renderCalendar();
      };
    }
    grid.appendChild(el);
  });
}

function ensureCalendar() {
  let c = document.getElementById('calendar-pop');
  if (!c) {
    c = document.createElement('div');
    c.id = 'calendar-pop';
    c.className = 'calendar-pop';

    const header = document.createElement('div');
    header.className = 'cal-header';

    const prev = document.createElement('button');
    prev.className = 'cal-nav';
    prev.id = 'cal-prev';
    prev.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>';

    const title = document.createElement('div');
    title.className = 'cal-month';
    title.id = 'cal-title';
    title.textContent = '---';

    const next = document.createElement('button');
    next.className = 'cal-nav';
    next.id = 'cal-next';
    next.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>';

    header.appendChild(prev);
    header.appendChild(title);
    header.appendChild(next);

    const grid = document.createElement('div');
    grid.className = 'cal-grid';
    grid.id = 'cal-grid';

    const today = document.createElement('button');
    today.className = 'cal-today-btn';
    today.id = 'cal-today';
    today.textContent = 'Сегодня';

    c.appendChild(header);
    c.appendChild(grid);
    c.appendChild(today);
    document.body.appendChild(c);

    document.getElementById('cal-prev').onclick = (e) => {
      e.stopPropagation();
      calMonth--;
      if (calMonth < 0) { calMonth = 11; calYear--; }
      renderCalendar();
    };
    document.getElementById('cal-next').onclick = (e) => {
      e.stopPropagation();
      calMonth++;
      if (calMonth > 11) { calMonth = 0; calYear++; }
      renderCalendar();
    };
    document.getElementById('cal-today').onclick = (e) => {
      e.stopPropagation();
      initCalendar();
      renderCalendar();
    };
    c.addEventListener('click', (e) => e.stopPropagation());
  }
  return c;
}

window.openCalendar = function() {
  const c = ensureCalendar();
  if (!c.classList.contains('on')) initCalendar();
  renderCalendar();
  c.classList.toggle('on');
};

window.addEventListener('load', () => {
  initCalendar();
  document.addEventListener('click', (e) => {
    const c = document.getElementById('calendar-pop');
    if (!c || !c.classList.contains('on')) return;
    if (e.target.closest('#calendar-pop')) return;
    if (e.target.closest('#topbar-clock')) return;
    c.classList.remove('on');
  });
});