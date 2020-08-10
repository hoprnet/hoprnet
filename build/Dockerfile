FROM node:12.9.1

WORKDIR /usr/src/app

RUN apt update && apt install -y yarn
RUN git clone https://github.com/hoprnet/hopr-admin.git
WORKDIR /usr/src/app/hopr-admin
RUN yarn policies set-version 1.19.2
RUN yarn

ENTRYPOINT cd /usr/src/app/hopr-admin && yarn run server
