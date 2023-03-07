ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-hopr-toolchain:latest}

# Creates a Docker container that encapsulates a horpd node

FROM ${HOPR_TOOLCHAIN_IMAGE} as build

WORKDIR /app/hoprnet

RUN mv /app/hoprnet-toolchain/.cargo . && \
    mv /app/hoprnet-toolchain/.yarn .

COPY .cargo/config.toml .cargo/
COPY vendor vendor/
COPY packages packages/
COPY Makefile Cargo.toml Cargo.lock .yarnrc.yml yarn.lock package.json tsconfig.build.json tsconfig.json tsconfig.scripts.json yarn.lock rust-toolchain.toml ./

LABEL description="Launches a hoprd node"

RUN make deps-docker package=hoprd && \
    # Remove all tests
    find ./packages -type f -name '*.spec.ts' -delete && \
    # Only build hoprd + dependencies
    make -j build package=hoprd && \
    # Remove all typescript files
    find ./packages -type f -name '*.ts' -delete && \
    # No need for these packages
    rm -R packages/avado packages/cover-traffic-daemon && \
    # Everything that we don't need in ethereum package
    rm -R packages/ethereum/contracts && \
    # Remove node_modules as they include devDependencies
    rm -R node_modules && \
    # As we are on *nix, use hardlinks in node_modules to save some space
    yarn config set nmMode hardlinks-local && \
    # Install without devDependencies
    PRODUCTION=true make deps-docker package=hoprd

# Debian 11.5 https://hub.docker.com/layers/library/debian/bullseye-slim/images/sha256-64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9?context=explore
FROM debian:bullseye-slim@sha256:64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9 as runtime

WORKDIR /app/hoprnet

RUN mkdir -p scripts/toolchain

COPY .nvmrc package.json ./
COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

RUN apt-get update \
 && apt-get install -y \
 bash curl jq tini ca-certificates tar xz-utils \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false \
 # Install Node.js
 && ./scripts/toolchain/install-toolchain.sh --runtime-only \
 # Remove unused files and config
 && rm -R scripts .nvmrc \
 # create directory which is later used for the database, so that it inherits
 # permissions when mapped to a volume
 && mkdir -p hoprd-db

COPY --from=build /app/hoprnet/packages ./packages
COPY --from=build /app/hoprnet/node_modules ./node_modules

# we copy foundry-tool over since we might need it at some point as a binary
COPY --from=build /app/hoprnet/.cargo/bin/foundry-tool .cargo/bin/

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprnet/hoprd-db"]

# set data directory to user-mountable directory
ENV HOPRD_DATA=/app/hoprnet/hoprd-db

WORKDIR /app/hoprnet/packages/hoprd

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS="--max_old_space_size=4096 --experimental-wasm-modules"

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080
# p2p UDP + TCP
EXPOSE 9091/udp
EXPOSE 9091/tcp

ENTRYPOINT ["/usr/bin/tini", "--", "node", "./lib/main.cjs"]
