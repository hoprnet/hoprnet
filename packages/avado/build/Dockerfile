ARG UPSTREAM_VERSION

FROM gcr.io/hoprassociation/hopr-admin:07aec21b as hopr-admin

FROM gcr.io/hoprassociation/hoprd:${UPSTREAM_VERSION}

RUN apk add --no-cache --purge -uU supervisor && \
    rm -rf /var/cache/apk/* /tmp/*

ENV AVADO=true

COPY supervisord.conf /etc/supervisord.conf

COPY --from=hopr-admin /app /hopr-admin

ENTRYPOINT ["supervisord", "--nodaemon", "--configuration", "/etc/supervisord.conf"]
