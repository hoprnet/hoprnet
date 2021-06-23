#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="install-websocat"
source "${mydir}/utils.sh"

declare exit_code=0

which websocat > /dev/null || exit_code=$?

if [ "${exit_code}" = "0" ]; then
  log "websocat binary found"
  exit 0
fi

declare kernel arch download_url bin_path

bin_path="${mydir}/../.bin"
download_url=https://github.com/vi/websocat/releases/download/v1.8.0/websocat
kernel=$(uname -s)
arch=$(uname -m)

case "${arch}" in
  "x86_32")
    arch="i386"
    ;;
  "x86_64")
    arch="amd64"
    ;;
  "arm")
    ;;
  *)
    log "no websocat binary available for architecture ${arch}"
    exit 1
    ;;
esac

if [ "${kernel}" = "Linux" ]; then
  download_url="${download_url}_${arch}-${kernel,,}-static"
elif [ "${kernel}" = "Darwin" ]; then
  download_url="${download_url}_mac"
else
  log "cannot install websocat binary for unsupported platform ${kernel} ${arch}"
fi

log "websocat binary not found, trying to install ${kernel} ${arch} version"
mkdir -p "${bin_path}"
curl -sL "${download_url}" > "${bin_path}/websocat"
chmod +x "${bin_path}/websocat"
