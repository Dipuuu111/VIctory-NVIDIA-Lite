# VIctory-NVIDIA-Lite

A performance and memory-optimization fork of **VIctory-NVIDIA**, engineered to run the latest flagship Xbox Series X|S titles cleanly at **native 1080p** on mainstream **8 GB VRAM / 16 GB RAM** NVIDIA Windows PCs and laptops.

> **Notice:** This repository contains source code for the compatibility layer only. It contains **no** game files, shaders, keys, or copyrighted assets. You must supply your own legal dump of the title.

---

## Why This Fork Exists

Upstream builds are primarily tuned for desktop workstations with 32–64 GB of system RAM and 16–24 GB GPUs. On 8 GB VRAM / 16 GB RAM laptops and desktops, that causes:
- Permanent VRAM starvation on Series S 1080p render targets (delaying bindless texture streaming by 50–100+ frames)
- Multi-gigabyte idle staging, snapshot, and shader-container RAM pools triggering heavy Windows pagefile thrashing
- High CPU frame times on the submitting thread (`refreshes`, descriptor heap sweeps, write-watch queries, and `card_steps` planning)

**VIctory-NVIDIA-Lite** focuses on fixing those architectural bottlenecks directly in Rust so native 1080p rendering stays clean, responsive, and stable within 8 GB VRAM / 16 GB RAM budgets.

---

## Progress & Optimizations Implemented

### 1. VRAM & Texture Streaming (8 GB GPU Fixes)
- **Fixed Series S 1080p Render-Target Reserve Lock (`resource.rs`):** `targets_kept()` previously waited for render targets to exceed `1,536 MiB` (tuned for 1440p/4K) before settling. Series S 1080p peaks around `~1,340 MiB`, permanently locking `~1.7 GB` of phantom VRAM and starving textures. Lowered the settle floor to `(most / 3).min(1024 MiB)`, cutting first-time bindless texture upload delay from **~55.5 frames (105 max) → 0.4–0.8 frames**.
- **Smaller Pool Blocks & Tighter Idle Caps (`pool.rs`):** Reduced texture/staging pool block size from `64 MiB` → `32 MiB` and capped idle VRAM/staging pools (`SPARE_KEPT` and `STAGING_KEPT`) so freed texture memory returns to D3D12/WDDM instead of hoarding up to 4 GiB.
- **Sharp 1080p Loading Cards & UI (`resource.rs`):** Restricted `VICTORY_TEXTURE_DROP_MIPS` (`drop_for`) to large power-of-two 3D streaming textures (`>= 1024×1024`, `>= 8` mips), leaving `1920×1080` loading cards and 2D UI artwork at full native resolution.
- **Specular Reflection Stand-In Fix (`draw.rs`):** Added `1×` half-resolution `DXGI_FORMAT_R32_FLOAT` depth matching for `Role::Miss` so Series S reflection miss shaders match cleanly.
- **Cut Texture-Pass CPU & Lock Contention (`draw.rs`):** Trimmed redundant static-texture fingerprint rechecks (`RECHECK_FRAMES = [4, 40]`), reducing unchanged rechecks by ~98% and halving texture-queue lock waits.

### 2. Host RAM Reduction (16 GB Host Fixes)
- **Zero-Heap Shader Container Mapping (`pipeline/known.rs`, `pipeline.rs`):** Memory-mapped `known-shaders.bin` read-only via `MapViewOfFile` so raw shader containers are demand-paged from disk (~0 bytes private heap RAM instead of ~200 MB).
- **Capped Idle Upload & Snapshot Pools (`host/send.rs`, `shadow.rs`):** Capped `host/send.rs` spare upload buffers to `128 MiB` (down from `768 MiB`) on `<= 32 GiB` hosts, and wired `VICTORY_SNAPSHOT_POOL_MB` into `shadow::give_back()`.

### 3. CPU Frame-Time & Threading Improvements
- **Cached Buffer Sweeps & Packed Descriptor Allocator (`draw.rs`, `descriptor/host.rs`):** Added `BufferSweep` per-part cached spans (cutting `heap buffers` pass time from `~14–16 ms` → `~1–2 ms`) and replaced the 256 KiB `Vec<bool>` dynamic-slot tracker with a 32 KiB L1-resident `DynamicBits` packed `u64` bitset.
- **Faster Write-Watch & Refresh Pipeline (`watch.rs`, `shadow.rs`, `host.rs`):**
  - Binary-search `Journal::order_ahead` / `covers_ahead` indexing.
  - Chunk-batched `JOURNAL` `RwLock` acquisitions per 64-item refresh chunk (~64× fewer `RwLock` acquisitions/frame).
  - Combined `Shadow::copied`, `ranged`, and O(1) `unseen` page tracking under `CopiedState` with an O(dirty pages) `copy_stale` fast path, releasing `copied` before copying and clearing stale `landing` flags in `take_landed`.
  - Deduplicated failed `GetWriteWatch` queries and pre-filtered guarded blocks in `watch.rs::ask_rest`.
- **Parallel Read-Backs & Fast `card_steps` Planning (`texback.rs`, `host/card_steps.rs`):**
  - Converted `texback::TICKETS` to `RwLock` with parallel `land_many` and per-flight range deduplication.
  - Eliminated ~2,975 per-frame `list.deferred()` mutex locks, linear scans, and `Box` heap allocations in `host/card_steps.rs`.
- **Release Profile (`Cargo.toml`):** Enabled `lto = "thin"` for release builds.

---

## Roadmap (In Progress)
- Further reducing Private RAM / WDDM commit on 16 GB systems (e.g., releasing post-compile DXIL blobs and tightening idle buffer retention) to eliminate SSD pagefile thrashing during active movement.
- Pushing steady native 1080p gameplay on 8 GB RTX laptops from ~10–12 FPS toward 20–30 FPS without upscaling or frame generation.

---

## Building from Source

### Requirements
- Windows 10 or 11 (x64)
- NVIDIA GeForce RTX GPU with recent drivers (8 GB VRAM supported; 16 GB+ system RAM recommended, SSD strongly recommended)
- [Rust](https://rustup.rs) toolchain and [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (*Desktop development with C++*)

### Build Commands
```powershell
git clone https://github.com/Dipuuu111/VIctory-NVIDIA-Lite.git
cd VIctory-NVIDIA-Lite
cargo build --release --locked -p victory
```

The compiled executable is placed at `target\release\victory.exe`.

### Running
Place `victory.exe` alongside your own dump's `game_scarlett_master.exe` (or pass its path directly):
```powershell
.\target\release\victory.exe run "path\to\game_scarlett_master.exe" --console series-s --execute
```
- **F1** — In-game settings overlay
- **F3** — Live performance, draw-call, and memory stats overlay

---

## Credits & Acknowledgements

This fork builds directly on the hard work of the community:
- **VIctory** and **XCL** — the core Xbox compatibility layer and original project.
- **[trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA)** — the upstream NVIDIA fork this repository is based on.
- **[XaviFortes/VIctory-RDNA](https://github.com/XaviFortes/VIctory-RDNA)** — inspiration for `BufferSweep` cached spans, `DynamicBits`, parallel `texback::land_many`, and `Journal::order_ahead`.
- **Dark-Victory** and other community contributors — low-RAM/VRAM tuning concepts.
