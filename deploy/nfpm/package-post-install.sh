#!/bin/bash
set -Eeo pipefail

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

find_available_port() {
  local initial_port="$1"
  local required_ports=1

  port_range=$(find_available_port_range "$initial_port" "$required_ports")
  echo "$port_range" | cut -d':' -f1
}

# Function to add the host address environment variable
add_host_address_env_var() {
  if [ -z "${HOPRD_HOST}" ]; then
    # Call the function to get the public IP
    public_ip=$(get_public_ip)
    # Find port available for p2p
    p2p_port=$(find_available_port 9091)
    # Set the HOPRD_HOST variable
    HOPRD_HOST="${public_ip}:${p2p_port}"
  fi
  append_env_data "# HOPRD_HOST is the public address and port of the HOPR node."
  append_env_data "HOPRD_HOST=${HOPRD_HOST}\n"
}

# Function to add the HOPRD_PASSWORD environment variable
add_hoprd_password_var() {
  if [ -z "${HOPRD_PASSWORD}" ]; then
    HOPRD_PASSWORD=$(openssl rand -hex 32)
  fi
  append_env_data "# HOPRD_PASSWORD is the password used to access the HOPR node identity file"
  append_env_data "HOPRD_PASSWORD=${HOPRD_PASSWORD}\n"
}

# Function to add the HOPRD_API_TOKEN environment variable
add_api_token_var() {
  if [ -z "${HOPRD_API_TOKEN}" ]; then
    HOPRD_API_TOKEN=$(openssl rand -hex 32)
  fi
  append_env_data "# HOPRD_API_TOKEN is the token use to access the HOPR rest API"
  append_env_data "HOPRD_API_TOKEN=${HOPRD_API_TOKEN}\n"
}

# Function to add the HOPRD_SAFE_ADDRESS and HOPRD_MODULE_ADDRESS environment variable
add_safe_addresses_var() {
  append_env_data "# HOPRD_SAFE_ADDRESS is ethereum address link to your safe and shown in https://hub.hoprnet.org"
  append_env_data "HOPRD_SAFE_ADDRESS=${HOPRD_SAFE_ADDRESS}\n"
  append_env_data "# HOPRD_MODULE_ADDRESS is ethereum address link to your safe module and shown in https://hub.hoprnet.org"
  append_env_data "HOPRD_MODULE_ADDRESS=${HOPRD_MODULE_ADDRESS}\n"
}

# Function to add the RPC provider environment variable
add_rpc_provider_var() {
  if [ -z "${HOPRD_PROVIDER}" ]; then
    # Default to a local RPC provider if not set
    HOPRD_PROVIDER="http://localhost:8545"
  fi
  append_env_data "# HOPRD_PROVIDER is the RPC provider URL"
  append_env_data "HOPRD_PROVIDER=${HOPRD_PROVIDER}\n"
}

# Function to add the HOPRD_API_HOST environment variable
add_hoprd_api_host_var() {
  if [ -z "${HOPRD_API_HOST}" ]; then
    HOPRD_API_HOST="0.0.0.0"
  fi
  append_env_data "# HOPRD_API_HOST is the host interface for the HOPR API"
  append_env_data "HOPRD_API_HOST=${HOPRD_API_HOST}\n"
}

# Function to add the HOPRD_NETWORK environment variable
add_network() {
  if [ -z "${HOPRD_NETWORK}" ]; then
    HOPRD_NETWORK="dufour"
  fi
  append_env_data "# HOPRD_NETWORK posible values are: dufour, rotsee"
  append_env_data "HOPRD_NETWORK=${HOPRD_NETWORK}\n"
}

#Function to add the HOPRD_API_PORT environment variable
add_hoprd_api_port_var() {
  if [ -z "${HOPRD_API_PORT}" ]; then
    HOPRD_API_PORT=$(find_available_port 3001)
  fi
  append_env_data "# HOPRD_API_PORT is the port for the HOPR API"
  append_env_data "HOPRD_API_PORT=${HOPRD_API_PORT}\n"
}

add_log_level_var() {
  # Set the log level to info by default
  if [ -z "${RUST_LOG}" ]; then
    RUST_LOG="info"
  fi
  append_env_data "# RUST_LOG is the log level for the HOPR node"
  append_env_data "RUST_LOG=${RUST_LOG}"
  append_env_data "# RUST_LOG=debug,libp2p_swarm=debug,libp2p_mplex=debug,multistream_select=debug,libp2p_tcp=debug,libp2p_dns=info,sea_orm=info,sqlx=info\n"
}

# Function to generate the environment file
generate_env_file() {
  # If the environment vars file not exists, automatically create it
  HOPRD_ENV_FILE="/etc/hoprd/hoprd.env"
  if [ ! -f "${HOPRD_ENV_FILE}" ]; then
    append_env_data "# This file contains custom installation configuration for HOPR node.\n"
    add_host_address_env_var
    add_hoprd_password_var
    add_api_token_var
    add_safe_addresses_var
    add_rpc_provider_var
    add_hoprd_api_host_var
    add_hoprd_api_port_var
    add_network
    add_log_level_var
    # Write collected data to the environment file
    mkdir -p "$(dirname "${HOPRD_ENV_FILE}")"
    chmod 750 "$(dirname "${HOPRD_ENV_FILE}")"
    printf '%b\n' "$env_data" >"${HOPRD_ENV_FILE}"
    chmod 640 "${HOPRD_ENV_FILE}"
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
  else
    echo "The config file located at ${HOPRD_CONFIG_FILE} already exists. Some default config attributes might have changed in the new version. You might need to update it manually from sample config file located at /etc/hoprd/hoprd-sample.cfg.yaml"
  fi
}

# Function to generate the identity file
generate_identity_file() {
  # If the identity file not exists, automatically create it
  if [ ! -f "/etc/hoprd/hopr.id" ]; then
    echo "Generating HOPR node identity file at /etc/hoprd/hopr.id..."
    if IDENTITY_PASSWORD=${HOPRD_PASSWORD} hopli identity create -x hopr -d /etc/hoprd/; then
      if [ -f /etc/hoprd/hopr0.id ]; then
        mv /etc/hoprd/hopr0.id /etc/hoprd/hopr.id
        chmod 644 /etc/hoprd/hopr.id
        show_node_address
      else
        echo "Error: Identity file was not created at expected location /etc/hoprd/hopr0.id"
        exit 1
      fi
    else
      echo "Error: Failed to create the identity file. Please check the HOPRD_PASSWORD environment variable."
      exit 1
    fi
  else
    if IDENTITY_PASSWORD=${HOPRD_PASSWORD} hopli identity read --identity-from-path /etc/hoprd/hopr.id | grep "^Identity addresses: \[\]" >/dev/null 2>&1; then
      echo "Could not read the identity file at /etc/hoprd/hopr.id. Please check the password set at HOPRD_PASSWORD for that identity file."
      exit 1
    else
      echo "The identity file located at /etc/hoprd/hopr.id already exists and is valid, skipping generation."
    fi
  fi
}

create_user_group() {
  # Create a user and group for the HOPR node if they do not exist
  if ! id -u hoprd >/dev/null 2>&1; then
    echo "Creating user and group for HOPR node..."
    groupadd -r hoprd
    mkdir -p /var/lib/hoprd /var/log/hoprd
    useradd --system -g hoprd --home /var/lib/hoprd --shell /usr/sbin/nologin -c "HOPR Node User" hoprd
    echo "Setting ownership and permissions for hoprd files..."
    chown hoprd:hoprd /etc/hoprd
    chown hoprd:hoprd /var/lib/hoprd /var/log/hoprd
    chown hoprd:hoprd /usr/bin/hoprd /usr/bin/hopli
    chmod 755 /usr/bin/hoprd /usr/bin/hopli /var/log/hoprd
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
  if [ -d /run/systemd/system ]; then
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
  else
    echo "Systemd not found, using service command to start HOPR node."
    echo "Starting HOPR node manually..."
  fi

}

# Function to show the HOPR node address
show_node_address() {
  node_address=$(IDENTITY_PASSWORD=${HOPRD_PASSWORD} hopli identity read -d /etc/hoprd/ | grep "Identity addresses:" | awk -F '[\\[\\]]' '{print $2}')
  echo "HOPR node address: ${node_address}"
  echo "Finish the onboarding of this node at https://hub.hoprnet.org by registering it and adding it to your safe."
}

# Main script execution starts here
echo "Starting HOPR node installation..."
generate_env_file
generate_identity_file
generate_config_file
create_user_group
start_service
echo "HOPR package installation completed successfully."
