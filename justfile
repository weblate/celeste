get-version:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version'

update-versions:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(just get-version)"
    sed -i "s|pkgver=.*|pkgver=${version}|" makedeb/PKGBUILD

create-flatpak:
    #!/usr/bin/env bash
    cd "$(git rev-parse --show-toplevel)/flatpak"
    flatpak-builder build-dir/ com.hunterwittenborn.Celeste.yml --force-clean