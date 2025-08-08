#!/usr/bin/env bash
set -Eeo pipefail
set -o errtrace

trap 'echo "Error occurred during package installation. Pausing for manual inspection..."; sleep 60' ERR

#set -x
DISTRIBUTION="${1:?Error: DISTRIBUTION parameter is required}"
export HOPRD_PASSWORD="$2"
export HOPRD_SAFE_ADDRESS="$3"
export HOPRD_MODULE_ADDRESS="$4"
export HOPRD_PROVIDER="$5"

# Install the package based on the distribution
case "$DISTRIBUTION" in
deb)
  sudo apt-get update
  sudo -E apt install -y "/tmp/hoprd.${DISTRIBUTION}"
  ;;
rpm)
  sudo -E dnf install -y "/tmp/hoprd.${DISTRIBUTION}"
  ;;
archlinux)
  # Archlinux mirrors conf in the GCP image is outdated by default
  sudo tee /etc/pacman.conf <<EOF
[options]
Architecture = auto
CheckSpace
SigLevel = Never

[core]
Include = /etc/pacman.d/mirrorlist

[extra]
Include = /etc/pacman.d/mirrorlist
EOF
  sudo pacman -Syy
  sudo -E pacman --noconfirm -U "/tmp/hoprd.${DISTRIBUTION}" # --verbose --debug
  ;;
*)
  echo "Unsupported distribution: $DISTRIBUTION"
  exit 1
  ;;
esac

# Check the health status of the hoprd service
if systemctl is-active --quiet hoprd; then
  echo "hoprd service is running."
else
  echo "hoprd service is not running."
  exit 1
fi
