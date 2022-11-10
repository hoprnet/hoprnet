ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-hopr-toolchain}

# Run HOPRd  within a single container using npm
FROM ${HOPR_TOOLCHAIN_IMAGE} as build

WORKDIR /app/hoprnet

COPY .yarn .yarn
COPY packages/ ./packages
COPY Makefile Cargo.toml Cargo.lock yarn.lock tsconfig.build.json tsconfig.json tsconfig.scripts.json yarn.lock rust-toolchain.toml ./

LABEL description="Launches a hoprd node"

RUN CI=true make deps package=hoprd

# Remove all tests
RUN find ./packages -type f -name '*.spec.ts' -delete 

RUN make -j build package=hoprd

# Remove all typescript files
RUN find ./packages -type f -name '*.ts' -delete

# # No need for these packages
RUN rm -R packages/avado packages/cover-traffic-daemon

# Everything that we don't need in ethereum package
RUN rm -R packages/ethereum/contracts packages/ethereum/deploy packages/ethereum/hardhat packages/ethereum/tasks packages/ethereum/test

RUN rm -R node_modules

# As we are on *nix, use hardlinks in node_modules to save some space
RUN yarn config set nmMode hardlinks-local

RUN DEBUG= CI=true NO_CARGO=true PRODUCTION=true make deps package=hoprd

# # use slim version of node on Debian buster for smaller image sizes
FROM alpine as runtime

WORKDIR /app/hoprnet

RUN apk add --no-cache tini libc6-compat bash curl tar jq xz

COPY .nvmrc .
RUN mkdir -p scripts/toolchain
COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

# Install Node.js
RUN ./scripts/toolchain/install-toolchain.sh --runtime-only

RUN apk del bash curl tar jq xz && rm -R scripts .nvmrc

# # making sure some standard environment variables are set for production use
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
# p2p
EXPOSE 9091

# symlink required to get WRTC to work
RUN ln -s /lib/libc.musl-x86_64.so.1 /lib/ld-linux-x86-64.so.2

COPY package.json ./
COPY --from=build /app/hoprnet/packages ./packages
COPY --from=build /app/hoprnet/node_modules ./node_modules

# create directory which is later used for the database, so that it inherits
# permissions when mapped to a volume
RUN mkdir -p hoprd-db

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprd-db"]

# set data directory to user-mountable directory
ENV HOPRD_DATA=/app/hoprd-db

WORKDIR /app/hoprnet/packages/hoprd

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

ENTRYPOINT ["/sbin/tini", "--", "node", "./lib/main.js"]