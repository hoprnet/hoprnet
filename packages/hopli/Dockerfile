ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-europe-west3-docker.pkg.dev/hoprassociation/docker-images/hopr-toolchain:latest}

# Debian 11.7
# https://hub.docker.com/layers/library/debian/11.7-slim/images/sha256-b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3?context=explore
ARG DEBIAN_IMAGE=${DEBIAN_IMAGE:-debian:11.7-slim@sha256:b74f58783fdb8f58107f1d0b3d8e7721e11feeacd9c4c8faa159a14bb0fc6bb3}

FROM ${HOPR_TOOLCHAIN_IMAGE} as builder

WORKDIR /app/hoprnet

# musl toolchain for static binaries
RUN apt update && apt install -y pkg-config libssl-dev musl-tools
ENV SYSROOT=/dummy
ENV OPENSSL_STATIC=1
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

COPY Makefile Cargo.toml Cargo.lock ./
COPY vendor vendor/
COPY packages packages/

RUN make build-solidity-types

WORKDIR /app/hoprnet/packages/hopli

RUN rustup target install $(uname -m)-unknown-linux-musl
RUN OPENSSL_LIB_DIR=/usr/lib/$(uname -m)-linux-gnu RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target $(uname -m)-unknown-linux-musl --target-dir /app/hoprnet/packages/hopli/target
RUN rm -rf target/release && \
    mv target/$(uname -m)-unknown-linux-musl/release target/

FROM ${DEBIAN_IMAGE} as runtime

LABEL name="hopli" \
    maintainer="tech@hoprnet.org" \
    vendor="HOPR" \
    summary="Hopli cli tool" \
    description="Hopli is a commandline tool that interacts mainly with Foundry"

ENV ETHERSCAN_API_KEY="random_giberish"

# build project sources
WORKDIR /app/hoprnet
VOLUME /app/hoprnet/node

COPY packages/ethereum/contracts packages/ethereum/contracts
COPY Makefile .
COPY vendor/solidity vendor/solidity

COPY --from=builder /app/hoprnet/packages/hopli/target/release/hopli /bin/hopli
RUN apt-get update && \
    apt install --no-install-recommends -y git curl ca-certificates make && \
    make install-foundry && \
    echo "export PATH=${PATH}:/root/.foundry/bin/" >> /root/.bashrc && \
    rm -rf /var/lib/apt/lists/* && \
    apt remove -y make && \
    apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false && \
    rm /app/hoprnet/Makefile

ENTRYPOINT ["/bin/hopli"]
CMD ["--help"]

