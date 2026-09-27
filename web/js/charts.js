// Real-time charts using Canvas (no external libs).
const CPU = { history: new Array(60).fill(0), ctx: null };
const MEM = { history: new Array(60).fill(0), ctx: null };

function setupCanvas(id) {
  const c = document.getElementById(id);
  if (!c) return null;
  const dpr = window.devicePixelRatio || 1;
  c.width = c.clientWidth * dpr;
  c.height = c.clientHeight * dpr;
  return c.getContext('2d');
}

function drawChart(ctx, data, color, label) {
  if (!ctx) return;
  const W = ctx.canvas.width;
  const H = ctx.canvas.height;

  ctx.clearRect(0, 0, W, H);

  // Grid
  ctx.strokeStyle = 'rgba(255,255,255,0.05)';
  ctx.lineWidth = 1;
  for (let i = 1; i < 4; i++) {
    const y = (H / 4) * i;
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(W, y); ctx.stroke();
  }

  // Line
  const max = Math.max(1, ...data);
  ctx.strokeStyle = color;
  ctx.lineWidth = 2 * (window.devicePixelRatio || 1);
  ctx.beginPath();
  data.forEach((v, i) => {
    const x = (i / (data.length - 1)) * W;
    const y = H - (v / max) * H * 0.9 - H * 0.05;
    if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
  });
  ctx.stroke();

  // Fill under
  ctx.lineTo(W, H); ctx.lineTo(0, H); ctx.closePath();
  const grad = ctx.createLinearGradient(0, 0, 0, H);
  grad.addColorStop(0, color + '40');
  grad.addColorStop(1, color + '00');
  ctx.fillStyle = grad;
  ctx.fill();
}

window.pushChart = function (dirtyPages) {
  CPU.history.shift();   CPU.history.push(Math.min(100, dirtyPages / 100));
  MEM.history.shift();   MEM.history.push(Math.min(100, dirtyPages / 200));
  drawChart(CPU.ctx, CPU.history, '#4a9eff');
  drawChart(MEM.ctx, MEM.history, '#00d4ff');
};

window.addEventListener('load', () => {
  CPU.ctx = setupCanvas('chart-cpu');
  MEM.ctx = setupCanvas('chart-mem');
  drawChart(CPU.ctx, CPU.history, '#4a9eff');
  drawChart(MEM.ctx, MEM.history, '#00d4ff');
  window.addEventListener('resize', () => {
    CPU.ctx = setupCanvas('chart-cpu');
    MEM.ctx = setupCanvas('chart-mem');
  });
});