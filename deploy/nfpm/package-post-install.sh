#!/usr/bin/env bash
set -Eeuo pipefail

env_data=""
HOPRD_CONFIG_FILE="/etc/hoprd/hoprd.cfg.yaml"

# Function to append data to the temporary variable
append_env_data() {
  env_data="${env_data}$1\n"
}

# Function to identify the public IP
get_public_ip() {
  # Get the public IP using a service like `curl` or `dig`
  public_ip=$(curl -s https://checkip.amazonaws.com)

  # Check if the public IP was retrieved successfully
  if [ -z "$public_ip" ]; then
    echo "Error: Unable to retrieve public IP."
    exit 1
  fi
  echo "$public_ip"
}

# Function to find an available port range
find_available_port_range() {
  local initial_port="$1"
  local required_ports="$2"

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

# Function to add the host address environment variable
add_host_address_env_var() {
  if [ -z "${HOPRD_HOST}" ]; then
       # Call the function to get the public IP
      public_ip=$(get_public_ip)
      # Find port available for p2p
      p2p_port=$(find_available_port_range 9091 1)
      # Set the HOPRD_HOST variable
      HOPRD_HOST="${public_ip}:${p2p_port}"
  fi
  append_env_data "# HOPRD_HOST is the public address and port of the HOPR node."
  append_env_data "HOPRD_HOST=${HOPRD_HOST}\n"
}

# Function to add the HOPRD_PASSWORD environment variable
add_hoprd_password_var() {
  if [ -z "${HOPRD_PASSWORD}" ]; then
    read -r -p "Enter HOPRD_PASSWORD (default: random 32 hex characters): " HOPRD_PASSWORD
    if [ -z "$HOPRD_PASSWORD" ]; then
      HOPRD_PASSWORD=$(openssl rand -hex 32)
    fi
  fi

  append_env_data "# HOPRD_PASSWORD is the password used to access the HOPR node identity file"
  append_env_data "HOPRD_PASSWORD=${HOPRD_PASSWORD}\n"
}

# Function to add the HOPRD_API_TOKEN environment variable
add_api_token_var() {
  if [ -z "${HOPRD_API_TOKEN}" ]; then
    read -r -p "Enter HOPRD_API_TOKEN (default: random 32 hex characters): " HOPRD_API_TOKEN
    if [ -z "$HOPRD_API_TOKEN" ]; then
      HOPRD_API_TOKEN=$(openssl rand -hex 32)
    fi
  fi

  append_env_data "# HOPRD_API_TOKEN is the token use to access the HOPR rest API"
  append_env_data "HOPRD_API_TOKEN=${HOPRD_API_TOKEN}\n"
}

# Function to add the HOPRD_SAFE_ADDRESS environment variable
add_safe_address_var() {
  if [ -z "${HOPRD_SAFE_ADDRESS}" ]; then
    echo "Safe address (HOPRD_SAFE_ADDRESS) is required. You can get it from https://hub.hoprnet.org"
    read -r -p "Enter HOPRD_SAFE_ADDRESS: " HOPRD_SAFE_ADDRESS
    # Validate that HOPRD_SAFE_ADDRESS is a valid Ethereum address
    if ! echo "$HOPRD_SAFE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
        echo "Invalid Safe Ethereum address format. Please enter a valid address."
        exit 1
    fi
  fi

  append_env_data "# HOPRD_SAFE_ADDRESS is ethereum address link to your safe"
  append_env_data "HOPRD_SAFE_ADDRESS=${HOPRD_SAFE_ADDRESS}\n"
}

# Function to add the HOPRD_MODULE_ADDRESS environment variable
add_module_address_var() {
  if [ -z "${HOPRD_MODULE_ADDRESS}" ]; then
    echo "Safe module address (HOPRD_MODULE_ADDRESS) is required. You can get it from https://hub.hoprnet.org"
    read -r -p "Enter HOPRD_MODULE_ADDRESS: " HOPRD_MODULE_ADDRESS
    # Validate that HOPRD_MODULE_ADDRESS is a valid Ethereum address
    if ! echo "$HOPRD_MODULE_ADDRESS" | grep -Eq "^0x[a-fA-F0-9]{40}$"; then
        echo "Invalid Safe Module Ethereum address format. Please enter a valid address."
        exit 1
    fi
  fi

  append_env_data "# HOPRD_MODULE_ADDRESS is ethereum address link to your safe module"
  append_env_data "HOPRD_MODULE_ADDRESS=${HOPRD_MODULE_ADDRESS}\n"
}

# Function to add the RPC provider environment variable
add_rpc_provider_var() {
  if [ -z "${HOPRD_PROVIDER}" ]; then
    echo "RPC provider (HOPRD_PROVIDER) is required. You can get it from http://chainlist.org/chain/100"
    read -r -p "Enter URL of RPC provider: " HOPRD_PROVIDER
    if ! echo "$HOPRD_PROVIDER" | grep -Eq "^https?://[a-zA-Z0-9.-]+(:[0-9]+)?(/.*)?$"; then
      echo "Invalid URL format. Please enter a valid URL."
      exit 1
    fi
  fi
  append_env_data "# HOPRD_PROVIDER is the RPC provider URL"
  append_env_data "HOPRD_PROVIDER=${HOPRD_PROVIDER}"
}

# Function to add the HOPRD_API_HOST environment variable
add_hoprd_api_host_var() {
  if [ -z "${HOPRD_API_HOST}" ]; then
    read -r -p "Enter HOPRD_API_HOST (default: 0.0.0.0): " HOPRD_API_HOST
    if [ -z "$HOPRD_API_HOST" ]; then
      HOPRD_API_HOST="0.0.0.0"
    fi
    # Validate that HOPRD_API_HOST is a standard IPv4 address
    if ! echo "$HOPRD_API_HOST" | grep -Eq "^([0-9]{1,3}\.){3}[0-9]{1,3}$"; then
      echo "Invalid IP address format for HOPRD_API_HOST. Please enter a valid IPv4 address."
      exit 1
    fi
  fi
  append_env_data "# HOPRD_API_HOST is the host interface for the HOPR API"
  append_env_data "HOPRD_API_HOST=${HOPRD_API_HOST}\n"
}

#Function to add the HOPRD_API_PORT environment variable
add_hoprd_api_port_var() {
  if [ -z "${HOPRD_API_PORT}" ]; then
    api_port=$(find_available_port_range 3001 1)
    read -r -p "Enter HOPRD_API_PORT (default: ${api_port}): " HOPRD_API_PORT
    # Validate that HOPRD_API_PORT is a valid port number
    if ! echo "$HOPRD_API_PORT" | grep -Eq "^[0-9]{1,5}$" || [ "$HOPRD_API_PORT" -lt 1 ] || [ "$HOPRD_API_PORT" -gt 65535 ]; then
      echo "Invalid port number for HOPRD_API_PORT. Please enter a valid port number between 1 and 65535."
      exit 1
    fi
  fi
  append_env_data "# HOPRD_API_PORT is the port for the HOPR API"
  append_env_data "HOPRD_API_PORT=${HOPRD_API_PORT}\n"
}

# Function to generate the environment file
generate_env_file() {
    # If the environment vars file not exists, automatically create it
    HOPRD_ENV_FILE="/etc/hoprd/hoprd.env"
    if [ ! -f "${HOPRD_ENV_FILE}" ]; then
        append_env_data "# This file contains custom installation configuration for HOPR node.\n\n"
        add_host_address_env_var
        add_hoprd_password_var
        add_api_token_var
        add_safe_address_var
        add_module_address_var
        add_rpc_provider_var
        add_hoprd_api_host_var
        add_hoprd_api_port_var

        # Write collected data to the environment file
        printf '%b\n' "$env_data" > "${HOPRD_ENV_FILE}"
        chmod 600 "${HOPRD_ENV_FILE}"
    else
        echo "The environment file located at ${HOPRD_ENV_FILE} already exists. Skipping generation."
    fi
}

# Function to generate the config file
generate_config_file() {
    # If the config file not exists, automatically create it
    if [ ! -f "${HOPRD_CONFIG_FILE}" ]; then
        echo "Generating HOPR node config file at ${HOPRD_CONFIG_FILE}..."
        cp /etc/hoprd/hoprd-sample.cfg.yaml "${HOPRD_CONFIG_FILE}"
        chmod 600 "${HOPRD_CONFIG_FILE}"
    else
        echo "The config file located at ${HOPRD_CONFIG_FILE} already exists. Some default config attributes might have changed in the new version. You might need to update it manually from sample config file located at /etc/hoprd/hoprd-sample.cfg.yaml"
    fi
}

# Function to generate the identity file
generate_identity_file() {
    # If the identity file not exists, automatically create it
    if [ ! -f "/etc/hoprd/hopr.id" ]; then
        echo "Generating HOPR node identity file at /etc/hoprd/hopr.id..."
        chmod 700 /etc/hoprd/
        IDENTITY_PASSWORD=${HOPRD_PASSWORD} hopli identity create -x hopr -d /etc/hoprd/
        mv /etc/hoprd/hopr0.id /etc/hoprd/hopr.id
    else
        echo "The identity file located at /etc/hoprd/hopr.id already exists. Skipping generation."
    fi
}

create_user_group() {
  # Create a user and group for the HOPR node if they do not exist
  if ! id -u hopr >/dev/null 2>&1; then
    echo "Creating user and group for HOPR node..."
    groupadd -r hoprd
    useradd --system -g hoprd --home /var/lib/hoprd --shell /usr/sbin/nologin -c "HOPR Node User" hoprd
    echo "Setting ownership and permissions for hoprd files..."
    chown -R hoprd:hoprd /etc/hoprd
    chown -R hoprd:hoprd /var/lib/hoprd
    chown hoprd:hoprd /usr/bin/hoprd
    chown hoprd:hoprd /usr/bin/hopli
    chmod 770 /etc/hoprd
    chmod 755 /usr/bin/hoprd
    chmod 755 /usr/bin/hopli
    # Add the logged-in user to the hoprd group
    if [ -n "$SUDO_USER" ]; then
      echo "Adding user '$SUDO_USER' to the hoprd group..."
      usermod -aG hoprd "$SUDO_USER"
    else
      echo "Could not identify the user who initiated the installation."
    fi
  else
    echo "User and group for HOPR node already exist."
  fi


}

# Function to start the HOPR node service
start_service() {
  systemctl daemon-reexec
  systemctl daemon-reload
  systemctl enable hoprd.service
  systemctl start hoprd.service
  echo "HOPR node installed successfully."
  
  # Wait for a few seconds to allow the service to initialize
  echo "Waiting for the HOPR node service to start..."
  sleep 30
  # Check the status of the service
  if ! systemctl is-active --quiet hoprd.service; then
    echo "Error: HOPR node service is not running. Please check the logs for more details."
    echo "You can check the logs with: journalctl -xeu hoprd.service -f"
    exit 1
  fi
}

# Function to show the HOPR node address
show_node_address() {
  node_address=$(hopli identity read -d /etc/hoprd/ | grep "Identity addresses:" | awk -F '[\\[\\]]' '{print $2}')
  echo "HOPR node address: ${node_address}"
  echo "Finish the onboarding of this node at https://hub.hoprnet.org by registering it and adding it to your safe."
}

# Main script execution starts here
echo "Starting HOPR node installation..."
generate_env_file
generate_config_file
generate_identity_file
create_user_group
start_service
show_node_address
echo "HOPR package installation completed successfully."
