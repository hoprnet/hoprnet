#!/bin/bash

apt-get update
apt-get -y install ca-certificates curl gnupg lsb-release net-tools iputils-ping socat traceroute

# Install NodeJS
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
sudo apt-get -y install nodejs

npm install -g npm
npm install -g yarn

# Install Docker
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

apt-get update
apt-get -y install docker-ce docker-ce-cli containerd.io

# Install docker-compose
curl -sSL "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose
ln -s /usr/local/bin/docker-compose /usr/bin/docker-compose

docker pull node:16-slim
