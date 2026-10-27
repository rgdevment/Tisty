param([switch]$DryRun)

$Local = $env:LOCALAPPDATA
$Roaming = $env:APPDATA
$Es = (Get-UICulture).TwoLetterISOLanguageName -eq 'es'

$ProgramFolders = @(
  (Join-Path $Local 'Programs\Tisty')
  (Join-Path $Local 'dev.rgdevment.tisty')
  (Join-Path $Roaming 'dev.rgdevment.tisty')
)

$KeptByKeys = @('secrets', 'identity')

$RegistryKeys = @(
  'HKCU:\Software\Classes\tisty'
  'HKCU:\Software\Classes\Tisty.Markdown'
  'HKCU:\Software\Classes\Tisty.Calendar'
)

$OpenWith = @(
  @('HKCU:\Software\Classes\.md\OpenWithProgids', 'Tisty.Markdown')
  @('HKCU:\Software\Classes\.markdown\OpenWithProgids', 'Tisty.Markdown')
  @('HKCU:\Software\Classes\.ics\OpenWithProgids', 'Tisty.Calendar')
  @('HKCU:\Software\Classes\.ical\OpenWithProgids', 'Tisty.Calendar')
  @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Run', 'Tisty')
)

function Say([string]$En, [string]$EsText) {
  if ($Es) { Write-Host $EsText } else { Write-Host $En }
}

function Ask([string]$En, [string]$EsText) {
  if ($Es) { Read-Host $EsText } else { Read-Host $En }
}

function YesTo([string]$Answer) { $Answer -match '^[yYsS]' }

function Remove-Trace([string]$At) {
  if (-not (Test-Path -LiteralPath $At)) { return }
  if ($DryRun) { Say "  would remove: $At" "  se borraría: $At"; return }
  Remove-Item -LiteralPath $At -Recurse -Force -ErrorAction SilentlyContinue
  if (Test-Path -LiteralPath $At) { Say "  could not remove: $At" "  no se pudo borrar: $At" }
  else { Say "  removed: $At" "  borrado: $At" }
}

function Stop-Tisty {
  Get-Process -Name 'tisty-gui', 'tisty' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
}

function Backup-Store {
  $roots = @((Join-Path $HOME '.tisty'), (Join-Path $Local 'Tisty')) | Where-Object { Test-Path -LiteralPath $_ -PathType Container }
  if (-not $roots) { return $true }
  $desk = [Environment]::GetFolderPath('Desktop')
  $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $made = @()
  foreach ($root in $roots) {
    $to = Join-Path $desk ("tisty-backup-$stamp-" + ($(if ($root.StartsWith($HOME + '\.')) { 'home' } else { 'localappdata' })) + '.zip')
    if ($DryRun) { Say "  would back up $root to $to" "  se respaldaría $root en $to"; continue }
    try {
      [System.IO.Compression.ZipFile]::CreateFromDirectory($root, $to, [System.IO.Compression.CompressionLevel]::Optimal, $true)
      $made += $to
    } catch {
      $why = $_.Exception.Message
      $made + $to | ForEach-Object { Remove-Item -LiteralPath $_ -Force -ErrorAction SilentlyContinue }
      Say 'The backup could not be made, so nothing was removed.' 'No se pudo hacer el respaldo, así que no se borró nada.'
      Write-Host "  $why"
      return $false
    }
  }
  $made | ForEach-Object { Say "Your tasks, documents and settings are kept in $_" "Tus tareas, documentos y ajustes quedan en $_" }
  $true
}

function Uninstall-Installer {
  $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Tisty'
  $said = (Get-ItemProperty -LiteralPath $key -ErrorAction SilentlyContinue).UninstallString
  if (-not $said) { return }
  $exe = $said.Trim('"')
  if (-not (Test-Path -LiteralPath $exe)) { return }
  if ($DryRun) { Say "  would run: $exe /S" "  se ejecutaría: $exe /S"; return }
  Start-Process -FilePath $exe -ArgumentList "/S _?=$(Split-Path -Parent $exe)" -Wait
  Remove-Trace (Split-Path -Parent $exe)
}

function Uninstall-Store {
  foreach ($package in @(Get-AppxPackage -Name 'rgdevment.Tisty' -ErrorAction SilentlyContinue)) {
    if ($DryRun) { Say "  would remove the Store package $($package.PackageFullName)" "  se quitaría el paquete de la Store $($package.PackageFullName)"; continue }
    Remove-AppxPackage -Package $package.PackageFullName
  }
}

function Remove-Registry {
  foreach ($key in $RegistryKeys) { Remove-Trace $key }
  foreach ($pair in $OpenWith) {
    if ($null -eq (Get-ItemProperty -LiteralPath $pair[0] -Name $pair[1] -ErrorAction SilentlyContinue)) { continue }
    if ($DryRun) { Say "  would remove: $($pair[0]) › $($pair[1])" "  se borraría: $($pair[0]) › $($pair[1])"; continue }
    Remove-ItemProperty -LiteralPath $pair[0] -Name $pair[1] -ErrorAction SilentlyContinue
    Say "  removed: $($pair[0]) › $($pair[1])" "  borrado: $($pair[0]) › $($pair[1])"
  }
}

function Remove-LocalTisty([bool]$Keys) {
  $root = Join-Path $Local 'Tisty'
  if ($Keys) { Remove-Trace $root; return }
  Get-ChildItem -LiteralPath $root -Force -ErrorAction SilentlyContinue |
    Where-Object { $KeptByKeys -notcontains $_.Name } |
    ForEach-Object { Remove-Trace $_.FullName }
}

function Show-PathLeft {
  $left = [Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -match '\\Tisty\\?$' }
  if (-not $left) { return }
  Say 'Your PATH still names a Tisty folder; take it out by hand in Environment Variables:' `
      'Tu PATH todavía nombra una carpeta de Tisty; quítala a mano en Variables de entorno:'
  $left | ForEach-Object { Write-Host "  $_" }
}

Say 'Tisty: remove every trace from this computer' 'Tisty: borrar todo rastro de este equipo'
if ($DryRun) { Say '(dry run: nothing is touched)' '(ensayo: no se toca nada)' }
Write-Host ''

$keys = $true
if (-not $DryRun) {
  Say "This computer keeps a copy of your recovery phrase. Save it first in Settings › Security if you might need it." `
      'Este equipo guarda una copia de tu frase de recuperación. Guárdala primero en Configuración › Seguridad si podrías necesitarla.'
  $keys = YesTo (Ask 'Remove it and any cloud access left? (y/N)' '¿Borrarla junto con los accesos a la nube que queden? (s/N)')
  Write-Host ''
  $sure = Ask 'Type DELETE to remove Tisty and everything it left' 'Escribe BORRAR para quitar Tisty y todo lo que dejó'
  if (@('DELETE', 'BORRAR') -notcontains $sure.Trim().ToUpper()) { Say 'Cancelled. Nothing was removed.' 'Cancelado. No se borró nada.'; exit 1 }
}

if (-not $DryRun) { Stop-Tisty }
if (-not (Backup-Store)) { exit 1 }
Uninstall-Installer
Uninstall-Store
Remove-Registry
foreach ($one in $ProgramFolders) { Remove-Trace $one }
Remove-Trace (Join-Path $HOME '.tisty')
Remove-LocalTisty $keys
Show-PathLeft

Write-Host ''
if ($DryRun) { Say 'Dry run over. Run it again without -DryRun to remove.' 'Fin del ensayo. Ejecútalo otra vez sin -DryRun para borrar.' }
else { Say 'Done. Tisty is gone from this computer.' 'Listo. Tisty ya no está en este equipo.' }
