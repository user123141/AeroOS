# AeroOS Structure
# 2026-09-28 01:35:49

|-- .cargo/
|   |-- config.toml
|-- .github/
|   |-- workflows/
|   |   |-- ci.yml
|-- benches/
|   |-- snapshot.rs
|-- boot/
|-- build/
|   |-- kernel/
|   |   |-- aeroos-initramfs
|   |   |-- aeroos-kernel
|   |-- aeroos.config.toml
|   |-- aeroos.exe
|   |-- wintun.dll
|-- config/
|   |-- aeroos.config.toml
|-- data/
|-- docs/
|   |-- AERO.md
|   |-- AEROFILE.md
|   |-- ARCHITECTURE.md
|   |-- BENCHMARKS.md
|   |-- LAST_ERROR.txt
|   |-- ROADMAP.md
|   |-- STRUCTURE.md
|-- guest-agent/
|   |-- src/
|   |   |-- main.rs
|   |-- Cargo.toml
|-- kernel/
|   |-- external/
|   |   |-- aeroos.fragment
|   |   |-- aeroos_defconfig
|   |   |-- virtio.fragment
|   |-- output/
|   |   |-- aeroos-initramfs
|   |   |-- aeroos-kernel
|   |-- aeroos.fragment
|   |-- aeroos_defconfig
|   |-- virtio.fragment
|-- scripts/
|   |-- build-all.ps1
|   |-- build-kernel.ps1
|   |-- build-native.ps1
|-- src/
|   |-- bin/
|   |   |-- aeroctl.rs
|   |   |-- gen-license.rs
|   |-- aerofile.rs
|   |-- agent.rs
|   |-- boot.rs
|   |-- boot_vm.rs
|   |-- config.rs
|   |-- crypto.rs
|   |-- data_folder.rs
|   |-- gpu_passthrough.rs
|   |-- hwid.rs
|   |-- hypervisor.rs
|   |-- ipc.rs
|   |-- license.rs
|   |-- main.rs
|   |-- migration.rs
|   |-- multi_vm.rs
|   |-- net_proxy.rs
|   |-- net_smoltcp.rs
|   |-- oci.rs
|   |-- registry.rs
|   |-- sandbox.rs
|   |-- security.rs
|   |-- snapshot.rs
|   |-- tap.rs
|   |-- terminal.rs
|   |-- virtio.rs
|   |-- virtio_9p.rs
|   |-- virtio_fs.rs
|   |-- vnc.rs
|   |-- web.rs
|-- vendor/
|   |-- conpty/
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |-- Cargo.toml
|   |-- whpx/
|   |   |-- src/
|   |   |   |-- lib.rs
|   |   |-- Cargo.toml
|-- web/
|   |-- css/
|   |   |-- style.css
|   |-- js/
|   |   |-- apps/
|   |   |-- browser.js
|   |   |-- charts.js
|   |   |-- files.js
|   |   |-- main.js
|   |   |-- terminal.js
|   |   |-- theme.js
|   |   |-- timeline.js
|   |   |-- vm-tabs.js
|   |-- vendor/
|   |-- index.html
|-- .editorconfig
|-- .gitattributes
|-- .gitignore
|-- build.rs
|-- Cargo.lock
|-- Cargo.toml
|-- CHANGELOG.md
|-- CODE_OF_CONDUCT.md
|-- CONTRIBUTING.md
|-- LICENSE
|-- README.md
|-- README.ru.md
|-- rust-toolchain.toml
|-- SECURITY.md
