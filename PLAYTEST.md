# Playtest: the mirror

Save: rustvr-base

`powershell -File tools\play.ps1 mirror` boots Oblivion into `rustvr-base`: Danwood, a Nord in iron
armour, in a house in Danwood. `powershell -File tools\play.ps1 -Close` puts the game away.

The heads, hair and their FaceGen morphs are already extracted into the shared `local_assets`. On
another machine, run `cargo run --release -p oblivion-assets --bin oblivion-extract -- export
meshes/characters meshes/armor meshes/clothes textures/characters`.

The research and the design are in `docs/research/mirror.md`.

(This file is finished at the end of the night: see below.)
