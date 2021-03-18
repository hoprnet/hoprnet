FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y sudo
RUN sudo apt install -y curl git

# nvm environment variables
ENV NODE_VERSION 14.15.5
ENV NVM_DIR /usr/local/nvm
RUN mkdir $NVM_DIR

RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.37.0/install.sh | bash

# add node and npm to path so the commands are available
ENV NODE_PATH $NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH $NVM_DIR/versions/node/v$NODE_VERSION/bin:$PATH

# install node and npm
RUN echo "source $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default" | bash

RUN mkdir /hoprd
WORKDIR /hoprd
RUN sudo env "PATH=$PATH" npm install --unsafe-perm=true -g @hoprnet/hoprd@v1.54.5
#RUN DEBUG=hopr* hoprd --init --password="foo" --run="balance"
