# Alpine 3.16 x86_64 https://hub.docker.com/layers/library/alpine/3.16.3/images/sha256-3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d
FROM alpine:3.16@sha256:3d426b0bfc361d6e8303f51459f17782b219dece42a1c7fe463b6014b189c86d as runtime

# This build arg is mandatory
ARG HOPRD_RELEASE

RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}

RUN apk add --no-cache bash

COPY start-nat-node.sh start-nat-node.sh

ENTRYPOINT ["./start-nat-node.sh"]
