FROM docker:20.10.14-alpine3.15@sha256:41978d1974f05f80e1aef23ac03040491a7e28bd4551d4b469b43e558341864e

# This build arg is mandatory
ARG HOPRD_RELEASE
RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}
RUN apk add bash
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh"]
