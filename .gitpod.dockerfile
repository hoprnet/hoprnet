FROM gitpod/workspace-full

# Switch to the root user to install system wide tools
USER root

# Install packages is a script provided by the base gitpod image
# Ref: https://github.com/gitpod-io/workspace-images/tree/master/base
RUN install-packages \
      tmux \
      neovim

# Installing gcloud into our path to have it available for devs
RUN echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] http://packages.cloud.google.com/apt cloud-sdk main" | tee -a /etc/apt/sources.list.d/google-cloud-sdk.list && curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key --keyring /usr/share/keyrings/cloud.google.gpg  add - && apt-get update -y && apt-get install google-cloud-sdk -y

# Switch to the gitpod user to install user specific tools
USER gitpod
RUN curl -so "$HOME/.tmux.conf" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf && \
    curl -so "$HOME/.tmux.conf.local" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf.local
