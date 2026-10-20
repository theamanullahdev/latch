#!/bin/bash
# Build or update a signed apt repository (suite stable, component main, amd64).
#   apt-repo.sh REPO_DIR DEB...
# Signs with the first secret key in the keyring, or GPG_KEY_ID if set.
# Old .deb files already in REPO_DIR/pool stay, so older versions remain installable.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)

REPO=${1:?repo dir}; shift
[ "$#" -gt 0 ] || { echo "give at least one .deb" >&2; exit 2; }

KEY=${GPG_KEY_ID:-$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ {print $10; exit}')}
[ -n "$KEY" ] || { echo "no GPG secret key found" >&2; exit 1; }

mkdir -p "$REPO/pool/main/l/latch" "$REPO/dists/stable/main/binary-amd64"
cp -- "$@" "$REPO/pool/main/l/latch/"
cd "$REPO"

BIN=dists/stable/main/binary-amd64
apt-ftparchive packages pool > "$BIN/Packages"
gzip -9nkf "$BIN/Packages"
xz -9kf "$BIN/Packages"

CONF=$(mktemp)
trap 'rm -f "$CONF"' EXIT
cat > "$CONF" <<'EOF'
APT::FTPArchive::Release::Origin "Latch";
APT::FTPArchive::Release::Label "Latch";
APT::FTPArchive::Release::Suite "stable";
APT::FTPArchive::Release::Codename "stable";
APT::FTPArchive::Release::Architectures "amd64";
APT::FTPArchive::Release::Components "main";
APT::FTPArchive::Release::Description "Latch apt repository";
EOF
apt-ftparchive -c "$CONF" release dists/stable > dists/stable/Release

gpg --batch --yes --default-key "$KEY" --clearsign -o dists/stable/InRelease dists/stable/Release
gpg --batch --yes --default-key "$KEY" --armor --detach-sign -o dists/stable/Release.gpg dists/stable/Release
gpg --batch --yes --armor --export "$KEY" > key.asc
touch .nojekyll
# Landing page with the install commands.
INDEX="$HERE/../packaging/apt-index.html"
[ -f "$INDEX" ] && cp "$INDEX" index.html || true
echo "repo ready: $(ls pool/main/l/latch | wc -l) package file(s), signed by ${KEY: -16}"
