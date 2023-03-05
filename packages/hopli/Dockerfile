ARG RUST_IMAGE=${RUST_IMAGE:-rust:1.67}

FROM ${RUST_IMAGE} as builder


RUN mkdir -p /hopli
WORKDIR /hopli

COPY ./packages/hopli/Cargo.toml ./Cargo.toml
COPY ./packages/hopli/src ./src

RUN cargo build --release


# Debian 11.5 https://hub.docker.com/layers/library/debian/bullseye-slim/images/sha256-64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9?context=explore
FROM debian:bullseye-slim@sha256:64e251208f3bddf166a7020437103f75aabf3fceb1d5bd932272f3f73668caa9 as runtime

LABEL name="hopli" \
      maintainer="tech@hoprnet.org" \
      vendor="HOPR" \
      summary="Hopli cli tool" \
      description="Hopli is a commandline tool that interacts mainly with Foundry"

# build project sources
WORKDIR /root
COPY ./packages/ethereum/contracts /root/contracts
COPY --from=builder /hopli/target/release/hopli /bin/hopli
RUN apt-get update && \
    apt install -y git curl && \
    curl -L "https://raw.githubusercontent.com/foundry-rs/foundry/master/foundryup/install" | bash && \
	$HOME/.foundry/bin/foundryup && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false && \
    echo "export PATH=$PATH:/root/.foundry/bin" >> /root/.bashrc

ENTRYPOINT ["/bin/hopli"]
CMD ["--help"]
