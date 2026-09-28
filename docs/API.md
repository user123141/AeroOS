# AeroOS API Reference

Public API of every module. Updated manually.

## IPC Commands (WebSocket, port 8081)

JSON with `"type"` tag. Query string: `?token=<session>`.

| Command | Payload | Response |
|---------|---------|----------|
| GetStatus | — | Status |
| GetStats | — | Stats |
| ListVms | — | VmsList |
| GetSerial | — | Serial |
| ClearSerial | — | Ok |
| Snapshot | { name } | Ok |
| Restore | { name } | Ok |
| ListSnapshots | — | Snapshots |
| DeleteSnapshot | { name } | Ok |
| SetVolume | { level: 0..100 } | Ok / Error |
| SetBrightness | { level: 0..100 } | Ok / Error |
| GetSystemState | — | SystemState |
| Search | { query } | SearchResults |
| Stop | — | Ok |
| TerminalInput | { data } | TerminalOutput |

## Rust modules

### hypervisor.rs
    pub fn new(config, df) -> Result<VirtualMachine>
    pub async fn run(&mut self) -> Result<()>
    pub fn stop(&self)
    pub fn get_state(&self) -> Result<Vec<u8>>
    pub fn restore_state(&mut self, data) -> Result<()>
    pub fn get_dirty_pages(&self) -> Vec<u64>
    pub fn clear_dirty_bitmap(&self)
    pub fn read_serial(&self) -> Vec<u8>
    pub fn clear_serial(&self)
    pub fn get_memory(&self) -> (*mut u8, usize)
    pub fn is_running(&self) -> bool
    pub fn config(&self) -> &AeroConfig
    pub fn stats(&self) -> VmStats

### snapshot.rs
    pub fn new(config) -> SnapshotManager
    pub fn create_incremental(&mut self, memory, size, dirty) -> Result<()>
    pub fn create_snapshot(&mut self, data, name) -> Result<PathBuf>
    pub fn restore_snapshot(&self, name) -> Result<Vec<u8>>
    pub fn list_snapshots(&self) -> Vec<String>
    pub fn verify_integrity(&self) -> Result<u64>
    pub async fn start_streaming(manager, vm, interval_ms)

### syscontrol.rs
    pub fn set_volume(level: u32) -> Result<()>
    pub fn get_volume() -> Result<u32>
    pub fn set_brightness(level: u32) -> Result<()>
    pub fn get_brightness() -> Result<u32>

### license.rs
    pub fn verify(&self) -> Result<()>
    pub fn load(path) -> Result<LicenseState>
    pub fn check(&self, feature) -> Result<()>
    pub fn max_vms(&self) -> u32
    pub fn max_ram_mb(&self) -> u64
    pub fn derive_ui_key(license) -> [u8; 32]

### boot.rs (AeroBoot)
    pub fn new() -> Self
    pub fn load_kernel(&mut self, path) -> Result<()>
    pub fn load_initramfs(&mut self, path) -> Result<()>
    pub fn compute_checksum(&mut self)
    pub fn verify(&self) -> bool
    pub fn prepare_memory_layout(&self, guest_mem) -> Result<()>
    pub fn build_uefi_image(&self, out_path) -> Result<()>
    pub fn load_from_data_folder(&mut self, kernel, initramfs) -> Result<()>

### boot_vm.rs
    pub fn load_kernel(guest_mem, size, kernel, initramfs, cmdline) -> Result<BootResult>
    pub fn setup_vcpu_registers(vcpu, entry) -> Result<()>

### ipc.rs
    pub fn new(vm, token) -> IpcState
    pub async fn run_websocket_server(state) -> Result<()>

### web.rs
    pub async fn run_server(token) -> Result<()>

### crypto.rs
    pub fn new(password, salt) -> Result<Encryptor>
    pub fn encrypt(&self, plaintext) -> Result<Vec<u8>>
    pub fn decrypt(&self, data) -> Result<Vec<u8>>
    pub fn random_salt() -> [u8; 16]

### security.rs
    pub fn sha256_file(path) -> Result<String>
    pub fn verify_artifacts() -> Result<()>
    pub fn generate_session_token() -> String
    pub fn check_disk_space(dir, required_mb) -> Result<()>

### config.rs
    pub fn load(path) -> Result<AeroConfig>

### data_folder.rs
    pub fn new(config) -> Result<DataFolder>
    pub fn prepare(&self) -> Result<()>
    pub fn kernel_path(&self) -> PathBuf
    pub fn initramfs_path(&self) -> PathBuf

### gpu_passthrough.rs
    pub fn new() -> Result<GpuPassthrough>
    pub fn devices(&self) -> &[GpuDevice]
    pub fn attach_to_vm(&self, vm_name, pci_id) -> Result<()>
    pub fn detach_from_vm(&self, vm_name) -> Result<()>

### hwid.rs
    pub fn compute_hwid() -> Result<String>
    pub fn check_hwid(expected) -> Result<()>

### migration.rs
    pub fn save_to_file(memory, header, out_path) -> Result<()>
    pub fn load_from_file(path) -> Result<(MigrationHeader, Vec<u8>)>

### multi_vm.rs
    pub fn new(max_vms) -> MultiVmManager
    pub fn create(&mut self, id, name, vm) -> Result<()>
    pub fn get(&self, id) -> Option<&VmInstance>
    pub fn count(&self) -> usize
    pub async fn all_stats(&self) -> HashMap<VmId, VmStats>

### net_smoltcp.rs
    pub fn new() -> Result<SmoltcpBackend>
    pub async fn run(&mut self) -> Result<()>

### net_proxy.rs
    pub async fn poll_listener(iface, sockets, listener_handle, host_target) -> Result<()>
    pub async fn poll_back(&mut self, sockets) -> Result<()>

### oci.rs
    pub fn new(image_dir) -> Result<OciRuntime>
    pub async fn fetch_manifest(&self, registry, repo, tag) -> Result<OciManifest>
    pub async fn pull_layers(&self, registry, repo, manifest) -> Result<PathBuf>
    pub fn to_initramfs(&self, image_dir, out_path) -> Result<()>

### registry.rs
    pub fn new() -> Result<AeroRegistry>
    pub fn list(&self) -> Vec<&'static str>
    pub async fn pull(&self, name, out_path) -> Result<()>

### sandbox.rs
    pub fn init_sandbox(allowed_paths) -> Result<()>
    pub fn is_sandboxed() -> bool

### virtio.rs
    pub fn new_block(base, size, path) -> VirtioMmio
    pub fn new_net(base, size, tap) -> VirtioMmio
    pub fn new_console(base, size) -> VirtioMmio
    pub fn new_gpu(base, size, w, h) -> VirtioMmio
    pub fn new_fs(base, size, host_path) -> VirtioMmio
    pub fn process_queue<M: GuestMemory>(&self, mem) -> u32

### virtio_fs.rs (async 9P)
    pub fn new(host_root) -> Result<VirtioFsServer>
    pub async fn handle(&self, data) -> Result<Vec<u8>>

### virtio_9p.rs (sync 9P)
    pub fn new(host_root) -> Result<P9Server>
    pub fn handle(&mut self, data) -> Result<Vec<u8>>

### vnc.rs
    pub fn new(fb, w, h) -> FrameStream
    pub fn capture_jpeg(&self) -> Result<Vec<u8>>
    pub async fn run(self, tx)

### terminal.rs
    pub fn new() -> Terminal
    pub fn init(&mut self) -> Result<()>
    pub fn write(&mut self, data) -> Result<()>
    pub fn read(&mut self) -> Result<Vec<u8>>

### tap.rs (wintun)
    pub fn open(name) -> Result<TapDevice>
    pub fn try_recv(&self) -> Option<Vec<u8>>
    pub fn send(&self, data) -> Result<()>

### agent.rs
    pub fn new() -> AgentChannel
    pub fn send_to_guest(&self, data)
    pub async fn send(&self, cmd) -> Result<GuestResponse>

### aerofile.rs
    pub fn load(path) -> Result<Aerofile>
    pub fn validate(&self) -> Result<()>
    pub fn template(name) -> Self
    pub fn save(&self, path) -> Result<()>