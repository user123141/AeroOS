# AeroOS Roadmap

## Legend
- ✅ Done
- 🟡 In progress
- ❌ Not started

## v3.2.0 — Momentum (current)

- ✅ Fix SnapshotConfig Default
- ✅ TCP Proxy (guest → host socket)
- ✅ VNC/Frame streaming (JPEG over WS)
- ✅ Live migration (.aeromig)
- ✅ OCI runtime (manifest parse)
- ✅ GPU passthrough (device detection)
- 🟡 Real TCP data flow end-to-end
- 🟡 9P read/write through virtio

## v3.3.0 — Continuum (next)

- ❌ 9P full data transfer
- ❌ OCI → initramfs conversion
- ❌ WebRTC VP8/H.264
- ❌ `aeroctl up` actual VM boot
- ❌ AeroRegistry online

## v4.0.0 — Singularity (vision)

- ❌ Multi-VM UI (per-VM stats, tabs)
- ❌ GPU passthrough actual (WDDM/Hyper-V GPU-P)
- ❌ Linux/macOS host support
- ❌ Enterprise license server

## Backlog

- Time-travel diff viewer
- Snapshot export to QCOW2
- Guest agent file sync
- Plugin system
- Encrypted disk images
- Sound subsystem