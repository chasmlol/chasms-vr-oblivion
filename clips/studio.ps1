# Sets the RustVR settings a trailer shot wants (the desktop view's Studio look, its field of view,
# anything else), and puts the player's own back afterwards. The runtime reads settings.toml as it
# starts and again whenever the file changes.
#
#   powershell -File clips\studio.ps1 -Look cinematic [-Set "spectator.fov=100;studio.grain=0"]
#   powershell -File clips\studio.ps1 -Restore
#
# Looks: clean, cinematic, hyper_real, action, vintage (the Studio tab's, value for value, from
# framework/crates/runtime/src/spectator_fx.rs `look`). -Set takes "section.key=value" pairs (a bare
# key is the studio's); a value is written as TOML, so a string needs its quotes ('grade="noir"').
# The player's settings.toml is copied aside (settings.before-clip.toml, next to it) the first time
# and only put back by -Restore, so a second shot set up before the first was put back still
# restores the player's own; each shot starts from the player's own, not the last shot's.
param([string]$Look = "", [string]$Set = "", [switch]$Restore)
$ErrorActionPreference = "Stop"
$dir = "$env:LOCALAPPDATA\RustVR"
$file = "$dir\settings.toml"
$aside = "$dir\settings.before-clip.toml"
$none = "$dir\settings.before-clip.none"

if ($Restore) {
    if (Test-Path $aside) {
        Copy-Item $aside $file -Force
        Remove-Item $aside -Force
        "your RustVR settings are back"
    } elseif (Test-Path $none) {
        Remove-Item $file -Force -ErrorAction SilentlyContinue
        Remove-Item $none -Force
        "your RustVR settings are back (there were none)"
    }
    exit 0
}

New-Item -ItemType Directory -Force $dir | Out-Null
if (-not (Test-Path $aside) -and -not (Test-Path $none)) {
    if (Test-Path $file) { Copy-Item $file $aside } else { Set-Content $none "" }
}

# The Studio's defaults (the clean look), in the order the runtime writes them.
$clean = [ordered]@{
    look = '"clean"'; smooth_snaps = 'true'; snap_speed = '160.0'; fill_edges = 'true'; flat_hud = 'false'; flat_hud_size = '1.0'
    motion_blur = 'false'; shutter = '180.0'; recorded_fps = '60.0'; camera_blur = '1.0'; centre_sharp = '0.5'; object_blur = '1.0'
    speed_fov = 'false'; speed_fov_deg = '10.0'; speed_from = '3.0'; speed_full = '9.0'
    shake = 'false'; shake_strength = '1.0'; handheld = '0.0'
    dof = 'false'; dof_blur = '1.0'; dof_near = '0.0'; focus_seconds = '0.3'
    lens_fov = '0.0'; fisheye = '0.0'; chromatic = '0.0'; vignette = '0.0'; glow = '0.0'; grain = '0.0'; sharpen = '0.0'
    exposure = '0.0'; contrast = '1.0'; saturation = '1.0'; temperature = '0.0'; tint = '0.0'
    grade = '"none"'; grade_mix = '1.0'; lut = '""'; lut_mix = '1.0'; chest_mount = '0.0'
}
$looks = @{
    clean = @{}
    cinematic = @{
        motion_blur = 'true'; shutter = '180.0'; recorded_fps = '24.0'; camera_blur = '0.8'; centre_sharp = '0.6'; object_blur = '1.0'
        speed_fov = 'true'; speed_fov_deg = '8.0'; shake = 'true'; shake_strength = '0.6'; handheld = '0.25'
        dof = 'true'; dof_blur = '1.2'; dof_near = '0.3'; focus_seconds = '0.4'
        lens_fov = '90.0'; chromatic = '0.15'; vignette = '0.35'; glow = '0.35'; grain = '0.12'; sharpen = '0.15'
        contrast = '1.1'; saturation = '0.95'; temperature = '0.1'; grade = '"teal_orange"'; grade_mix = '0.6'
    }
    hyper_real = @{
        motion_blur = 'true'; shutter = '300.0'; recorded_fps = '30.0'; camera_blur = '1.0'; centre_sharp = '0.15'; object_blur = '1.0'
        shake = 'true'; shake_strength = '1.0'; handheld = '0.6'
        lens_fov = '106.0'; fisheye = '0.45'; chromatic = '0.35'; vignette = '0.5'; glow = '0.15'; grain = '0.25'; sharpen = '0.6'
        exposure = '0.15'; contrast = '1.15'; saturation = '0.8'; temperature = '-0.15'; grade = '"bodycam"'; grade_mix = '0.8'
        chest_mount = '0.7'
    }
    action = @{
        motion_blur = 'true'; shutter = '180.0'; recorded_fps = '30.0'; camera_blur = '1.0'; centre_sharp = '0.5'; object_blur = '1.2'
        speed_fov = 'true'; speed_fov_deg = '12.0'; speed_from = '2.5'; speed_full = '8.0'
        shake = 'true'; shake_strength = '1.6'; handheld = '0.15'
        lens_fov = '95.0'; fisheye = '0.1'; chromatic = '0.2'; vignette = '0.3'; glow = '0.25'; grain = '0.05'; sharpen = '0.3'
        contrast = '1.2'; saturation = '1.15'; temperature = '0.05'; grade = '"teal_orange"'; grade_mix = '0.4'
    }
    vintage = @{
        motion_blur = 'true'; shutter = '180.0'; recorded_fps = '24.0'; camera_blur = '0.8'; centre_sharp = '0.3'; object_blur = '1.0'
        shake = 'true'; shake_strength = '0.4'; handheld = '0.4'
        dof = 'true'; dof_blur = '0.8'; dof_near = '0.5'; focus_seconds = '0.5'
        lens_fov = '85.0'; fisheye = '0.05'; chromatic = '0.25'; vignette = '0.6'; glow = '0.5'; grain = '0.5'
        exposure = '0.1'; contrast = '0.9'; saturation = '0.6'; temperature = '0.3'; grade = '"warm_film"'; grade_mix = '1.0'
    }
}

# The file's tables, each a list of lines ("" for what comes before the first).
$tables = [ordered]@{ "" = [System.Collections.Generic.List[string]]::new() }
$cur = ""
$src = if (Test-Path $aside) { $aside } elseif (Test-Path $file) { $file } else { $null }
if ($src) {
    foreach ($line in Get-Content $src) {
        if ($line -match '^\s*\[([^\[\]]+)\]\s*$') { $cur = $Matches[1].Trim(); $tables[$cur] = [System.Collections.Generic.List[string]]::new(); continue }
        $tables[$cur].Add($line)
    }
}
function Set-Key([string]$table, [string]$key, [string]$value) {
    if (-not $tables.Contains($table)) { $tables[$table] = [System.Collections.Generic.List[string]]::new() }
    $lines = $tables[$table]
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match "^\s*$([regex]::Escape($key))\s*=\s*(.*)$") {
            # (A whole number where the file has a decimal stays a decimal: "fov=100" is 100.0.)
            $old = $Matches[1]
            if ($value -match '^-?\d+$' -and $old -match '^-?\d+\.\d') { $value = "$value.0" }
            $lines[$i] = "$key = $value"
            return
        }
    }
    # (Before the table's trailing blank lines.)
    $at = $lines.Count
    while ($at -gt 0 -and -not $lines[$at - 1].Trim()) { $at-- }
    $lines.Insert($at, "$key = $value")
}

if ($Look) {
    $l = $Look.ToLower().Replace("-", "_")
    if ($l -eq "hyperreal") { $l = "hyper_real" }
    if (-not $looks.ContainsKey($l)) { "no look $Look (clean, cinematic, hyper_real, action, vintage)"; exit 1 }
    # The look's own values over the clean look's; the viewers' own options stay as the player has them.
    $keep = @("smooth_snaps", "snap_speed", "fill_edges", "flat_hud", "flat_hud_size")
    foreach ($k in $clean.Keys) {
        if ($keep -contains $k -and $tables.Contains("studio") -and ($tables["studio"] | Where-Object { $_ -match "^\s*$k\s*=" })) { continue }
        $v = if ($looks[$l].ContainsKey($k)) { $looks[$l][$k] } else { $clean[$k] }
        Set-Key "studio" $k $v
    }
    Set-Key "studio" "look" "`"$l`""
}
foreach ($pair in ($Set -split ';' | ForEach-Object { $_.Trim() } | Where-Object { $_ })) {
    $k, $v = $pair -split '=', 2
    $k = $k.Trim(); $v = $v.Trim()
    $table, $key = if ($k.Contains('.')) { $k -split '\.', 2 } else { "studio", $k }
    Set-Key $table $key $v
    # (Anything set by hand in the studio makes the look a custom one, as the Studio tab has it.)
    if ($table -eq "studio" -and $key -ne "look" -and @("smooth_snaps", "snap_speed", "fill_edges", "flat_hud", "flat_hud_size") -notcontains $key) { Set-Key "studio" "look" '"custom"' }
}

$out = [System.Collections.Generic.List[string]]::new()
foreach ($t in $tables.Keys) {
    if ($t) {
        if ($out.Count -and $out[$out.Count - 1].Trim()) { $out.Add("") }
        $out.Add("[$t]")
    }
    $out.AddRange($tables[$t])
}
New-Item -ItemType Directory -Force $dir | Out-Null
[System.IO.File]::WriteAllText($file, (($out -join "`r`n").TrimEnd() + "`r`n"))
"RustVR settings for the shot:$(if ($Look) { " the $l look" })$(if ($Set) { " ($Set)" })"
