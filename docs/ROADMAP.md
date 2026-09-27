# AeroOS Roadmap

## Legend
- ✅ Done
- 🟡 In progress
- ❌ Not started

## v3.0.0 — Singularity (current)

- ✅ Fix smoltcp 0.11 Device trait
- ✅ Fix config.rs ram_only
- ✅ Professional .gitignore
- ✅ .cargo/config.toml (OOM protection)
- 🟡 Real TCP proxy (host socket bridge)
- 🟡 9P end-to-end

## v3.1.0 — Continuum (planned)

- ❌ Real TCP proxy via smoltcp
- ❌ 9P read/write end-to-end
- ❌ WebRTC/VNC canvas streaming
- ❌ `aeroctl up` boots VM from Aerofile
- ❌ AeroRegistry online

## v3.2.0 — Horizon

- ❌ Multi-VM UI (tabs + per-VM stats)
- ❌ Live migration
- ❌ GPU passthrough (VirGL)
- ❌ OCI compatibility

## Backlog

- Time-travel diff viewer
- Snapshot export to QCOW2
- Guest agent file sync
- Plugin system for UI
- Encrypted disk images