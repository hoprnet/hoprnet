# Debian 11.7
# https://hub.docker.com/layers/library/debian/11.7-slim/images/sha256-b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3?context=explore
FROM --platform=linux/amd64 debian:11.7-slim@sha256:b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3 as runtime

LABEL description="Image that contains all utilities necessary to build hopr monorepo"

# use bash to have source command and persistent environment
SHELL ["/bin/bash", "-lc"]

WORKDIR /app/hoprnet-toolchain

RUN mkdir -p scripts/toolchain

ENV DEBIAN_FRONTEND="noninteractive"

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

RUN ls -a

# Hoprd Docker image copies files from .cargo
COPY .cargo .cargo
COPY Makefile rust-toolchain.toml Cargo.toml ./

COPY scripts/toolchain/install-toolchain.sh ./scripts/toolchain

# Make sure that Rust toolchain utilities can be found
ENV PATH=${PATH}:${HOME}/.cargo/bin:/app/hoprnet-toolchain/.foundry/bin

# Install foundry
RUN make install-foundry

# Downloads prebuilt toolchain utilities
RUN ./scripts/toolchain/install-toolchain.sh
RUN echo "export PATH=${PATH}:${HOME}/.cargo/bin:/app/hoprnet-toolchain/.foundry/bin" >> /root/.bashrc
