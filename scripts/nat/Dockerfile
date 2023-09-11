# Docker 20.10.23 on Alpine 3.17 x86_64 https://hub.docker.com/layers/library/docker/23.0.0-alpine3.17/images/sha256-81d8c3b659f44a0a0bdcde3c1f0b27f3ca246105a8a1ecab242efc6de586de7e
FROM docker:23.0.5-alpine3.17@sha256:f23f0a4013f184f6af3a3892dd12eba74bdbc5988d2a54ae468a8a6a44078434 as runtime

# This build arg is mandatory
ARG HOPRD_RELEASE

RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}

RUN apk add --no-cache bash

COPY start-nat-node.sh start-nat-node.sh

ENTRYPOINT ["./start-nat-node.sh"]
