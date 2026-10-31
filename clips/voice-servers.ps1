# The voice plugin's servers (docs/playtest/voice.md), for clip 08 or any run with Voice on. Starts
# what isn't up: m8's stt-server (speech to text: Parakeet, with word times) and Breeze (the cloned
# voices), on the ports %LOCALAPPDATA%\RustVR\voice.toml calls them on, waits for both, and turns
# RustVR's Voice setting on. Where the servers and their models are is this machine's own: the
# [voice] lines of %LOCALAPPDATA%\RustVR\dev.toml (tools\dev.example.toml). The LLM's key is yours,
# in voice.toml (api_key) or the environment variable it names (api_key_env); never in a repository.
$ErrorActionPreference = "Continue"
. "$PSScriptRoot\..\tools\paths.ps1"
function Up($port) {
    try { Invoke-RestMethod "http://127.0.0.1:$port/health" -TimeoutSec 1 | Out-Null; $true } catch { $false }
}

# The plugin's config: the servers' ports, and the LLM's key.
$config = "$env:LOCALAPPDATA\RustVR\voice.toml"
$text = if (Test-Path $config) { Get-Content $config -Raw } else { "" }
function Port($section, $key, $default) {
    if ($text -match "(?ms)^\[$section\][^\[]*?^\s*$key\s*=\s*`"[a-z]+://[^:/`"]+:(\d+)") { [int]$Matches[1] } else { $default }
}
$sttPort = Port "stt" "url" 8083
$ttsPort = Port "voice" "server" 7860
if ($text) {
    $keyed = $text -match '(?m)^\s*api_key\s*=\s*"[^"\s]+"'
    $envName = if ($text -match '(?m)^\s*api_key_env\s*=\s*"([^"]+)"') { $Matches[1] } else { "" }
    if (-not $keyed -and $envName -and -not [Environment]::GetEnvironmentVariable($envName)) {
        "No LLM key yet. Paste it into api_key in $config"
        "(or set it with:  setx $envName <your key>, then a new Explorer window) and run this .bat again."
        exit 1
    }
} else {
    "No $config yet: the voice plugin writes one the first time it starts. Fill in its [llm] then."
}

if (-not (Up $sttPort)) {
    $exe = Get-DevSetting "stt_server" "voice"
    $model = Get-DevSetting "stt_model" "voice"
    if (-not $exe) { "speech to text isn't up on ${sttPort}: set stt_server and stt_model under [voice] in $DevConfig"; exit 1 }
    "starting speech to text (Parakeet) on $sttPort"
    Start-Process -WindowStyle Minimized -FilePath $exe -ArgumentList "`"$model`"", "$sttPort"
}
if (-not (Up $ttsPort)) {
    $exe = Get-DevSetting "tts_server" "voice"
    $dir = Get-DevSetting "tts_dir" "voice"
    $model = Get-DevSetting "tts_model" "voice"
    if (-not $exe) { "Breeze isn't up on ${ttsPort}: set tts_server, tts_dir and tts_model under [voice] in $DevConfig"; exit 1 }
    if (-not $dir) { $dir = Split-Path -Parent $exe }
    "starting Breeze (the voices) on ${ttsPort}: about 10 seconds"
    Start-Process -WindowStyle Minimized -WorkingDirectory $dir -FilePath $exe `
        -ArgumentList $model, '--host', '127.0.0.1', '--port', "$ttsPort"
}
$t0 = Get-Date
while (-not ((Up $sttPort) -and (Up $ttsPort))) {
    if (((Get-Date) - $t0).TotalMinutes -gt 8) { "the voice servers didn't come up in 8 minutes"; exit 1 }
    Start-Sleep 3
}
"speech to text and the voices are up"

# A first line through each, so the first line in the game isn't the one that warms them up (Breeze
# takes ~10 s over its first): your own voice clip, and its text (the .txt beside it).
$clip = Get-DevSetting "warm_up" "voice"
if ($clip -and (Test-Path $clip)) {
    "warming them up"
    $said = (Get-Content ([IO.Path]::ChangeExtension($clip, ".txt")) -Raw -ErrorAction SilentlyContinue)
    $said = if ($said) { $said.Trim() } else { "" }
    curl.exe -s -o NUL -F "text=Well met." -F "ref_text=$said" -F "seed=42" -F "ref_audio=@$clip;type=audio/wav" "http://127.0.0.1:$ttsPort/v1/audio/speech"
    curl.exe -s -o NUL -F "response_format=verbose_json" -F "file=@$clip;type=audio/wav" "http://127.0.0.1:$sttPort/v1/audio/transcriptions"
}

# RustVR's Voice setting (Settings, under the sounds: "Talk to the game's characters out loud").
$settings = "$env:LOCALAPPDATA\RustVR\settings.toml"
$text = if (Test-Path $settings) { Get-Content $settings -Raw } else { "" }
if ($text -match '(?m)^\[voice\]') {
    $text = $text -replace '(?ms)(^\[voice\][^\[]*?enabled\s*=\s*)false', '${1}true'
} else {
    $text = $text.TrimEnd() + "`r`n`r`n[voice]`r`nenabled = true`r`n"
}
Set-Content -Path $settings -Value $text -NoNewline
"RustVR's Voice is on"
exit 0
