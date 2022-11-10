FROM alpine:16@sha256:1304f174557314a7ed9eddb4eab12fed12cb0cd9809e4c28f29af86979a3c870

LABEL description="Image that contains all utilities necessary to build hopr monorepo, including Rust and Yarn"

# python, perl are used by some nodejs dependencies as an installation requirement
RUN apk add --no-cache bash build-base ca-certificates jq curl git

WORKDIR /app/hoprnet-toolchain

# use bash to have source command and persistent environment
SHELL ["/bin/bash", "-lc"]

# Gets yarn executable
COPY .yarn .yarn/

COPY package.json .yarnrc.yml rust-toolchain.toml .nvmrc yarn.lock ./
RUN mkdir -p scripts/toolchain
COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

# Downloads prebuilt toolchain utilities
RUN ./scripts/toolchain/install-toolchain.sh