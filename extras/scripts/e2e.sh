#!/bin/bash
# End-to-end tests in a fake root (bwrap user namespace). Real system untouched.
#   e2e.sh helper        run the real root helper, check results
#   e2e.sh gui [page]    open the real GUI on the fake root, pkexec stubbed
#   e2e.sh deb FILE [page]  extract the .deb, run ITS app + helper on the fake root
# Needs: cargo build (debug), bubblewrap.
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
HELPER=$ROOT/target/debug/latch-helper
APP=$ROOT/target/debug/latch
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
mkdir -p "$T/fake" "$T/stub" "$T/home"

fake() { printf '%s\n' '#!/bin/sh' "$2" > "$T/fake/$1"; chmod 755 "$T/fake/$1"; }
fake wine 'case "$1" in --version) echo wine-fake;; *) echo latch-ok;; esac'
fake xdotool 'echo "x:1 y:2 screen:0 window:3"'
echo "ENABLED=yes" > "$T/fake/ufw.conf"
fake ufw "echo \"\$@\" >> $T/ufw.log
case \"\$*\" in *enable) echo ENABLED=yes > $T/fake/ufw.conf;; disable) echo ENABLED=no > $T/fake/ufw.conf;; esac"
printf '#!/bin/sh\nexec "$@"\n' > "$T/stub/pkexec"
chmod 755 "$T/stub/pkexec"

BIND=(--bind "$T/fake/ufw" /usr/sbin/ufw --bind "$T/fake/ufw.conf" /etc/ufw/ufw.conf)
for bin in /usr/bin/wine /usr/bin/wine64 /usr/bin/xdotool; do
  [ -e "$bin" ] || continue
  src=wine; [ "$bin" = /usr/bin/xdotool ] && src=xdotool
  BIND+=(--bind "$T/fake/$src" "$(readlink -f "$bin")")
done

EXTRA=()
sandbox() {
  bwrap --unshare-user --uid 0 --gid 0 --ro-bind / / --dev /dev --proc /proc \
    --bind "$T" "$T" --tmpfs /usr/libexec --tmpfs /usr/share/polkit-1/actions \
    "${BIND[@]}" "${EXTRA[@]}" --setenv HOME "$T/home" --setenv PATH "$T/stub:$PATH" "$@"
}

INSIDE='
H=$1; T=$2
"$H" install; echo "K:install_exit=$?"
echo "K:copies=$(ls /usr/libexec/latch | wc -l)"
echo "K:copy_mode=$(stat -c %a /usr/libexec/latch/wine-enable)"
echo "K:policy_mode=$(stat -c %a /usr/share/polkit-1/actions/org.latch.policy)"
W=$(readlink -f /usr/bin/wine); X=$(readlink -f /usr/bin/xdotool)
/usr/libexec/latch/wine-disable;    echo "K:wine_locked=$(stat -c %a $W)"
/usr/libexec/latch/wine-enable;     echo "K:wine_unlocked=$(stat -c %a $W)"
/usr/libexec/latch/xdotool-disable; echo "K:xdo_locked=$(stat -c %a $X)"
/usr/libexec/latch/xdotool-enable;  echo "K:xdo_unlocked=$(stat -c %a $X)"
/usr/libexec/latch/firewall-disable; echo "K:fw_conf_off=$(cat "$T/fake/ufw.conf")"
/usr/libexec/latch/firewall-enable;  echo "K:fw_conf_on=$(cat "$T/fake/ufw.conf")"
echo "K:ufw_calls=$(tr "\n" "," < "$T/ufw.log")"
cp "$H" "$T/bogus"; "$T/bogus" 2>/dev/null; echo "K:bad_name_exit=$?"
"$H" frobnicate 2>/dev/null; echo "K:bad_arg_exit=$?"
/usr/libexec/latch/latch-helper uninstall; echo "K:left=$(ls /usr/libexec/latch 2>/dev/null | wc -l)"
echo "K:policy_gone=$([ -e /usr/share/polkit-1/actions/org.latch.policy ] && echo no || echo yes)"
'

run_helper() {
  local out pass=0 fail=0 k
  declare -A got
  out=$(sandbox bash -c "$INSIDE" _ "$HELPER" "$T")
  while IFS= read -r line; do
    case "$line" in K:*) k=${line#K:}; got[${k%%=*}]=${k#*=};; esac
  done <<< "$out"
  expect() {
    if [ "${got[$1]:-<none>}" = "$2" ]; then echo "ok    $1 = $2"; pass=$((pass + 1))
    else echo "FAIL  $1: got '${got[$1]:-<none>}', want '$2'"; fail=$((fail + 1)); fi
  }
  expect install_exit 0;  expect copies 7;          expect copy_mode 755;   expect policy_mode 644
  expect wine_locked 700; expect wine_unlocked 755; expect xdo_locked 700;  expect xdo_unlocked 755
  expect fw_conf_off ENABLED=no; expect fw_conf_on ENABLED=yes
  expect ufw_calls "disable,--force enable,"
  expect bad_name_exit 2; expect bad_arg_exit 2;    expect left 0;          expect policy_gone yes
  echo "-- $pass passed, $fail failed"
  [ "$fail" -eq 0 ]
}

case "${1:-}" in
  helper) run_helper ;;
  gui) shift; sandbox "$APP" "$@" ;;
  deb)
    DEB=${2:?path to .deb}; shift 2
    mkdir -p "$T/deb" && dpkg-deb -x "$DEB" "$T/deb" || exit 1
    EXTRA=(--bind "$T/deb/usr/libexec/latch" /usr/libexec/latch
           --bind "$T/deb/usr/share/polkit-1/actions/org.latch.policy" /usr/share/polkit-1/actions/org.latch.policy)
    sandbox "$T/deb/usr/bin/latch" "$@" ;;
  *) echo "usage: e2e.sh helper | gui [page] | deb FILE [page]"; exit 2 ;;
esac
