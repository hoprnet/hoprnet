FROM docker:20.10.12-alpine3.15

ARG HOPR_RELEASE
ENV HOPR_RELEASE=${HOPR_RELEASE:-latest}
RUN apk add bash
COPY start-nat-node.sh start-nat-node.sh
ENTRYPOINT ["./start-nat-node.sh"]
