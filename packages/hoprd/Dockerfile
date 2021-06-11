# Run HOPRd  within a single container using npm

FROM node:14-buster@sha256:d1d71f63a8b92fa73d8df65fc512f64f2bac8a46b08cbd8caf4d36203d6e210b

RUN wget https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_amd64.deb && \
    dpkg -i dumb-init_*.deb

ARG HOPRD_VERSION
ENV HOPRD_VERSION=${HOPRD_VERSION:-}

ENV NODE_ENV 'production'
ENV NEXT_TELEMETRY_DISABLED 1
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS=--max_old_space_size=4096

WORKDIR /app

VOLUME ["/app/db"]

RUN yarn add @hoprnet/hoprd@${HOPRD_VERSION}

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080

ENTRYPOINT ["dumb-init", "yarn", "--cwd", "/app", "hoprd"]
