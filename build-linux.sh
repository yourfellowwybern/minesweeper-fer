#!/usr/bin/env bash
# Build & jalankan Minesweeper di Linux.
set -e

if command -v apt-get >/dev/null; then
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
elif command -v dnf >/dev/null; then
  sudo dnf install -y gcc pkg-config libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel
elif command -v pacman >/dev/null; then
  sudo pacman -S --needed --noconfirm base-devel pkgconf libx11 libxi mesa alsa-lib
else
  echo "Distro tidak dikenali: install manual gcc, pkg-config, libX11, libXi, libGL, dan ALSA (dev)."
fi

if ! command -v cargo >/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . "$HOME/.cargo/env"
fi

cargo run --release
