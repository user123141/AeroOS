# Benchmarks

> Data collected on: [your machine specs]
> To reproduce: `cargo bench`

## Cold start

| Configuration | Time to UI |
|---------------|------------|
| AeroOS (no VM) | TBD |
| AeroOS (with VM) | TBD |
| WSL2 | ~3 s |
| VirtualBox | ~8 s |

## Memory footprint

| Configuration | Idle | Running |
|---------------|------|---------|
| AeroOS (no VM) | TBD | TBD |
| AeroOS (2 GB VM) | TBD | TBD |
| WSL2 | ~200 MB | ~500 MB |

## Snapshot performance

| Operation | Throughput |
|-----------|------------|
| LZ4 compress | TBD MB/s |
| BLAKE3 hash | TBD MB/s |
| AES-256-GCM encrypt | TBD MB/s |
| Full snapshot (1 GB diff) | TBD ms |

## Instructions

Run:

    cd AeroOS
    cargo bench 2>&1 | tee docs/benchmarks-raw.txt

Then update this file with the results.