ARG RUST_IMAGE=${RUST_IMAGE:-rust:1.67}
# Debian Bullseye https://hub.docker.com/layers/library/debian/bullseye-20230227/images/sha256-6f1f698e20c8ae73b110eb614a2c6d13fbdb504bbbbcf2965c4d0b80b7bb7fb1?context=explore
ARG DEBIAN_IMAGE=${RUST_IMAGE:-debian:bullseye-20230227@sha256:6f1f698e20c8ae73b110eb614a2c6d13fbdb504bbbbcf2965c4d0b80b7bb7fb1}

FROM ${RUST_IMAGE} as builder


RUN mkdir -p /hopli
WORKDIR /hopli

COPY ./packages/hopli/Cargo.toml ./Cargo.toml
COPY ./packages/hopli/src ./src

RUN cargo build --release


FROM ${DEBIAN_IMAGE} as runtime

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
    apt install --no-install-recommends -y git curl ca-certificates && \
    curl -L "https://raw.githubusercontent.com/foundry-rs/foundry/master/foundryup/install" | bash && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false && \
    $HOME/.foundry/bin/foundryup

ENTRYPOINT ["/bin/hopli"]
CMD ["--help"]
