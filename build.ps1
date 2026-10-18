param([switch]$Test, [switch]$NoDeploy, [string]$ModsDir,
      [string]$BbportDir, [string]$ClangBin = 'C:\msys64\clang64\bin')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = $PSScriptRoot
Set-Location $root
if (Test-Path $ClangBin) { $env:PATH = "$ClangBin;$env:PATH" }
function Run([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed ($LASTEXITCODE)" }
}
Run cargo @('build', '--release', '--locked')
if ($Test) {
    Run cargo @('test', '--locked')
    $metadataJson = & cargo metadata --format-version 1 --locked
    if ($LASTEXITCODE -ne 0) { throw 'Cannot resolve SDK source.' }
    $sdk = ($metadataJson | ConvertFrom-Json).packages | Where-Object name -eq 'bbmod'
    $bb = [IO.Path]::GetFullPath((Join-Path (Split-Path $sdk.manifest_path -Parent) '../../..'))
    $out = Join-Path $root 'out'
    $hostBuild = "$out/host"
    Run cmake @('-S', $bb, '-B', $hostBuild, '-G', 'Ninja', '-DCMAKE_BUILD_TYPE=Release',
        '-DCMAKE_C_COMPILER=clang', '-DCMAKE_CXX_COMPILER=clang++',
        '-DBBMOD_BUILD_TESTS=OFF', '-DBBMOD_WITH_ZYDIS=OFF')
    Run cmake @('--build', $hostBuild, '--parallel', '4')
    $includes = @("-I$bb/host/include", "-I$bb/sdk/include", "-I$bb/host/src")
    foreach ($fixture in @(@('pad', "$bb/tests/test_pad.c"), @('mod-integration', "$root/tests/mod-integration.c"))) {
        Run clang (@('-std=c11', '-g', '-UNDEBUG') + $includes + @($fixture[1],
            "$hostBuild/libbbmod_host.a", '-lbcrypt', '-o', "$out/test-$($fixture[0]).exe"))
    }
    $fixtureRoot = Join-Path $out ('integration-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force "$fixtureRoot/bbmods/bbmod.dualsense", "$fixtureRoot/config" | Out-Null
    Copy-Item target/release/bbmod_dualsense.dll, mod.toml -Destination "$fixtureRoot/bbmods/bbmod.dualsense"
    Push-Location $fixtureRoot
    try {
        Run "$out/test-pad.exe" @()
        Run "$out/test-mod-integration.exe" @("$fixtureRoot/bbmods")
    } finally { Pop-Location }
    if ($BbportDir) {
        $port = (Resolve-Path $BbportDir).Path
        Run clang (@('-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread', "-I$port", "-I$port/src") +
            $includes + @("$root/tests/runtime-pad.c", "$port/src/runtime_input_config.c",
            '-lSDL3', '-o', "$out/test-runtime-pad.exe"))
        Run "$out/test-runtime-pad.exe" @()
    }
}
$package = Join-Path $root 'dist/bbmod.dualsense'
New-Item -ItemType Directory -Force $package | Out-Null
Copy-Item target/release/bbmod_dualsense.dll, mod.toml, README.md -Destination $package -Force
Copy-Item -LiteralPath licenses -Destination $package -Recurse -Force
Write-Host "Package: $package"
if ($ModsDir -and -not $NoDeploy) {
    $install = Join-Path ([IO.Path]::GetFullPath($ModsDir)) 'bbmod.dualsense'
    New-Item -ItemType Directory -Force $install | Out-Null
    if (-not (Test-Path "$install/mod.toml")) { Copy-Item mod.toml $install }
    $dll = Join-Path $install 'bbmod_dualsense.dll'
    $staged = Join-Path $install ('bbmod_dualsense.' + [guid]::NewGuid().ToString('N') + '.tmp')
    Copy-Item "$package/bbmod_dualsense.dll" $staged
    if (Test-Path $dll) {
        [IO.File]::Replace($staged, $dll, "$dll.previous")
    } else { [IO.File]::Move($staged, $dll) }
    Write-Host "Installed: $dll (existing settings preserved)"
}
