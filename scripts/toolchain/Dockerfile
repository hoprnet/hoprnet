# Alpine 3.16 x86_64 https://hub.docker.com/layers/library/alpine/3.16.3/images/sha256-3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d
FROM alpine:3.16@sha256:3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d as runtime

LABEL description="Image that contains all utilities necessary to build hopr monorepo, including Rust and Yarn"

WORKDIR /app/hoprnet-toolchain

RUN mkdir -p scripts/toolchain

# install glibc for alpine to be compatible with libc-based software
ENV GLIBC_REPO=https://github.com/sgerrand/alpine-pkg-glibc
ENV GLIBC_VERSION=2.34-r0

RUN apk add --no-cache --update \
      bash build-base ca-certificates jq curl git libstdc++ lsof && \
    for pkg in glibc-${GLIBC_VERSION} glibc-bin-${GLIBC_VERSION}; do \
      curl -sSL ${GLIBC_REPO}/releases/download/${GLIBC_VERSION}/${pkg}.apk -o /tmp/${pkg}.apk; \
    done && \
    apk add --force-overwrite --allow-untrusted /tmp/*.apk && \
    apk fix --force-overwrite alpine-baselayout-data && \
    rm -v /tmp/*.apk && \
    /usr/glibc-compat/sbin/ldconfig /lib /usr/glibc-compat/lib && \
    echo 'hosts: files mdns4_minimal [NOTFOUND=return] dns mdns4' >> /etc/nsswitch.conf

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
