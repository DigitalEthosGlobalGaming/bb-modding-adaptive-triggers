# Adaptive Triggers for bbmod

DualSense adaptive trigger feedback and configurable lightbar for Bloodborne
running on a bbport build with [bbmod](https://github.com/DigitalEthosGlobalGaming/bb-modding)
integrated. Mod ID: `bbmod.dualsense`.

Phase 1 uses physical L2/R2 input: R2 resistance/charge/give and an L2 pulse/buzz.
It does not confirm attacks or shots from game state. HP lightbar mode and
gameplay-driven weapon, stamina, ammo and menu detection remain future work.

**Download and use this mod at your own risk. The bb-modding mod list does not
vet, audit or verify listed mods, and inclusion is not an endorsement.**
Hardware and gameplay validation are still pending.

## Requirements and install

- Windows x64 and a DualSense controller.
- A bbport build with bbmod ABI 1.3 pad effects **and its SDL backend** integrated.
  An unmodified bbport installation or ABI 1.2 host cannot run this mod.
- Microsoft Visual C++ x64 runtime for the MSVC DLL, if not already installed.

Download a build ZIP from this repository's Actions artifacts when available,
or build from source. Copy the complete `bbmod.dualsense/` folder into your
integrated port's `bbmods/` directory. F1 opens the Adaptive Triggers panel.
Use Mods > Reload for updates, or launch with `BB_MODS_DEV=1` for hot reload.
Check `bbmods.log` for compatibility errors. Building the mod does not upgrade
the port or add its controller backend.

## Build and contribute

Install stable Rust for Windows MSVC with Visual Studio C++ build tools.
The SDK is fetched from a pinned commit of the public framework repository;
the original local Bloodborne workspace is not required.

```powershell
git clone https://github.com/DigitalEthosGlobalGaming/bb-modding-adaptive-triggers.git
cd bb-modding-adaptive-triggers
.\build.ps1                    # build and package; no deployment by default
.\build.ps1 -Test              # also run Rust and native host/DLL tests
.\build.ps1 -ModsDir C:\bbport\bbmods # explicitly install, preserving existing settings
```

Native tests require MSYS2 CLANG64 Clang, CMake and Ninja. Override `-ClangBin`
for another location. The package is `dist/bbmod.dualsense/`. Deployment retains
the previous DLL as `bbmod_dualsense.dll.previous` and preserves `mod.toml`.
`-NoDeploy` prevents installation even when `-ModsDir` is supplied.

`cargo test --locked` runs the Rust logic checks. `build.ps1 -Test` additionally
tests host controller ownership and the real mod DLL using a fake backend:
panel preview/reset, configuration, reload/state transfer, simulated fault
cleanup and unload. CI builds this package without touching a game installation.
Optional `-Test -BbportDir <modified-port-source>` also compiles the runtime pad
packet harness against that checkout and requires SDL3. That optional check is
not part of standalone CI and needs a compatible, modified port source tree.

Submit focused pull requests with the change, compatibility impact and actual
checks run. Preserve host ownership, unload/reload cleanup and panic unwinding.
Report real hardware/gameplay evidence separately from synthetic tests.

## Configuration and limitations

Configure mapping, intensity, presets, RGB lightbar and reset from the F1 panel.
For persistence across launches, put overrides in
`config/bbmod.dualsense.toml` beside the runtime's `bbmods/` directory, using the
sections in `mod.toml`. Panel settings survive hot reload; unload/load to reread
file settings without restoring the old state. Use fixed/off lightbar modes.

Overlay capture suppresses effects. The panel's Reset triggers and disable
button releases effects; test effects are armed in the panel and run after it
closes. Raw eleven-byte effects are experimental. In-game menus are not detected
in Phase 1. Host cleanup on a normal unload is not hardware validation, and a
hard process termination cannot guarantee a controller reset.

The design record is [specification.md](specification.md); its opening draft
status predates the implementation. [Local history](docs/LOCAL-HISTORY.md)
preserves earlier workspace notes and reported checks, including paths that do
not apply to a standalone clone. Current source and CI logs define this snapshot.
