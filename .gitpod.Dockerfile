FROM gitpod/workspace-full@sha256:5f36273bfffde146590ce2b3e1d1a5d59c3829253599701d91e0787f0f53ead9

# Install & use custom Node.js version
ENV NODE_VERSION=16
RUN bash -c ". .nvm/nvm.sh && \
        nvm deactivate && \
        nvm uninstall 14.17.3 && \
        nvm install 16 && \
        nvm alias default 16"
ENV PATH=/home/gitpod/.nvm/versions/node/v${NODE_VERSION}/bin:$PATH

# Install yarn (without node)
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add - && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list && \
    sudo apt update && sudo apt install --no-install-recommends yarn

# Switch to the root user to install system wide tools
USER root

# Install packages is a script provided by the base gitpod image
# Ref: https://github.com/gitpod-io/workspace-images/tree/master/base
RUN install-packages \
      tmux \
      neovim \
      netcat

# Installing gcloud into our path to have it available for devs
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add - && apt-get update -y && apt-get install google-cloud-sdk -y

# Switch to the gitpod user to install user specific tools
USER gitpod
RUN curl -so "$HOME/.tmux.conf" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf && \
    curl -so "$HOME/.tmux.conf.local" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf.local
