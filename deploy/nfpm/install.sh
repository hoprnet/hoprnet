#!/bin/sh

HOPRD_CONFIG_FILE="/etc/hoprd/hoprd.cfg.yaml"
HOPRD_TEMPLATE_CONFIG_FILE="/etc/hoprd/hoprd-template.cfg.yaml"

# If the config file already exists, skip the generation
if [ -f "${HOPRD_CONFIG_FILE}" ]; then
    echo "${HOPRD_CONFIG_FILE} already exists. Skipping config generation. You might need to update it manually from template ${HOPRD_TEMPLATE_CONFIG_FILE}"
    systemctl daemon-reexec
    systemctl daemon-reload
    systemctl enable hoprd.service
    systemctl restart hoprd.service
    exit 0
fi

# Function to identify the public IP
get_public_ip() {
  # Get the public IP using a service like `curl` or `dig`
  public_ip=$(curl -s http://checkip.amazonaws.com)

  # Check if the public IP was retrieved successfully
  if [ -z "$public_ip" ]; then
    echo "Error: Unable to retrieve public IP."
    exit 1
  fi
  echo "$public_ip"
}

# Function to find an available port range
find_available_port_range() {
  local initial_port=$1
  local required_ports=$2

  # Loop through potential starting ports
  while true; do
    local end_port=$((initial_port + required_ports - 1))

    # Check if all ports in the range are available
    local all_ports_available=true
    for port in $(seq "$initial_port" "$end_port"); do
      if nc -z 127.0.0.1 "$port" 2>/dev/null; then
        initial_port=$((port + 1))
        all_ports_available=false
        break
      fi
    done

    # If all ports are available, return the range
    if [ "$all_ports_available" = true ]; then
      echo "${initial_port}:${end_port}"
      return 0
    fi
  done
}

if [ -z "$HOPRD_HOST" ]; then
    # Call the function to get the public IP
    HOPRD_HOST_ADDRESS=$(get_public_ip)
    # Find port available for p2p
    HOPRD_HOST_PORT=$(find_available_port_range 9091 1)
else
    HOPRD_HOST_ADDRESS=$(echo "$HOPRD_HOST" | cut -d':' -f1)
    HOPRD_HOST_PORT=$(echo "$HOPRD_HOST" | cut -d':' -f2)
fi

HOPRD_API_HOST=${HOPRD_API_HOST:-0.0.0.0}
HOPRD_PASSWORD=${HOPRD_PASSWORD:-$(openssl rand -hex 32)}
HOPRD_API_TOKEN=${HOPRD_API_TOKEN:-$(openssl rand -hex 32)}

if [ -z "$HOPRD_API_PORT" ]; then
    HOPRD_API_PORT=$(find_available_port_range 3001 1)
fi

HOPRD_PROVIDER=${HOPRD_PROVIDER:-}
if [ -z "$HOPRD_PROVIDER" ]; then
    read -r -p "Enter url of rpc provider: " HOPRD_PROVIDER
fi

if ! echo "$HOPRD_PROVIDER" | grep -Eq "^https?://[a-zA-Z0-9.-]+(:[0-9]+)?(/.*)?$"; then
    echo "Invalid URL format. Please enter a valid URL."
    exit 1 # Invalid URL
fi

HOPRD_SAFE_ADDRESS=${HOPRD_SAFE_ADDRESS:-}
if [ -z "$HOPRD_SAFE_ADDRESS" ]; then
    echo "Safe address (HOPRD_SAFE_ADDRESS) is required. You can get it from https://hub.hoprnet.org"
    read -p "Enter HOPRD_SAFE_ADDRESS: " HOPRD_SAFE_ADDRESS
fi

# Validate that HOPRD_SAFE_ADDRESS is a valid Ethereum address
if ! echo "$HOPRD_SAFE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
    echo "Invalid Safe Ethereum address format. Please enter a valid address."
    exit 1
fi

HOPRD_MODULE_ADDRESS=${HOPRD_MODULE_ADDRESS:-}
if [ -z "$HOPRD_MODULE_ADDRESS" ]; then
    echo "Module address (HOPRD_MODULE_ADDRESS) is required. You can get it from https://hub.hoprnet.org"
    read -p "Enter HOPRD_MODULE_ADDRESS: " HOPRD_MODULE_ADDRESS
fi

# Validate that HOPRD_MODULE_ADDRESS is a valid Ethereum address
if ! echo "$HOPRD_MODULE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
    echo "Invalid Safe Module Ethereum address format. Please enter a valid address."
    exit 1
fi

envsubst < "${HOPRD_TEMPLATE_CONFIG_FILE}" > ${HOPRD_CONFIG_FILE}
chmod 600 "${HOPRD_CONFIG_FILE}"

IDENTITY_PASSWORD=${HOPRD_PASSWORD} hopli identity create -x hopr -d /etc/hoprd/
mv /etc/hoprd/hopr0.id /etc/hoprd/hopr.id
rm /etc/hoprd/hoprd-template.cfg.yaml

systemctl daemon-reexec
systemctl daemon-reload
systemctl enable hoprd.service
systemctl start hoprd.service

echo "HOPR node installed successfully."
echo "Finish node onboarding at https://hub.hoprnet.org"
echo "You can check the logs with: journalctl -xeu hoprd.service -f"