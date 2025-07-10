class Hoprd < Formula
  desc "HOPR Node"
  homepage "https://hoprnet.org"
  version "2.2.3"

  on_macos do
    on_arm do
      url "file://#{ENV['HOME']}/Documents/github/hoprnet/dist/bin/hoprd-aarch64-darwin"
      sha256 "60589aca5513b0528c10c9b5e728015279ba9eeef7122b6cf213bffe4bf1a054"
    end

    on_intel do
      url "https://github.com/hoprnet/hoprd/releases/download/v#{version}/hoprd-x86_64-darwin"
      sha256 "sha256-for-macos-x86_64-binary"
    end
  end

  resource "hopli" do
    on_macos do
      on_arm do
        url "file://#{ENV['HOME']}/Documents/github/hoprnet/dist/bin/hopli-aarch64-darwin"
        sha256 "a863f7b7ab37ce3429c41102e78ad43c6d3bb5a4db22e6284c0ce3fdf8c7d8fd"
      end

      on_intel do
        url "https://github.com/hoprnet/hoprd/releases/download/v#{version}/hopli-x86_64-darwin"
        sha256 "sha256-for-hopli-x86_64"
      end
    end
  end

  depends_on "logrotate"

  def install
    # Install binaries
    cpu_arch = Hardware::CPU.arm? ? "aarch64" : "x86_64"
    bin.install "hoprd-#{cpu_arch}-darwin" => "hoprd"
    chmod 0755, bin/"hoprd"

    resource("hopli").stage do
      bin.install "hopli-#{cpu_arch}-darwin" => "hopli"
      chmod 0755, bin/"hopli"
    end

    (bin/"hoprd-service").write <<~SH
      #!/bin/bash
      set -a
      set -euo pipefail
  
      # Paths
      ENV_FILE="#{etc}/hoprd/hoprd.env"
      CONFIG_FILE="#{etc}/hoprd/hoprd.cfg.yaml"
      LOGROTATE_CONF="#{etc}/hoprd/logrotate.conf"
      LOGROTATE_STATE="#{var}/log/hoprd/logrotate.status"
      NOW=$(date +%s)
      LOGROTATE_INTERVAL_SEC=86400 # 24 hours
      LOGROTATE_LOG="#{var}/log/hoprd/logrotate.out"

      # Load environment variables
      if [ -f "$ENV_FILE" ]; then
        source "$ENV_FILE"
      fi

      # === Graceful shutdown ===
      cleanup() {
        echo "[$(date)] Stopping hoprd-service and logrotate loop..." >> "$LOGROTATE_LOG"
        if [ -n "${LOGROTATE_PID:-}" ]; then
          kill "$LOGROTATE_PID" 2>/dev/null || true
          wait "$LOGROTATE_PID" 2>/dev/null || true
        fi
        exit 0
      }

      trap cleanup INT TERM EXIT

      # === Start logrotate loop in background ===
      (
        while true; do
          echo "[$(date)] Running logrotate" >> "$LOGROTATE_LOG"
          "#{Formula["logrotate"].opt_sbin}/logrotate" --state "$LOGROTATE_STATE" "$LOGROTATE_CONF" >> "$LOGROTATE_LOG" 2>&1 || true
          sleep "$LOGROTATE_INTERVAL_SEC"
        done
      ) &
      LOGROTATE_PID=$!

      # Start the hoprd application
      exec "#{bin}/hoprd" --configurationFilePath "$CONFIG_FILE" "$@"
    SH
    chmod 0755, bin/"hoprd-service"

    # Create directories 
    (var/"lib/hoprd").mkpath
    (var/"lib/hoprd").mkpath
    (etc/"hoprd").mkpath

    # Installation tasks
    generate_hoprd_config
    create_log_config

  end

  def post_install
    env_file = etc/"hoprd/hoprd.env"
    config = generate_env_vars(env_file)
    node_address = generate_identity_file(config["HOPRD_PASSWORD"])
    show_installation_summary(config, node_address)
  end


  service do
    run bin/"hoprd-service"
    working_dir var/"lib/hoprd"
    log_path var/"log/hoprd/hoprd.log"
    error_log_path var/"log/hoprd/hoprd.log"
    keep_alive true
    run_at_load false # Cannot run until all environment variables are set correctly
    process_type :interactive
  end

  test do
    system "#{bin}/hoprd", "--version"
    system "#{bin}/hopli", "--version"
  end

  private

  require "json"
  require "open3"
  require "uri"

  def create_log_config
    log_conf_path = etc/"hoprd/logrotate.conf"

    if File.exist?(log_conf_path)
      FileUtils.rm(log_conf_path)
    end
    log_conf_path.write <<~EOS
      #{var}/log/hoprd/hoprd.log {
        size 10M
        rotate 5
        compress
        missingok
        notifempty
        copytruncate
      }
    EOS
  end
  
  def generate_hoprd_config
    config_path = etc/"hoprd/hoprd.cfg.yaml"

    if File.exist?(config_path)
      backup_path = "#{config_path}.bak.#{Time.now.to_i}"
      ohai "Backing up existing hoprd.cfg.yaml to #{backup_path}"
      FileUtils.mv(config_path, backup_path)
    end

    config_path.write <<~EOS
      ---
      hopr:
        db:
          data: #{var}/lib/hoprd
        strategy:
          on_fail_continue: true
          allow_recursive: true
          strategies:
          - !Aggregating
             aggregation_threshold: 330
             unrealized_balance_ratio: 0.95
             aggregate_on_channel_close: true
          - !AutoRedeeming
             redeem_only_aggregated: true
             minimum_redeem_ticket_value: "2 wxHOPR"
          - !ClosureFinalizer
             max_closure_overdue: 300
        chain:
          announce: true
          keep_logs: true
          fast_sync: true
        safe_module:
          safe_transaction_service_provider: https://safe-transaction.prod.hoprnet.link/
      identity:
        file: "#{etc}/hoprd/hopr.id"
      api:
        enable: true
    EOS
    chmod 0640, config_path
  end

  def generate_env_vars(env_file)
    config = {}
    # Source 1: Read from hoprd.env file
    if File.exist?(env_file)
      File.readlines(env_file).each do |line|
        key, value = line.strip.split('=', 2)
        config[key] = value if key && value
      end
    else
      # Source 2: Apply default values if not set
      config["HOPRD_HOST"] ||= "#{detect_public_ip}:9091"    
      config["HOPRD_PASSWORD"] ||= `openssl rand -hex 32`.chomp
      config["HOPRD_API_TOKEN"] ||= `openssl rand -hex 32`.chomp
      config["HOPRD_PROVIDER"] ||= "http://localhost:8545"
      config["HOPRD_SAFE_ADDRESS"] ||= "0x0000000000000000000000000000000000000000"
      config["HOPRD_MODULE_ADDRESS"] ||= "0x0000000000000000000000000000000000000000"
      config["HOPRD_API_HOST"] ||= "0.0.0.0"
      config["HOPRD_API_PORT"] ||= "3001"
      config["HOPRD_NETWORK"] ||= "dufour"
      write_env_file(env_file, config)
    end
    config
  end

  def write_env_file(env_file, config)
    env_content = <<~EOS
        # Auto-generated HOPR configuration (#{Time.now.utc.iso8601})

        # HOPRD_HOST is the public address and port of the HOPR node.
        HOPRD_HOST=#{config["HOPRD_HOST"]}

        # HOPRD_PASSWORD password to access the identity file
        HOPRD_PASSWORD=#{config["HOPRD_PASSWORD"]}

        # HOPRD_API_TOKEN is the token use to access the HOPR rest API
        HOPRD_API_TOKEN=#{config["HOPRD_API_TOKEN"]}

        # HOPRD_PROVIDER is the RPC provider URL
        HOPRD_PROVIDER=#{config["HOPRD_PROVIDER"]}

        # HOPRD_SAFE_ADDRESS is ethereum address link to your safe and shown in https://hub.hoprnet.org
        HOPRD_SAFE_ADDRESS=#{config["HOPRD_SAFE_ADDRESS"]}

        # HOPRD_MODULE_ADDRESS is ethereum address link to your safe module and shown in https://hub.hoprnet.org
        HOPRD_MODULE_ADDRESS=#{config["HOPRD_MODULE_ADDRESS"]}

        # HOPRD_API_HOST is the host interface for the HOPR API
        HOPRD_API_HOST=#{config["HOPRD_API_HOST"]}

        # HOPRD_API_PORT is the port for the HOPR API
        HOPRD_API_PORT=#{config["HOPRD_API_PORT"]}

        # HOPRD_NETWORK posible values are: dufour, rotsee
        HOPRD_NETWORK=#{config["HOPRD_NETWORK"]}

        # Log Level
        RUST_LOG=info
        # RUST_LOG=debug,libp2p_swarm=debug,libp2p_mplex=debug,multistream_select=debug,libp2p_tcp=debug,libp2p_dns=info,sea_orm=info,sqlx=info
    EOS

    env_file.write(env_content)
    chmod 0644, env_file
    ohai "Configuration written to #{env_file}"
  end

  def generate_identity_file(password)
    identity_file = etc/"hoprd/hopr.id"
    dir = etc/"hoprd"
    ENV["IDENTITY_PASSWORD"] = password
    if !identity_file.exist?
        ohai "Generating HOPR node identity file at #{identity_file}"
        cmd = "#{bin}/hopli identity create -x hopr -d #{dir} > /dev/null 2>&1"
        success = system("/bin/sh", "-c", cmd)

        if (generated_file = etc/"hoprd/hopr0.id").exist?
          File.rename(generated_file, identity_file)
          chmod 0640, identity_file
          node_address = get_node_address(password)
        end
    else
        # Verify existing identity file
        cmd = "#{bin}/hopli identity read --identity-from-path #{identity_file}" #  > /dev/null 2>&1
        stdout, stderr, status = Open3.capture3(cmd)
        if status.success?
          ohai "Using existing identity file at #{identity_file}"
          node_address = get_node_address(password)
        else
          opoo "Command failed: #{cmd}"
          opoo "STDOUT: #{stdout}" unless stdout.empty?
          opoo "STDERR: #{stderr}" unless stderr.empty?
          onoe <<~EOS
            Could not read the identity file at #{identity_file}
            Please check the password set in HOPRD_PASSWORD for that identity file.
            You can either:
            1. Set the correct HOPRD_PASSWORD and reinstall
            2. Remove the identity file and let the installer generate a new one:
            rm #{identity_file}
            brew postinstall #{name}
          EOS
          exit 1
        end
    end
    node_address
  end

  def get_node_address(password)
    output = `IDENTITY_PASSWORD=#{password.shellescape} #{bin}/hopli identity read -d #{etc}/hoprd/ 2>&1`
    node_address = output.lines.grep(/Identity addresses:/).first.to_s.match(/\[(.*?)\]/).to_a[1]
    unless node_address
        onoe "Failed to read node address from existing identity file"
        exit 1
    end
    node_address.strip
  end

  def detect_public_ip
    Timeout.timeout(3) do
        `curl -sf https://api.ipify.org`.chomp.tap do |ip|
        return ip if ip.match?(/\A\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\z/)
        end
    end
    rescue Timeout::Error, StandardError
        "0.0.0.0"
  end

  def is_valid_ethereum_address?(address)
    isInvalid = address.nil? || address.empty? || address == "0x0000000000000000000000000000000000000000"
    isValid = address.match?(/\A0x[a-fA-F0-9]{40}\z/)
    return isValid && !isInvalid
    rescue StandardError
      false
  end

  def is_valid_rpc_provider?(url)
    payload = {
      jsonrpc: "2.0",
      method: "web3_clientVersion",
      params: [],
      id: 1
    }.to_json

    uri = URI(url)
    unless uri.is_a?(URI::HTTP) || uri.is_a?(URI::HTTPS)
      return false
    end

    stdout, stderr, status = Open3.capture3(
      "curl", "-s", "-X", "POST",
      "-H", "Content-Type: application/json",
      "--data", payload,
      url
    )

    return false unless status.success?

    begin
      response = JSON.parse(stdout)
      return response.key?("result") && !response["result"].nil?
    rescue JSON::ParserError
      return false
    end


  end

  def show_installation_summary(config, node_address)
    invalid_variables = []
    if !is_valid_ethereum_address?(config["HOPRD_SAFE_ADDRESS"])
      invalid_variables << "HOPRD_SAFE_ADDRESS"
    end
    if !is_valid_ethereum_address?(config["HOPRD_MODULE_ADDRESS"])
      invalid_variables << "HOPRD_MODULE_ADDRESS"
    end
    if !is_valid_rpc_provider?(config["HOPRD_PROVIDER"])
      invalid_variables << "HOPRD_PROVIDER"
    end
    unless invalid_variables.empty?
      opoo <<~EOS

        ----------------------------------------------------------------------------------------------------
        The following environment variables are invalid: #{invalid_variables.join(", ")}
        Please update them in #{etc}/hoprd/hoprd.env before proceeding.
        ----------------------------------------------------------------------------------------------------
      EOS
    end
    ohai "Generated HOPR configuration:"
    puts <<~EOS
        API Endpoint:       http://#{config["HOPRD_HOST"]}:#{config["HOPRD_API_PORT"]} or http://localhost:#{config["HOPRD_API_PORT"]}
        Safe Address:       #{config["HOPRD_SAFE_ADDRESS"]}
        Module Address:     #{config["HOPRD_MODULE_ADDRESS"]}
        Node Address:       #{node_address}
        Config Directory:   #{etc}/hoprd
        Data Directory:     #{var}/lib/hoprd
        Log Location:       #{var}/log/hoprd
        HOPRd Binary:       #{bin}/hoprd
        HOPLi Binary:       #{bin}/hopli
        
        Next Steps:
            - Edit file #{etc}/hoprd/hoprd.env to set the appropiate environment variables.
            - Edit file #{etc}/hoprd/hoprd.cfg.yaml to customize your node settings.
            - Register your node address at: https://hub.hoprnet.org
            - Add funds to your node address: #{node_address}
            - Start the service: brew services start hoprd
        Note: Your API token and password are stored in #{etc}/hoprd/hoprd.env (Keep this file secure!)
    EOS

  end



end