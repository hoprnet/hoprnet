# Run HOPRd  within a single container using npm

# use latest alpine version with node v14, fairly small Docker image size as a
# result
FROM node:14-alpine3.13@sha256:782e891986f16cc661bfe928d0d163d4d0e6cf5cc05453dff2093c015fcc4a64

# we use dumb-init as process 1 to catch signals properly
# python is used by some nodejs dependencies as an installation requirement
RUN apk --no-cache --update-cache --available upgrade && \
    apk --no-cache --update-cache add \
    build-base \
    dumb-init \
    python3

# enable to pass the version to Docker using either --build-arg or an
# environment variable
# if its not given, yarn will install the latest version of the package
ARG HOPRD_VERSION
ENV HOPRD_VERSION=${HOPRD_VERSION:-}

# making sure some standard environment variables are set for production use
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
