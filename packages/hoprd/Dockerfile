# Run HOPRd  within a single container.

# -- BASE STAGE --------------------------------

# Based on https://snyk.io/blog/10-best-practices-to-containerize-nodejs-web-applications-with-docker/
# From the #1 recommendation “Use explicit and deterministic Docker base image tags”

# Docker image builds are inconsistent. Just like we’re using lockfiles to get a deterministic
# npm install behavior every time we install npm packages, we’d also like to get deterministic
# docker image builds. If we build the image from node—which effectively means the node:latest
# tag—then every build will pull a newly built Docker image of node.

# Originally node:12.9.1-buster, obtained by
# $ docker pull node:12.9.1-buster
# $ docker images --digests
# and added the sha to the image.
FROM node:12.9.1-buster@sha256:966f569423c977e862b4e5b1f01234720dd212394cd13f5904d47864aece5417 AS base
WORKDIR /hoprd

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    ca-certificates \
    fuse \
    gcc \
    cmake \
    wget

ENV YARN_VERSION 1.19.2
RUN yarn policies set-version $YARN_VERSION
COPY package.json .
RUN yarn install --build-from-source

# -- BUILD STAGE --------------------------------

FROM base as build

# build `hoprd` and `hopr-admin`
COPY src src
COPY hopr-admin hopr-admin
COPY tsconfig.json tsconfig.json
RUN yarn build
RUN yarn buildAdmin

# -- RUNTIME STAGE ------------------------------\

FROM node:12.9.1-buster@sha256:966f569423c977e862b4e5b1f01234720dd212394cd13f5904d47864aece5417 AS runtime

RUN wget https://github.com/Yelp/dumb-init/releases/download/v1.2.4/dumb-init_1.2.4_amd64.deb
RUN dpkg -i dumb-init_*.deb

ENV NODE_ENV 'production'
ENV NEXT_TELEMETRY_DISABLED 1
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS=--max_old_space_size=4096

WORKDIR /app

# Server
COPY --from=build /hoprd/lib/ /app/
COPY --from=build /hoprd/hopr-admin /app/hopr-admin
COPY --from=build /hoprd/node_modules /app/node_modules
COPY --from=build /hoprd/package.json /app/package.json

VOLUME ["/app/db"]

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080

ENTRYPOINT ["dumb-init", "node", "/app/index.js"]
