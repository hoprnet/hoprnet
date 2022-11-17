# Alpine 3.16 x86_64 https://hub.docker.com/layers/library/alpine/3.16.3/images/sha256-3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d
FROM alpine:3.16@sha256:3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d as runtime

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