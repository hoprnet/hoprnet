ARG HOPR_TOOLCHAIN_IMAGE=${HOPR_TOOLCHAIN_IMAGE:-gcr.io/hoprassociation/hopr-toolchain:latest}
# Debian 11.5 https://hub.docker.com/layers/library/debian/bullseye-slim/images/sha256-64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9?context=explore
ARG DEBIAN_IMAGE=${DEBIAN_IMAGE:-debian:bullseye-slim@sha256:64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9}

FROM ${HOPR_TOOLCHAIN_IMAGE} as builder

WORKDIR /app/hoprnet

# musl toolchain for static binaries
RUN apt update && apt install -y pkg-config libssl-dev musl-tools
ENV SYSROOT=/dummy
ENV OPENSSL_STATIC=1
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

COPY ./Cargo.toml ./Cargo.toml
COPY packages packages/

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
WORKDIR /root
COPY ./packages/ethereum/contracts /root/contracts
COPY Makefile /root
COPY --from=builder /app/hoprnet/packages/hopli/target/release/hopli /bin/hopli
RUN apt-get update && \
    apt install --no-install-recommends -y git curl ca-certificates make && \
    make install-foundry && \
    echo "export PATH=${PATH}:/root/.foundry/bin/" >> /root/.bashrc && \
    rm -rf /var/lib/apt/lists/* && \
    apt remove -y make && \
    apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false && \
    rm /root/Makefile

ENTRYPOINT ["/bin/hopli"]
CMD ["--help"]
