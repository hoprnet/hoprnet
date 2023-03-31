# Debian 11.5 https://hub.docker.com/layers/library/debian/bullseye-slim/images/sha256-64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9?context=explore
FROM debian:bullseye-slim@sha256:64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9 as runtime

LABEL description="Image that contains all utilities necessary to build hopr monorepo, including Rust and Yarn"

# use bash to have source command and persistent environment
SHELL ["/bin/bash", "-lc"]

WORKDIR /app/hoprnet-toolchain

RUN mkdir -p scripts/toolchain

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      bash \
      build-essential \
      ca-certificates \
      curl \
      unzip \
      git \
      jq \
      lsof \
      python3 \
      unzip \
      xz-utils \
      time \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

# Gets yarn executable
COPY .yarn .yarn/

COPY Makefile package.json .yarnrc.yml rust-toolchain.toml .nvmrc yarn.lock Cargo.toml ./

COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

# Make sure that Rust toolchain utilities can be found
ENV PATH=${PATH}:${HOME}/.cargo/bin:/app/hoprnet-toolchain/.foundry/bin

# Install foundry
RUN make install-foundry

# Downloads prebuilt toolchain utilities
RUN ./scripts/toolchain/install-toolchain.sh
RUN echo "export PATH=${PATH}:${HOME}/.cargo/bin:/app/hoprnet-toolchain/.foundry/bin" >> /root/.bashrc
