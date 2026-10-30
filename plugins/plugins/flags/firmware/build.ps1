<#
Builds the PitFlag firmware images with arduino-cli: one image per chip, into dist/.

  .\build.ps1 -Setup            # once: get arduino-cli, the pinned cores and the pinned library
  .\build.ps1                   # build every image and check its size
  .\build.ps1 -Only atmega168   # build one image

arduino-cli and its data live in $env:PITFLAG_ARDUINO_HOME (default D:\tools\arduino-cli).
Works on Windows PowerShell 5.1. Keep this file ASCII: 5.1 reads a file without a BOM as ANSI.
#>
param([switch]$Setup, [string[]]$Only)

$ErrorActionPreference = 'Stop'

$CliVersion = '1.1.1'
$Esp32Index = 'https://espressif.github.io/arduino-esp32/package_esp32_index.json'
$Cores = @('arduino:avr@1.8.6', 'esp32:esp32@3.0.7')
$Libraries = @('Adafruit NeoPixel@1.12.3')

# The build fails above these shares of the board's flash and RAM.
$FlashLimit = 90
$RamLimit = 75

# Heap: bytes the sketch allocates at start-up, which the compiler does not count as RAM.
# Adafruit NeoPixel allocates 3 bytes per LED (64 LEDs) plus the 2-byte malloc header on AVR.
$Targets = @(
    @{ Image = 'atmega328p'; Fqbn = 'arduino:avr:nano:cpu=atmega328'; Ext = 'hex'; Heap = 194 },
    @{ Image = 'atmega168';  Fqbn = 'arduino:avr:nano:cpu=atmega168'; Ext = 'hex'; Heap = 194 },
    @{ Image = 'atmega32u4'; Fqbn = 'arduino:avr:leonardo';           Ext = 'hex'; Heap = 194 },
    # Trim: the plugin carries these images and PitCrew writes them, so the 0xFF padding is cut off.
    @{ Image = 'esp32';      Fqbn = 'esp32:esp32:esp32';              Ext = 'bin'; Heap = 192; Trim = $true },
    @{ Image = 'esp32c3';    Fqbn = 'esp32:esp32:esp32c3:CDCOnBoot=cdc'; Ext = 'bin'; Heap = 192; Trim = $true }
)

# Cuts the trailing 0xFF bytes of a merged ESP32 image, keeping whole 4 KB flash sectors.
function Save-Trimmed([string]$src, [string]$dst) {
    $b = [IO.File]::ReadAllBytes($src)
    $end = $b.Length
    while ($end -gt 0 -and $b[$end - 1] -eq 0xFF) { $end-- }
    $end = [math]::Min($b.Length, [math]::Ceiling($end / 4096) * 4096)
    $out = New-Object byte[] $end
    [Array]::Copy($b, $out, $end)
    [IO.File]::WriteAllBytes($dst, $out)
}

$Home_ = if ($env:PITFLAG_ARDUINO_HOME) { $env:PITFLAG_ARDUINO_HOME } else { 'D:\tools\arduino-cli' }
$Cli = Join-Path $Home_ 'arduino-cli.exe'
$Config = Join-Path $Home_ 'arduino-cli.yaml'
$Sketch = Join-Path $PSScriptRoot 'pitflag'
$Dist = Join-Path $PSScriptRoot 'dist'
$Work = Join-Path $PSScriptRoot 'build'

function Invoke-Cli {
    & $Cli --config-file $Config @args
    if ($LASTEXITCODE -ne 0) { throw "arduino-cli $($args -join ' ') failed with exit code $LASTEXITCODE" }
}

if ($Setup) {
    New-Item -ItemType Directory -Force $Home_ | Out-Null
    if (-not (Test-Path $Cli)) {
        $zip = Join-Path $Home_ "arduino-cli_$CliVersion.zip"
        $url = "https://downloads.arduino.cc/arduino-cli/arduino-cli_${CliVersion}_Windows_64bit.zip"
        Write-Host "Downloading $url"
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $zip
        Expand-Archive -Force $zip $Home_
        Remove-Item $zip
    }
    $h = $Home_.Replace('\', '/')
    @(
        'board_manager:',
        '  additional_urls:',
        "    - $Esp32Index",
        'directories:',
        "  data: $h/data",
        "  downloads: $h/staging",
        "  user: $h/user",
        'updater:',
        '  enable_notification: false'
    ) | Set-Content -Encoding Ascii $Config
    Invoke-Cli version
    Invoke-Cli core update-index
    foreach ($c in $Cores) { Invoke-Cli core install $c }
    foreach ($l in $Libraries) { Invoke-Cli lib install $l }
    Write-Host 'Setup done.'
    exit 0
}

if (-not (Test-Path $Cli)) { throw "arduino-cli is not at $Cli. Run .\build.ps1 -Setup first." }

New-Item -ItemType Directory -Force $Dist, $Work | Out-Null
$report = @()
$failed = @()

foreach ($t in $Targets) {
    if ($Only -and ($Only -notcontains $t.Image)) { continue }
    $buildPath = Join-Path $Work $t.Image
    $outDir = Join-Path $buildPath 'out'
    Write-Host "== $($t.Image) ($($t.Fqbn))"
    $json = & $Cli --config-file $Config compile --fqbn $t.Fqbn --build-path $buildPath --output-dir $outDir --warnings default --json $Sketch
    $code = $LASTEXITCODE
    $r = ($json -join "`n") | ConvertFrom-Json
    if ($code -ne 0 -or -not $r.success) {
        Write-Host $r.compiler_out
        Write-Host $r.compiler_err
        $failed += "$($t.Image): compile failed (exit $code)"
        continue
    }

    $sections = $r.builder_result.executable_sections_size
    $flash = $sections | Where-Object { $_.name -eq 'text' }
    $ram = $sections | Where-Object { $_.name -eq 'data' }
    $flashPct = [math]::Round(100.0 * $flash.size / $flash.max_size, 1)
    $ramUsed = $ram.size + $t.Heap
    $ramPct = [math]::Round(100.0 * $ramUsed / $ram.max_size, 1)
    $line = '{0,-11} flash {1,7} / {2,7} B ({3,5}%)   RAM {4,6} / {5,6} B ({6,5}%, heap included)' -f $t.Image, $flash.size, $flash.max_size, $flashPct, $ramUsed, $ram.max_size, $ramPct
    Write-Host $line
    $report += $line
    if ($flashPct -gt $FlashLimit) { $failed += "$($t.Image): flash $flashPct% is above $FlashLimit%" }
    if ($ramPct -gt $RamLimit) { $failed += "$($t.Image): RAM $ramPct% is above $RamLimit%" }

    $src = if ($t.Ext -eq 'hex') { Join-Path $outDir 'pitflag.ino.hex' } else { Join-Path $outDir 'pitflag.ino.merged.bin' }
    if (-not (Test-Path $src)) {
        $failed += "$($t.Image): $src was not produced"
        continue
    }
    $dst = Join-Path $Dist "pitflag-$($t.Image).$($t.Ext)"
    if ($t.Trim) { Save-Trimmed $src $dst } else { Copy-Item -Force $src $dst }
}

$report | Set-Content -Encoding Ascii (Join-Path $Dist 'size-report.txt')

if ($failed.Count -gt 0) {
    Write-Host ''
    $failed | ForEach-Object { Write-Host "FAILED $_" }
    exit 1
}
Write-Host ''
Write-Host "Images are in $Dist"
exit 0
