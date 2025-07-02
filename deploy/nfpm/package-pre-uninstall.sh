#!/usr/bin/env sh

stop_service() {
  systemctl disable hoprd.service || true
  systemctl stop hoprd.service || true
  systemctl daemon-reexec
  systemctl daemon-reload
}

delete_user_group() {
  if id -u hoprd >/dev/null 2>&1; then
    echo "Deleting user and group for HOPR node..."
    gpasswd -d "$SUDO_USER" hoprd || true
    chown -R "$SUDO_USER:$SUDO_USER" /etc/hoprd || true
    userdel -r hoprd
  else
    echo "User and group for HOPR node already deleted."
  fi
}

delete_folders() {
  rm -rf /var/lib/hoprd || true
  rm -rf /var/log/hoprd || true
}

stop_service
delete_user_group
delete_folders
