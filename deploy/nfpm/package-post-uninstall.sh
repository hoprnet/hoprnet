#!/bin/bash
set -Eeo pipefail


delete_user_group() {
  if id -u hoprd >/dev/null 2>&1; then
    echo "Deleting user and group for HOPR node..."
    gpasswd -d "$SUDO_USER" hoprd >/dev/null
    chown "$SUDO_USER:$SUDO_USER" /etc/hoprd >/dev/null
    userdel -r hoprd
  else
    echo "User and group for HOPR node already deleted."
  fi
}

delete_folders() {
  rm -rf /var/lib/hoprd || true
  rm -rf /var/log/hoprd || true
}

delete_user_group
delete_folders
