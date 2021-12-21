# Run HOPRd  within a single container using npm

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:269885d847d382df99cc93ab5c37d3aea9f81fd8add895a9a3c639f84ae45aa4 as build

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
FROM node:16-buster-slim@sha256:269885d847d382df99cc93ab5c37d3aea9f81fd8add895a9a3c639f84ae45aa4 as runtime

# we use tini as process 1 to catch signals properly, which is also built into
# D<Plug>_ocker by default
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     tini \
  && rm -rf /var/lib/apt/lists/* \
  && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

WORKDIR /app

# copy over the contents of node_modules etc
COPY --from=build /app .

# create directory which is later used for the database, so that it inherits
# permissions when mapped to a volume
RUN mkdir db

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# set volume which can be mapped by users on the host system
VOLUME ["/app/db"]

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS=--max_old_space_size=4096

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080
# p2p
EXPOSE 9091

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hoprd"]
