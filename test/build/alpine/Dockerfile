FROM node:12.20.0-alpine

RUN mkdir /hoprd
WORKDIR /hoprd
RUN npm install --unsafe-perm=true -g @hoprnet/hoprd@v1.54.5
#RUN DEBUG=hopr* hoprd --init --password="foo" --run="balance"
