# Build all hopr packages. This image is meant to be used as a base for
# Docker-based development or utility Docker containers. It doesn't run anything
# itself if started.

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:380112b3f3df96ae593b0b89f4c44e501e9ae0ec1580b270b50f8ac52688256e as build

RUN apt-get update \
  && apt-get install -y \
     git \
     jq


WORKDIR /app

COPY . /app

ENV HOPR_ENVIRONMENT_ID=hardhat-localhost

RUN yarn && yarn build

ENTRYPOINT ["npx", "hardhat", "node", "--show-stack-traces", "--network", "hardhat"]
