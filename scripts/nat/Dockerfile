FROM docker:20.10.12-alpine3.15@sha256:a729cce205a05b0b86dc8dca87823efaffc3f74979fe7dc86a707c2fbf631b61

# This build arg is mandatory
ARG HOPRD_RELEASE
RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}
RUN apk add bash
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh"]
