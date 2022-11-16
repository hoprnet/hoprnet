ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-hopr-toolchain:latest}

# Creates a Docker container that encapsulates a cover-traffic node

FROM ${HOPR_TOOLCHAIN_IMAGE} as build

WORKDIR /app/hoprnet

COPY .yarn .yarn
COPY Makefile Cargo.toml Cargo.lock yarn.lock tsconfig.build.json tsconfig.json tsconfig.scripts.json yarn.lock rust-toolchain.toml ./
COPY packages/ ./packages

LABEL description="Launches a cover-traffic node which keeps sending cover-traffic through the hopr network"

RUN CI=true make deps package=hopr-cover-traffic-daemon && \
    # Remove all tests
    find ./packages -type f -name '*.spec.ts' -delete && \
    # Only build cover-traffic-daemon + dependencies
    NO_NEXT=true make -j build package=cover-traffic-daemon && \
    # Remove all typescript files 
    find ./packages -type f -name '*.ts' -delete && \
    # No need for these packages
    rm -R packages/avado packages/horpd && \
    # Everything that we don't need in ethereum package
    rm -R packages/ethereum/contracts packages/ethereum/deploy packages/ethereum/hardhat packages/ethereum/tasks packages/ethereum/test && \
    # Remove node_modules as they include devDependencies
    rm -R node_modules && \
    # As we are on *nix, use hardlinks in node_modules to save some space
    yarn config set nmMode hardlinks-local && \
    # Install without devDependencies and release build of wrtc
    DEBUG= CI=true NO_CARGO=true PRODUCTION=true make deps package=hopr-cover-traffic-daemon

# Alpine 3.16 x86_64
FROM alpine:16@sha256:1304f174557314a7ed9eddb4eab12fed12cb0cd9809e4c28f29af86979a3c870 as runtime

WORKDIR /app/hoprnet

RUN mkdir -p scripts/toolchain

COPY .nvmrc package.json .
COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

RUN apk add --no-cache tini libc6-compat bash curl tar jq xz && \
    # Install Node.js
    ./scripts/toolchain/install-toolchain.sh --runtime-only && \
    # Remove unused tools
    apk del bash curl tar jq xz && \
    # Remove unused files and config
    rm -R scripts .nvmrc && \
    # symlink required to get WRTC to work
    ln -s /lib/libc.musl-x86_64.so.1 /lib/ld-linux-x86-64.so.2 && \
    # create directory which is later used for the database, so that it inherits
    # permissions when mapped to a volume
    mkdir -p hoprd-db

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS='--max_old_space_size=4096 --experimental-wasm-modules'

# p2p
EXPOSE 9091

# copy over the contents of node_modules etc
COPY package.json .
COPY --from=build /app/hoprnet/node_modules node_modules/
COPY --from=build /app/hoprnet/packages packages/

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprnet/hoprd-db"]

# set data directory to user-mountable directory
ENV HOPRD_DATA=/app/hoprnet/hoprd-db

WORKDIR /app/hoprnet/packages/cover-traffic-daemon

ENTRYPOINT ["/sbin/tini", "--", "node", "lib/main.cjs"]