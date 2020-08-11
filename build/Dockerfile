FROM node:12.9.1

WORKDIR /usr/src/app

RUN apt update && apt install -y yarn
ADD ./build/hopr-admin /usr/src/app/hopr-admin
WORKDIR /usr/src/app/hopr-admin
RUN rm -r .git && git init # Yarn is broken https://github.com/yarnpkg/yarn/issues/7537
RUN yarn policies set-version 1.19.2
RUN yarn

ENTRYPOINT cd /usr/src/app/hopr-admin && yarn run server
