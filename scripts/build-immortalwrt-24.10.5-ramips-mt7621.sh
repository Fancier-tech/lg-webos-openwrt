#!/usr/bin/env bash
set -euo pipefail

SDK_URL="https://downloads.immortalwrt.org/releases/24.10.5/targets/ramips/mt7621/immortalwrt-sdk-24.10.5-ramips-mt7621_gcc-13.3.0_musl.Linux-x86_64.tar.zst"
SDK_SHA256="db8d13635f083c77e32895e3a953cfb6511aa8eff4c75ad3fd0bf831790fc2df"
WORKDIR="${1:-$HOME/immortalwrt-lgtvctl-build}"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCHIVE="$WORKDIR/$(basename "$SDK_URL")"

mkdir -p "$WORKDIR"
cd "$WORKDIR"

if [ ! -f "$ARCHIVE" ]; then
  echo "Downloading ImmortalWrt SDK..."
  curl -L "$SDK_URL" -o "$ARCHIVE"
fi

echo "${SDK_SHA256}  $ARCHIVE" | sha256sum -c -

SDK_DIR="$(find "$WORKDIR" -maxdepth 1 -type d -name 'immortalwrt-sdk-24.10.5-ramips-mt7621*' | head -n1 || true)"
if [ -z "$SDK_DIR" ]; then
  echo "Extracting SDK..."
  tar --use-compress-program=unzstd -xf "$ARCHIVE" -C "$WORKDIR"
  SDK_DIR="$(find "$WORKDIR" -maxdepth 1 -type d -name 'immortalwrt-sdk-24.10.5-ramips-mt7621*' | head -n1)"
fi

cd "$SDK_DIR"

./scripts/feeds update -a
./scripts/feeds install rust
./scripts/feeds install openssl

# Fix Rust CI LLVM 404 in GitHub Actions
sed -i 's/--set=llvm\.download-ci-llvm=true/--set=llvm.download-ci-llvm=false/' feeds/packages/lang/rust/Makefile
grep -n 'download-ci-llvm' feeds/packages/lang/rust/Makefile

rm -rf package/lgtvctl
mkdir -p package/lgtvctl
cp -a "$PROJECT_DIR/openwrt/package/lgtvctl/." package/lgtvctl/

make defconfig
make package/lgtvctl/compile V=s

find bin/packages -type f -name 'lgtvctl_*.ipk' -print
