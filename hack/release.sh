#!/usr/bin/env bash
set -euo pipefail

version=${1}

sed -i "s/## Unreleased/## Unreleased\n\n## ${version}/" CHANGELOG.md
sed -i "s/version =.* # hack\/release.sh$/version = \"${version}\" # hack\/release.sh/" dsp/Cargo.toml
sed -i "s/version =.* # hack\/release.sh$/version = \"${version}\" # hack\/release.sh/" tests/Cargo.toml
sed -i "s/rev .*/rev \"v${version}\")/" hardware/Module.kicad_sch
sed -i "s/gr_text \"board .*\" /gr_text \"board v${version}\" /" hardware/Module.kicad_pcb
sed -i "s/rev .*/rev \"v${version}\")/" hardware/Module.kicad_pcb

make

rm -rf release
mkdir release

export CHANGES=$(awk "/## ${version}/{flag=1;next}/## */{flag=0}flag" CHANGELOG.md | awk 'NF')

envsubst < hack/release.tmpl.md > release/notes.md
