# Run hopr-cover-traffic-daemon within a single container using npm

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:a5eecf2ee53935cb7974dfff42260d97289d61be9d7a2062c693be93c0cdcd7a as build

# python is used by some nodejs dependencies as an installation requirement
RUN apt-get update && \
    apt-get install -y \
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
ENV NODE_OPTIONS=--max_old_space_size=4096
ENV npm_config_build_from_source false

# install hoprd as a local module
RUN yarn add @hoprnet/hopr-cover-traffic-daemon@${PACKAGE_VERSION}

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:a5eecf2ee53935cb7974dfff42260d97289d61be9d7a2062c693be93c0cdcd7a as runtime

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

# set volume which can be mapped by users on the host system
VOLUME ["/app/db"]

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV DEBUG 'hopr*'
ENV NODE_OPTIONS=--max_old_space_size=4096

# p2p
EXPOSE 9091

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hopr-cover-traffic-daemon"]
