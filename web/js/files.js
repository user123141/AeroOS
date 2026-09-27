window.renderFiles = function(files) {
  const list = document.getElementById('file-list');
  list.innerHTML = '';
  files.forEach(f => {
    const div = document.createElement('div');
    div.className = 'info-row';
    div.innerHTML = '<span>' + f.name + '</span><span>' + f.size + '</span>';
    list.appendChild(div);
  });
};
let filesInited = false;
window.initFiles = function() {
  if (filesInited) return;
  send({ type: 'ListFiles' });
  filesInited = true;
};