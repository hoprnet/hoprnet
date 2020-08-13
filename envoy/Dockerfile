FROM envoyproxy/envoy-alpine:v1.14.1

COPY ./envoy.yaml.tmpl /tmpl/envoy.yaml.tmpl
COPY docker-entrypoint.sh /

RUN chmod 500 /docker-entrypoint.sh

RUN apk add --update && \
    apk add --no-cache gettext 

ENTRYPOINT ["/docker-entrypoint.sh"]
