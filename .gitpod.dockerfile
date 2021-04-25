FROM gitpod/workspace-full

# Switch to the root user to install system wide tools
USER root

# Install packages is a script provided by the base gitpod image
# Ref: https://github.com/gitpod-io/workspace-images/tree/master/base
RUN sudo install-packages \
      tmux \
      neovim

# Switch to the gitpod user to install user specific tools
USER gitpod
RUN curl -so "$HOME/.tmux.conf" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf && \
    curl -so "$HOME/.tmux.conf.local" https://raw.githubusercontent.com/gpakosz/.tmux/master/.tmux.conf.local
