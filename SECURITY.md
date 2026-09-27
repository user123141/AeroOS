# Security Policy

## Supported versions

| Version | Supported |
|---------|:---------:|
| 1.9.x | ✅ |
| 1.8.x | ⚠️ |
| < 1.8 | ❌ |

## Reporting a vulnerability

**Do NOT open a public issue for security bugs.**

Email: `security@aeroos.dev` (placeholder)

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We aim to respond within 48 hours.

## Security model

AeroOS runs with **administrator privileges** because WHPX requires them.
The threat model assumes:

- The host OS is trusted
- The guest OS is untrusted
- Snapshot files on disk are untrusted (verify before restore)

### Mitigations

| Attack | Mitigation |
|--------|-----------|
| Malicious guest code | VM isolation via WHPX |
| Snapshot tampering | BLAKE3 hash verification |
| WebSocket hijacking | Session token + localhost-only bind |
| XSS in UI | CSP header, no `eval`, escaped output |
| Path traversal in file ops | `canonicalize()` + whitelist |

## Known limitations

- WebSocket is unencrypted (localhost only)
- No 2FA on the UI
- Session token regenerated on restart (no persistence)

## Updates

Security patches are released as patch versions (e.g. 1.9.1).