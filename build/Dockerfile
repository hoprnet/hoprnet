FROM node:12

WORKDIR /usr/src/app

RUN apt update && apt install -y yarn
RUN git clone https://github.com/hoprnet/hopr-core.git
RUN cd hopr-core
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
RUN export NVM_DIR="$HOME/.nvm"
RUN $NVM_DIR/nvm.sh
 
RUN nvm install 12
RUN nvm use
RUN yarn install

ENTRYPOINT cd /usr/src/app/hopr-core && yarn hopr 0
