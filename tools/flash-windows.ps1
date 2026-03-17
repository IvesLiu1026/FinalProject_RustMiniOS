param(
    [string]$ElfPath = ""
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path

if ([string]::IsNullOrWhiteSpace($ElfPath)) {
    $ElfPath = Join-Path $Root "firmware\finalproject_rustminios.elf"
}

if (-not (Test-Path $ElfPath)) {
    Write-Error "Firmware ELF not found. Pass the ELF path explicitly or run this script from a release bundle."
}

$ProbeRs = if ($env:MINIOS_PROBE_RS_BIN) {
    $env:MINIOS_PROBE_RS_BIN
} else {
    (Get-Command probe-rs -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -First 1)
}

if (-not $ProbeRs) {
    Write-Error "probe-rs not found. Install it from https://probe.rs/docs/getting-started/installation"
}

$Chip = if ($env:MINIOS_PROBE_RS_CHIP) { $env:MINIOS_PROBE_RS_CHIP } else { "STM32F407ZGTx" }

& $ProbeRs run --chip $Chip $ElfPath
