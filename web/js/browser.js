let browserInited = false;
window.initBrowser = function() {
  if (browserInited) return;
  const go = document.getElementById('browser-go');
  const url = document.getElementById('browser-url');
  const frame = document.getElementById('browser-frame');
  go.onclick = () => { if (url.value) frame.src = url.value; };
  url.onkeydown = e => { if (e.key === 'Enter' && url.value) frame.src = url.value; };
  browserInited = true;
};