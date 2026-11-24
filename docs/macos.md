# Experimental Apple Silicon support

VIctory still builds as a Windows x64 executable. On Apple Silicon the
experimental runtime stack is Wine, Rosetta, and Apple's D3DMetal. There
is no native macOS game runner yet.

## Tested configuration

- Apple M1, macOS 15.7.6, Rosetta with AVX enabled.
- Maintained Whisky fork (`frankea/whisky/whisky`) 3.10.2.
- WhiskyWine 4.8.3 / Wine 11.19.
- Apple's evaluation environment for Windows games 3.0, imported through
  Whisky Settings → Game Porting Toolkit.

The Windows release build, `victory.exe config`, and mapping/import
resolution pass. After the memory fix described below, title execution
gets past initialization, opens the data archives, and renders the account
picker and first-run accessibility menus at approximately 18–20 fps.
The user also completed setup and the intro in Series X mode, but the
title/start screen was not reached: the run remained on a slow post-intro
loading screen. Series S reduced the measured process footprint from
10.6 GiB to 4.7 GiB, but that attempt stayed black and ended at a title
breakpoint during loading. These remain compatibility failures; this
fork does not yet claim a working start screen on Mac. A synthetic window
test succeeds when launched through Whisky's desktop UI.

The coding agent's background process cannot initialize Wine's display
driver on this setup, even for a synthetic window. Launch GUI tests from
an interactive Terminal or Whisky instead of that background session.

The upstream Windows results in the README do not establish Mac support.
Stop at the start screen when testing. Do not load the world.

## Setup and build

Install Rosetta and the compiler tools, then install the maintained Whisky
fork. The original Homebrew `whisky` cask installs the archived app.

```sh
softwareupdate --install-rosetta --agree-to-license
brew install llvm lld rustup
brew install --cask frankea/whisky/whisky
```

Complete Whisky's runtime setup and create a Windows 11 bottle. Import
Apple's evaluation-environment DMG using Settings → Game Porting Toolkit.
Enable AVX for the bottle. The launch environment must include
`ROSETTA_ADVERTISE_AVX=1`. Select D3DMetal as the graphics backend.

Install the Rust version in `rust-toolchain.toml`, including the
`x86_64-pc-windows-msvc` target. Build with Homebrew LLVM and LLD on PATH:

```sh
export PATH="$(brew --prefix rustup)/bin:$(brew --prefix llvm)/bin:$(brew --prefix lld)/bin:$HOME/.cargo/bin:$PATH"
cargo install cargo-xwin --locked
cargo xwin build --locked --release -p victory --target x86_64-pc-windows-msvc
```

The output is `target/x86_64-pc-windows-msvc/release/victory.exe`. Run it
inside the bottle. First run `config`, then `run <exe>` without
`--execute`, then attempt `run <exe> --execute`. Use Wine-visible paths.
Keep the title executable beside its data and DLLs, outside the source
checkout. Set `VICTORY_CACHE_DIR` and `VICTORY_DATA_DIR` to writable
locations outside the checkout. A read-only game directory is supported;
keep derived data and saves separate from the original files.

For a lower memory budget, add `--console series-s` to the `run` command.
This reduces the game's allocations and resolution, but did not resolve
the remaining loading failure in the tested configuration.

## Rosetta memory-write tracking

Wine implements `MEM_WRITE_WATCH` using page protection faults. A 256-bit
AVX store spanning a protected 16 KiB host-page boundary can cause
Rosetta to abort with `unexpectedly need to EmulateForward on a synchronous
exception`. This is reproducible with a synthetic allocation and store,
without game data. The same store succeeds on ordinary writable memory.

VIctory detects Wine's Darwin host through `ntdll!wine_get_host_version`.
On that host it omits `MEM_WRITE_WATCH` from title allocations and bypasses
the GPU memory tracker's read-only view guards. Both mechanisms can
trigger the fault. GPU synchronization uses the existing full-copy and
texture-hashing fallbacks. This trades performance for compatibility;
Windows and Wine on other hosts keep their existing tracking behavior.

The Windows unit tests include regressions for a cross-page AVX store in
title memory, mapped-view protection, and allocation constraints. Build:

```sh
cargo xwin test --locked --release -p victory-hle -p victory-gpu \
  --target x86_64-pc-windows-msvc --no-run
```

Run the resulting `victory_hle-*.exe` and `victory_gpu-*.exe` test binaries
in the Mac bottle with the `rosetta` filter and AVX enabled. The
`virtual_alloc_meets_console_constraints` test also checks that the Mac
allocations are not write-watched. Fault-tracking-specific tests are
skipped on this host because that optimization is deliberately disabled.
