# The voice plugin's servers for clip 08 (a copy of the voice worktree's voice-servers.ps1: this
# machine's paths). Starts what isn't up: m8's stt-server (Parakeet, with word times) on 8083 and
# Breeze (the cloned voices: the C++ one, Breeze-TTS-2.cpp's breeze-server built for CUDA, Q8_0) on
# 7880, waits for both, and turns RustVR's Voice setting on. The DeepSeek key is the one in
# %LOCALAPPDATA%\RustVR\voice.toml (api_key), or DEEPSEEK_API_KEY.
$ErrorActionPreference = "Continue"
function Up($port) {
    try { Invoke-RestMethod "http://127.0.0.1:$port/health" -TimeoutSec 1 | Out-Null; $true } catch { $false }
}

$config = "$env:LOCALAPPDATA\RustVR\voice.toml"
$inConfig = (Test-Path $config) -and ((Get-Content $config -Raw) -match '(?m)^\s*api_key\s*=\s*"[^"\s]+"')
if (-not $inConfig -and -not $env:DEEPSEEK_API_KEY) {
    "No DeepSeek key yet. Paste it into api_key in $config"
    "(or set it with:  setx DEEPSEEK_API_KEY <your key>, then a new Explorer window) and run this .bat again."
    exit 1
}
if (-not (Up 8083)) {
    "starting speech to text (Parakeet) on 8083"
    Start-Process -WindowStyle Minimized -FilePath "<builds>\voice\m8-target\release\stt-server.exe" `
        -ArgumentList '"<voice models>\models\sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8"', '8083'
}
if (-not (Up 7880)) {
    "starting Breeze (the voices) on 7880: about 10 seconds"
    Start-Process -WindowStyle Minimized -WorkingDirectory "<builds>\voice\breeze-cpp" -FilePath "<builds>\voice\breeze-cpp\src\build-cuda\Release\breeze-server.exe" `
        -ArgumentList 'models\breeze-tts-2-q8_0.gguf', '--host', '127.0.0.1', '--port', '7880'
}
$t0 = Get-Date
while (-not ((Up 8083) -and (Up 7880))) {
    if (((Get-Date) - $t0).TotalMinutes -gt 8) { "the voice servers didn't come up in 8 minutes"; exit 1 }
    Start-Sleep 3
}
"speech to text and the voices are up"

# A first line through each, so the first line in the game isn't the one that warms them up (Breeze
# takes ~10 s over its first): your own clip from m8's voices, and its text.
$clip = "<voice models>\voices\soldier.wav"
if (Test-Path $clip) {
    "warming them up"
    $said = (Get-Content "<voice models>\voices\soldier.txt" -Raw).Trim()
    curl.exe -s -o NUL -F "text=Well met." -F "ref_text=$said" -F "seed=42" -F "ref_audio=@$clip;type=audio/wav" http://127.0.0.1:7880/v1/audio/speech
    curl.exe -s -o NUL -F "response_format=verbose_json" -F "file=@$clip;type=audio/wav" http://127.0.0.1:8083/v1/audio/transcriptions
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
