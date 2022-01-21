FROM docker:20.10.12-alpine3.15

# This build arg is mandatory
ARG HOPRD_RELEASE
RUN test -n "$HOPRD_RELEASE"

ENV HOPRD_RELEASE=${HOPRD_RELEASE}
RUN apk add bash
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh"]
