#
# Run HOPRd  within a single container.
#
# NB. This is not a long term solution, so you probably don't want to rely on 
# this, our ultimate goal is a single HOPRd process, but this is a quick solution
# to expedite our HOPR node work.
#

# -- BASE STAGE --------------------------------

FROM node:12.9.1-buster AS base
WORKDIR /hoprd

RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    ca-certificates \
    fuse \
    gcc \
    cmake \
    wget

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

FROM node:12.9.1-buster AS runtime

# specify network, this will be passed to process.yaml --env
ARG network

# install yarn
RUN yarn global add pm2

ENV NODE_ENV 'production'
WORKDIR /app

# Server
COPY --from=build /hoprd/lib/ /app/
COPY --from=build /hoprd/hopr-admin /app/hopr-admin
COPY --from=build /hoprd/node_modules /app/node_modules
COPY --from=build /hoprd/package.json /app/package.json

# PM2
COPY process.yaml /app/process.yaml

# Docker Entrypoint
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
# we cannot specify a build-arg in the entrypoint :(
RUN sed -i "s|replaced_by_dockerfile_network|${network}|g" docker-entrypoint.sh

VOLUME ["/app/db"]

#? Hopr-server gRPC
EXPOSE 9091
# Hopr-server gRPC
EXPOSE 50051
# Admin web server
EXPOSE 3000

ENTRYPOINT ["./docker-entrypoint.sh"]
