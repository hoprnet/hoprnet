# Run HOPRd  within a single container using npm

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-bullseye-slim@sha256:95d4fdef5e12916a89f7d56dca6d8099de61df875c3c61106cb53e0ba1a48d44 as build

# use bash to have source command and persistent environment
SHELL ["/bin/bash", "-lc"]

# python is used by some nodejs dependencies as an installation requirement
RUN apt-get update && \
    apt-get install -y \
    git \
    python3 \
    unzip \
    curl \
    build-essential \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

# enable to pass the version to Docker using either --build-arg or an
# environment variable
# if its not given, yarn will install the latest version of the package
ARG PACKAGE_VERSION
RUN echo "ARG PACKAGE_VERSION=${PACKAGE_VERSION}"
ENV PACKAGE_VERSION=${PACKAGE_VERSION:-}
RUN echo "ENV PACKAGE_VERSION=${PACKAGE_VERSION}"

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_OPTIONS=--max_old_space_size=4096
ENV npm_config_build_from_source false

WORKDIR /app

# Install minimal version of Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal

# Make cargo available in home path
RUN source $HOME/.cargo/env

ADD https://github.com/hoprnet/hoprnet/archive/refs/tags/v${PACKAGE_VERSION}.zip .
RUN unzip ./v${PACKAGE_VERSION}.zip 
RUN mv /app/hoprnet-${PACKAGE_VERSION} /app/hoprnet

WORKDIR /app/hoprnet

RUN CI=true make -j deps

# Remove all tests
RUN find ./packages -type f -name '*.spec.ts' -delete 

RUN make -j build

# Remove all typescript files
RUN find ./packages -type f -name '*.ts' -delete

# Remove node_modules as they contain devDependencies
RUN rm -R node_modules

# Remove all other directories and files that we don't need
RUN rm -R target packages/*/src packages/*/docs test docs .github .processes .vscode *.md tsconfig* flake* shell.nix scripts cloudbuild.yaml Makefile Dockerfile.hardhat Cargo.* rust-toolchain.toml Brewfile .actrc .editorconfig .gitpod* .nvmrc .prettierignore .readthedocs.yml .snyk .gitignore .envrc

# No need for these packages
RUN rm -R packages/avado packages/cover-traffic-daemon

# Everything that we don't need in ethereum package
RUN rm -R packages/ethereum/contracts packages/ethereum/deploy packages/ethereum/hardhat packages/ethereum/tasks packages/ethereum/test

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-bullseye-slim@sha256:95d4fdef5e12916a89f7d56dca6d8099de61df875c3c61106cb53e0ba1a48d44 as runtime

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
# p2p
EXPOSE 9091

# we use tini as process 1 to catch signals properly, which is also built into Docker by default
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     tini \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

WORKDIR /app

# copy over built artifacts
COPY --from=build /app/hoprnet .

# As we are on Debian, use hardlinks in node_modules to save space
RUN yarn config set nmMode hardlinks-local

# Install without devDependencies, and using WebRTC Release builds (`DEBUG=`)
# see https://github.com/node-webrtc/node-webrtc/blob/685aa433623b9bf98e8c7c0124e7864e01b883e6/scripts/download-prebuilt.js#L11
RUN DEBUG= CI=true yarn workspaces focus --production @hoprnet/hoprd

# Remove yarn cache globally
RUN yarn cache clean --all

WORKDIR /app/packages/hoprd

# create directory which is later used for the database, so that it inherits
# permissions when mapped to a volume
RUN mkdir -p hoprd-db

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprd-db"]

# set data directory to user-mountable directory
ENV HOPRD_DATA=/app/hoprd-db

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

ENTRYPOINT ["/usr/bin/tini", "--", "node", "./lib/main.cjs"]
