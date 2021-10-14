FROM node:16-buster-slim@sha256:a49f003fbc2439e20601ed466a2cbc80699f238b56bb78ccb934bb3d92a23d53 as build

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_OPTIONS=--max_old_space_size=4096
ENV npm_config_build_from_source false

# copying everything and preparing for installing
WORKDIR /app

COPY . .
# installing dependencies
RUN yarn install
# build hoprd locally
RUN yarn build
# run tests
RUN yarn test

# use slim version of node on Debian buster for smaller image sizes
FROM node:16-buster-slim@sha256:a49f003fbc2439e20601ed466a2cbc80699f238b56bb78ccb934bb3d92a23d53 as runtime

# making sure some standard environment variables are set for production use
ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1
ENV NODE_OPTIONS=--max_old_space_size=4096
ENV DEBUG 'hopr*'

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
RUN mkdir -p db

# DISABLED temporarily until a migration path has been tested
# switch to normal user, to prevent dangerous root access
# RUN chown -R node:node .

# set volume which can be mapped by users on the host system
VOLUME ["/app/db"]

# DISABLED temporarily until a migration path has been tested
# finally set the non-root user so the process also run un-privilidged
# USER node

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001
# Healthcheck server
EXPOSE 8080
# p2p
EXPOSE 9091

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "workspace", "@hoprnet/hoprd", "exec", "--", "yarn", "start"]
