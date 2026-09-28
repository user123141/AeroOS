// AeroOS Crypto-UI binding: decrypt main.js if license is valid.
//
// Flow:
//   1. fetch /api/ui-key
//   2. If 403 — key not available, load main.js as plain (open mode)
//   3. If 200 — decrypt main.js.enc with AES-256-GCM
//   4. eval decrypted source

(async function() {
  async function fetchKey() {
    try {
      const r = await fetch('/api/ui-key');
      if (r.status === 403) return null;
      if (!r.ok) return null;
      const j = await r.json();
      return j.key || null;
    } catch (e) {
      console.warn('Crypto-UI: fetch key failed', e);
      return null;
    }
  }

  async function fetchEncrypted() {
    try {
      const r = await fetch('web_encrypted/main.js.enc');
      if (!r.ok) return null;
      const buf = await r.arrayBuffer();
      return new Uint8Array(buf);
    } catch (e) {
      return null;
    }
  }

  async function decrypt(keyHex, blob) {
    const keyBytes = new Uint8Array(keyHex.match(/.{1,2}/g).map(b => parseInt(b, 16)));
    const nonce = blob.slice(0, 12);
    const ct = blob.slice(12);
    const cryptoKey = await crypto.subtle.importKey(
      'raw', keyBytes, { name: 'AES-GCM' }, false, ['decrypt']
    );
    const pt = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: nonce }, cryptoKey, ct
    );
    return new TextDecoder().decode(pt);
  }

  const key = await fetchKey();
  if (key) {
    const blob = await fetchEncrypted();
    if (blob) {
      try {
        const src = await decrypt(key, blob);
        const s = document.createElement('script');
        s.textContent = src;
        document.head.appendChild(s);
        console.log('Crypto-UI: main.js decrypted and loaded');
        return;
      } catch (e) {
        console.error('Crypto-UI: decrypt failed, falling back to plain', e);
      }
    }
  }

  // Fallback: load plain main.js
  const s = document.createElement('script');
  s.src = 'js/main.js';
  document.head.appendChild(s);
  console.log('Crypto-UI: plain main.js loaded (no license or decrypt failed)');
})();