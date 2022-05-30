# Build all hopr packages. This image is meant to be used as a base for
# Docker-based development or utility Docker containers. It doesn't run anything
# itself if started.

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:380112b3f3df96ae593b0b89f4c44e501e9ae0ec1580b270b50f8ac52688256e as build

# python is used by some nodejs dependencies as an installation requirement
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     git \
     python3 \
     build-essential \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

WORKDIR /app

ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_OPTIONS=--max_old_space_size=4096

RUN yarn && yarn build && yarn test

ENTRYPOINT ["npx", "ts-node"]
