#!/usr/bin/env bash

apt-get update
apt-get -y install apt-utils ca-certificates curl gnupg lsb-release net-tools iputils-ping socat traceroute

# Install Docker
if ! command -v docker; then
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

	apt-get update
	apt-get -y install docker-ce docker-ce-cli containerd.io
fi

declare os=$(uname -s | tr -s "[:upper:]" "[:lower:]")
declare arch=$(uname -m)
if [[ "${arch}" = *arm64* ]]; then
	arch="aarch64"
fi

# Install docker-compose
if ! command -v docker-compose; then
	curl -sSL "https://github.com/docker/compose/releases/download/v2.2.3/docker-compose-${os}-${arch}" -o /usr/local/bin/docker-compose
	chmod +x /usr/local/bin/docker-compose
	ln -s /usr/local/bin/docker-compose /usr/bin/docker-compose
fi

docker pull node:16-slim
