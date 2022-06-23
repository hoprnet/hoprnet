# Run HOPRd  within a single container using npm

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-bullseye-slim@sha256:8265ac132f720998222008355e11535caf53d6bccecbb562a055605138975b4e as build

# python is used by some nodejs dependencies as an installation requirement
RUN apt-get update && \
    apt-get install -y \
    git \
    python3 \
    build-essential

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

WORKDIR /app

# install hoprd as a local module
RUN yarn add @hoprnet/hoprd@${PACKAGE_VERSION}

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-bullseye-slim@sha256:8265ac132f720998222008355e11535caf53d6bccecbb562a055605138975b4e as runtime

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

# we use tini as process 1 to catch signals properly, which is also built into Docker by default
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
RUN mkdir -p hoprd-db

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# set volume which can be mapped by users on the host system
VOLUME ["/app/hoprd-db"]

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hoprd"]
