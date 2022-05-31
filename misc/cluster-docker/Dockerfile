# Run a cluster of 5 hoprd nodes and a hardhat instance in a single Docker
# container

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:380112b3f3df96ae593b0b89f4c44e501e9ae0ec1580b270b50f8ac52688256e as build

# python is used by some nodejs dependencies as an installation requirement
RUN apt-get update && \
    apt-get install -y \
    git \
    python3 \
    build-essential

WORKDIR /app

# enable to pass the version to Docker using either --build-arg or an
# environment variable
# if its not given, yarn will install the latest version of the package
ARG PACKAGE_VERSION
RUN echo "ARG PACKAGE_VERSION=${PACKAGE_VERSION}"
ENV PACKAGE_VERSION=${PACKAGE_VERSION:-}
RUN echo "ENV PACKAGE_VERSION=${PACKAGE_VERSION}"

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_OPTIONS=--max_old_space_size=4096
ENV npm_config_build_from_source false

# install hoprd as a local module
RUN yarn add @hoprnet/hoprd@${PACKAGE_VERSION}

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:380112b3f3df96ae593b0b89f4c44e501e9ae0ec1580b270b50f8ac52688256e as runtime

# we use tini as process 1 to catch signals properly, which is also built into Docker by default
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     tini supervisor \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

RUN mkdir -p /var/log/supervisor

COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

WORKDIR /app

# copy over the contents of node_modules etc
COPY --from=build /app .

# create directory which is later used for the database, so that it inherits
# permissions when mapped to a volume
RUN mkdir hoprd-db

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprd-db"]

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV DEBUG 'hopr*'
# only 1GB of RAM is used at max to keep overall memory consumption of a container at 6GB max
ENV NODE_OPTIONS=--max_old_space_size=1024

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080
# p2p
EXPOSE 9091

ENTRYPOINT ["/usr/bin/tini", "--", "supervisord"]
