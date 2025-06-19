#!/bin/sh


stop_service() {
    systemctl disable hoprd.service || true
    systemctl stop hoprd.service || true
    systemctl daemon-reexec
    systemctl daemon-reload
}

delete_user_group() {
  # Create a user and group for the HOPR node if they do not exist
  if id -u hopr >/dev/null 2>&1; then
    echo "Deleting user and group for HOPR node..."
    usermod -dG hoprd "$SUDO_USER" || true
    chown -R "$SUDO_USER:$SUDO_USER" /etc/hoprd || true
    userdel -r hoprd
    groupdel hoprd || true
  else
    echo "User and group for HOPR node already deleted."
  fi
}

delete_folders() {
    # Create a user and group for the HOPR node if they do not exist
    rm -rf /var/lib/hoprd || true
    rm /etc/hoprd/hoprd.cfg.yaml || true
}

stop_service
delete_user_group
delete_folders