# AeroOS API Reference

Справочник публичного API всех модулей. Обновляется вручную после значимых изменений.

## Модули

### `aerofile.rs`
Парсинг Aerofile (TOML конфиг VM).

    pub fn load(path: &Path) -> Result<Aerofile>
    pub fn validate(&self) -> Result<()>
    pub fn template(name: &str) -> Aerofile
    pub fn save(&self, path: &Path) -> Result<()>

### `agent.rs`
Хост-сторона протокола AeroAgent.

    pub fn new() -> AgentChannel
    pub fn send_to_guest(&self, data: Vec<u8>)
    pub fn recv_from_guest(&self) -> Option<Vec<u8>>
    pub async fn send(&self, cmd: HostCommand) -> Result<GuestResponse>

### `boot_vm.rs`
Загрузка bzImage (Linux 64-bit boot protocol).

    pub fn load_kernel(guest_mem, size, kernel_path, initramfs, cmdline) -> Result<BootResult>
    pub fn setup_vcpu_registers(vcpu: &mut VirtualProcessor, entry: u64) -> Result<()>

### `config.rs`
Загрузка `aeroos.config.toml`.

    pub fn load(path: &Path) -> Result<AeroConfig>

### `crypto.rs`
AES-256-GCM + Argon2id.

    pub fn new(password: &[u8], salt: [u8; 16]) -> Result<Encryptor>
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>>
    pub fn random_salt() -> [u8; 16]

### `data_folder.rs`
Data Folder mode (kernel рядом с .exe).

    pub fn new(config: &AeroConfig) -> Result<DataFolder>
    pub fn prepare(&self) -> Result<()>
    pub fn kernel_path(&self) -> PathBuf
    pub fn initramfs_path(&self) -> PathBuf

### `gpu_passthrough.rs`
Hyper-V GPU-P (partitioning).

    pub fn new() -> Result<GpuPassthrough>
    pub fn devices(&self) -> &[GpuDevice]
    pub fn attach_to_vm(&self, vm_name: &str, pci_id: &str) -> Result<()>
    pub fn detach_from_vm(&self, vm_name: &str) -> Result<()>

### `hwid.rs`
Hardware fingerprint (SHA-256).

    pub fn compute_hwid() -> Result<String>
    pub fn check_hwid(expected: &str) -> Result<()>

### `hypervisor.rs` — **ключевой**
Управление VM через WHPX.

    pub fn new(config: &AeroConfig, df: &DataFolder) -> Result<VirtualMachine>
    pub async fn run(&mut self) -> Result<()>
    pub fn stop(&self)
    pub fn get_state(&self) -> Result<Vec<u8>>
    pub fn restore_state(&mut self, data: &[u8]) -> Result<()>
    pub fn get_dirty_pages(&self) -> Vec<u64>
    pub fn clear_dirty_bitmap(&self)
    pub fn read_serial(&self) -> Vec<u8>
    pub fn clear_serial(&self)
    pub fn get_memory(&self) -> (*mut u8, usize)
    pub fn is_running(&self) -> bool
    pub fn config(&self) -> &AeroConfig
    pub fn stats(&self) -> VmStats

### `ipc.rs`
WebSocket IPC с токен-аутентификацией.

    pub fn new(vm: Arc<Mutex<VirtualMachine>>, token: String) -> IpcState
    pub async fn run_websocket_server(state: IpcState) -> Result<()>

### `license.rs`
Ed25519 + HWID + tier gating.

    pub fn verify(&self) -> Result<()>
    pub fn load(path: &Path) -> Result<LicenseState>
    pub fn check(&self, feature: &str) -> Result<()>
    pub fn max_vms(&self) -> u32
    pub fn max_ram_mb(&self) -> u64

### `migration.rs`
Live migration (.aeromig формат).

    pub fn save_to_file(memory: &[u8], header: MigrationHeader, out_path: &Path) -> Result<()>
    pub fn load_from_file(path: &Path) -> Result<(MigrationHeader, Vec<u8>)>

### `multi_vm.rs`
Multi-VM manager.

    pub fn new(max_vms: u32) -> MultiVmManager
    pub fn create(&mut self, id: &str, name: &str, vm: VirtualMachine) -> Result<()>
    pub fn get(&self, id: &str) -> Option<&VmInstance>
    pub fn count(&self) -> usize
    pub async fn all_stats(&self) -> HashMap<VmId, VmStats>

### `net_proxy.rs`
TCP proxy guest → host socket.

    pub async fn poll_listener(iface, sockets, listener_handle, host_target) -> Result<()>
    pub async fn poll_back(&mut self, sockets) -> Result<()>

### `net_smoltcp.rs`
User-mode TCP/IP (без прав админа).

    pub fn new() -> Result<SmoltcpBackend>
    pub async fn run(&mut self) -> Result<()>

### `oci.rs`
OCI pull + initramfs conversion.

    pub fn new(image_dir: &Path) -> Result<OciRuntime>
    pub async fn fetch_manifest(&self, registry, repo, tag) -> Result<OciManifest>
    pub async fn pull_layers(&self, registry, repo, manifest) -> Result<PathBuf>
    pub fn to_initramfs(&self, image_dir: &Path, out_path: &Path) -> Result<()>

### `registry.rs`
AeroRegistry клиент.

    pub fn new() -> Result<AeroRegistry>
    pub fn list(&self) -> Vec<&'static str>
    pub async fn pull(&self, name: &str, out_path: &Path) -> Result<()>

### `sandbox.rs`
Windows AppContainer.

    pub fn init_sandbox(allowed_paths: &[&str]) -> Result<()>
    pub fn is_sandboxed() -> bool

### `security.rs`
SHA-256, session tokens, disk checks.

    pub fn sha256_file(path: &Path) -> Result<String>
    pub fn verify_artifacts() -> Result<()>
    pub fn generate_session_token() -> String
    pub fn check_disk_space(dir: &Path, required_mb: u64) -> Result<()>

### `snapshot.rs` — **ключевой**
Инкрементальные снапшоты.

    pub fn new(config: &SnapshotConfig) -> SnapshotManager
    pub fn create_incremental(&mut self, memory, size, dirty: &[u64]) -> Result<()>
    pub fn create_snapshot(&mut self, data: &[u8], name: &str) -> Result<PathBuf>
    pub fn restore_snapshot(&self, name: &str) -> Result<Vec<u8>>
    pub fn list_snapshots(&self) -> Vec<String>
    pub fn verify_integrity(&self) -> Result<u64>
    pub async fn start_streaming(manager, vm, interval_ms)

### `tap.rs`
Wintun TAP (требует админ).

    pub fn open(name: &str) -> Result<TapDevice>
    pub fn try_recv(&self) -> Option<Vec<u8>>
    pub fn send(&self, data: &[u8]) -> Result<()>

### `terminal.rs`
ConPTY терминал.

    pub fn new() -> Terminal
    pub fn init(&mut self) -> Result<()>
    pub fn write(&mut self, data: &[u8]) -> Result<()>
    pub fn read(&mut self) -> Result<Vec<u8>>

### `virtio.rs`
VirtIO MMIO устройства.

    pub fn new_block(base, size, path) -> VirtioMmio
    pub fn new_net(base, size, tap) -> VirtioMmio
    pub fn new_console(base, size) -> VirtioMmio
    pub fn new_gpu(base, size, w, h) -> VirtioMmio
    pub fn new_fs(base, size, host_path) -> VirtioMmio
    pub fn process_queue<M: GuestMemory>(&self, mem: &M) -> u32

### `virtio_9p.rs` (sync)
Базовый 9P сервер.

    pub fn new(host_root: &Path) -> Result<P9Server>
    pub fn handle(&mut self, data: &[u8]) -> Result<Vec<u8>>

### `virtio_fs.rs` (async)
Расширенный 9P сервер с tokio.

    pub fn new(host_root: &Path) -> Result<VirtioFsServer>
    pub async fn handle(&self, data: &[u8]) -> Result<Vec<u8>>

### `vnc.rs`
JPEG frame streaming.

    pub fn new(fb, w, h) -> FrameStream
    pub fn capture_jpeg(&self) -> Result<Vec<u8>>
    pub async fn run(self, tx: broadcast::Sender<Vec<u8>>)

### `web.rs`
HTTP server для UI.

    pub async fn run_server(token: String) -> Result<()>