# AeroOS Roadmap

## v4.1.0 — Horizon (current)

- ✅ Fix WHvSetVirtualProcessorRegisters (params + 16-byte values)
- ✅ End-to-end boot Alpine kernel (CR0/CR4/EFER + регистры)
- ✅ GPU-P through Hyper-V (PowerShell cmdlets)
- ✅ 9P wiring in virtio
- ✅ Multi-VM UI with per-VM stats
- ✅ Enterprise offline licensing (Ed25519 + HWID + nonce)
- 🟡 Live test: real Alpine kernel boot
- 🟡 9P read/write end-to-end

## v4.2.0 — Continuum

- ❌ Full end-to-end VM boot test
- ❌ WebRTC VP8 streaming
- ❌ OCI → initramfs end-to-end
- ❌ Aeroctl up actual boot

## v5.0.0 — Singularity

- ❌ Linux/macOS host
- ❌ Full GPU passthrough

## Backlog

- Time-travel diff viewer
- Snapshot export to QCOW2
- Plugin system