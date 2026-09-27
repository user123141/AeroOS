let termInstance = null;
window.initTerminal = function() {
  if (termInstance) return;
  const container = document.getElementById('xterm-container');
  if (!container || !window.Terminal) return;
  const term = new window.Terminal({
    cursorBlink: true,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    fontSize: 13,
    theme: { background: 'rgba(0,0,0,0)', foreground: '#e0e0e0', cursor: '#ffffff' },
  });
  const fit = new window.FitAddon.FitAddon();
  term.loadAddon(fit);
  term.open(container);
  fit.fit();
  term.writeln('AeroOS Terminal v1.0.0');
  term.writeln('Connected to ConPTY backend.');
  term.writeln('');
  term.write('$ ');
  let line = '';
  term.onData(data => {
    if (data === '\r') {
      window.term = term;
      if (line.trim()) send({ type: 'TerminalInput', data: line });
      term.write('\r\n$ ');
      line = '';
    } else if (data === '\u007f') {
      if (line.length > 0) { line = line.slice(0, -1); term.write('\b \b'); }
    } else { line += data; term.write(data); }
  });
  window.term = term;
  termInstance = term;
  window.addEventListener('resize', () => fit.fit());
};