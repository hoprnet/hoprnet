"""
Demo test: verify a HOPR node, inspect the network, and download a file through a session.

Steps:
  1. Wait for the node to become ready
  2. Verify at least 3 outgoing open channels
  3. Download the network graph (DOT) and render it via Graphviz
  4. Open an N-hop session (configurable: 1, 2, or 3 hops)
  5. Download a file through the HOPR session
  6. Parse the node log file for session diagnostics

Usage:
    python tools/test_n_hop_session.py --api http://127.0.0.1:3001/api/v4
    python tools/test_n_hop_session.py --api http://127.0.0.1:3001/api/v4 --hops 2
    python tools/test_n_hop_session.py --api http://127.0.0.1:3001/api/v4 --log ./rotsee.log
    python tools/test_n_hop_session.py --api http://127.0.0.1:3001/api/v4 \\
        --url "https://www.fileexamples.com/api/sample-file?format=png&size=2097152"
"""

import argparse
import json
import logging
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path
from urllib.parse import urlparse

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
log = logging.getLogger(__name__)

DEFAULT_URL = "https://www.fileexamples.com/api/sample-file?format=png&size=2097152"
MIN_OUTGOING_CHANNELS = 3


def parse_args():
    p = argparse.ArgumentParser(description="HOPR demo: network inspection + file download through session")

    p.add_argument(
        "--api",
        default="http://127.0.0.1:3001/api/v4",
        help="Client node API base URL (default: http://127.0.0.1:3001/api/v4)",
    )
    p.add_argument("--token", default=None, help="API bearer token (omit if auth is disabled)")
    p.add_argument(
        "--hops", type=int, default=1, choices=[1, 2, 3], help="Number of hops for session routing (default: 1)"
    )
    p.add_argument(
        "--destination", default=None, help="Destination peer address; if omitted, picks the best-scoring peer"
    )
    p.add_argument(
        "--url", default=DEFAULT_URL, help=f"URL to download through the HOPR session (default: {DEFAULT_URL})"
    )
    p.add_argument(
        "--output",
        default="downloaded_file",
        help="Output file path for the downloaded content (default: downloaded_file)",
    )
    p.add_argument(
        "--graph-output",
        default="network_graph",
        help="Base name for graph output files (.dot and .png, default: network_graph)",
    )
    p.add_argument("--log", default="./rotsee.log", help="Path to the hoprd log file (default: ./rotsee.log)")
    p.add_argument(
        "--ready-timeout", type=int, default=120, help="Seconds to wait for the node to become ready (default: 120)"
    )
    p.add_argument("--timeout", type=int, default=120, help="Socket/download timeout in seconds (default: 120)")
    p.add_argument(
        "--min-peer-score",
        type=float,
        default=0.0,
        help="Minimum quality score to consider a peer; 0 = any peer with probe data (default: 0.0)",
    )
    p.add_argument(
        "--min-channel-balance", type=float, default=0.1, help="Minimum channel balance in wxHOPR (default: 0.1)"
    )
    p.add_argument(
        "--skip-graph-render",
        action="store_true",
        help="Skip rendering DOT to PNG (useful if graphviz is not installed)",
    )
    p.add_argument(
        "--address-file", default=None, help="Write our node's onchain address to this file (for downstream scripts)"
    )
    p.add_argument("-v", "--verbose", action="store_true", help="Enable debug logging")

    return p.parse_args()


# ---------------------------------------------------------------------------
# HTTP helpers
# ---------------------------------------------------------------------------


def api_request(base_url, method, path, body=None, token=None, timeout=30):
    url = f"{base_url}{path}"
    data = json.dumps(body).encode() if body else None
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"

    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            content_type = resp.headers.get("Content-Type", "")
            raw = resp.read()
            if "application/json" in content_type:
                return json.loads(raw) if raw else None
            return raw.decode("utf-8", errors="replace") if raw else None
    except urllib.error.HTTPError as e:
        body_text = e.read().decode()
        log.error(f"{method} {url} failed: {e.code} {body_text}")
        raise


def wait_for_ready(base_url, token, timeout=60):
    deadline = time.time() + timeout
    last_err = None
    while time.time() < deadline:
        try:
            req = urllib.request.Request(f"{base_url}/readyz")
            if token:
                req.add_header("Authorization", f"Bearer {token}")
            with urllib.request.urlopen(req, timeout=5):
                return
        except urllib.error.HTTPError as e:
            # Any HTTP response means the node is up and listening
            log.info(f"Node is up (got HTTP {e.code} from readyz)")
            return
        except Exception as e:
            last_err = e
            time.sleep(2)
    raise RuntimeError(f"Node at {base_url} not ready after {timeout}s (last error: {last_err})")


# ---------------------------------------------------------------------------
# Node API wrappers
# ---------------------------------------------------------------------------


def get_address(base_url, token):
    data = api_request(base_url, "GET", "/account/addresses", token=token)
    return data["native"]


def get_peers(base_url, token):
    """Returns connected peers from /network/connected with graph-based scores."""
    data = api_request(base_url, "GET", "/network/connected", token=token)
    return data if data else []


def get_outgoing_channels(base_url, token):
    data = api_request(base_url, "GET", "/channels?fullTopology=false&includingClosed=false", token=token)
    return data.get("outgoing", []) if data else []


def get_ticket_price(base_url, token):
    data = api_request(base_url, "GET", "/network/price", token=token)
    return data["price"]


def parse_balance(balance_str):
    parts = balance_str.strip().split()
    return float(parts[0])


def open_channel(base_url, token, destination, amount):
    body = {"destination": destination, "amount": amount}
    return api_request(base_url, "POST", "/channels", body=body, token=token)


def fund_channel(base_url, token, channel_id, amount):
    body = {"amount": amount}
    return api_request(base_url, "POST", f"/channels/{channel_id}/fund", body=body, token=token)


def get_network_graph(base_url, token):
    """Fetch the network graph in DOT format."""
    return api_request(base_url, "GET", "/network/graph", token=token)


def get_node_info(base_url, token):
    return api_request(base_url, "GET", "/node/info", token=token)


# ---------------------------------------------------------------------------
# Step 1: Verify channels
# ---------------------------------------------------------------------------


def ensure_channels_to_best_peers(api, token, min_peer_score, min_channel_balance):
    """Ensure we have open, funded channels to the best-scoring peers.

    Waits up to 60s for peers with non-zero scores to appear, then opens
    channels to them. Returns the list of open channels to active peers.
    """
    # Wait for peers with actual probe scores
    deadline = time.time() + 180
    scored_peers = []
    while time.time() < deadline:
        peers = get_peers(api, token)
        scored_peers = [p for p in peers if p.get("score", 0) > 0 or p.get("probeRate", 0) > 0]
        scored_peers.sort(key=lambda p: p.get("score", 0), reverse=True)
        if len(scored_peers) >= MIN_OUTGOING_CHANNELS:
            break
        remaining = int(deadline - time.time())
        log.info(
            f"Waiting for scored peers... {len(scored_peers)} found "
            f"(need {MIN_OUTGOING_CHANNELS}, {remaining}s remaining)"
        )
        time.sleep(5)

    log.info(f"Peers with score >= {min_peer_score}: {len(scored_peers)}")
    for p in scored_peers[:10]:
        score = p.get("score", 0)
        log.info(f"  {p['address'][:16]}... score={score:.4f} latency={p.get('averageLatency', 0)}ms")

    if len(scored_peers) < MIN_OUTGOING_CHANNELS:
        raise RuntimeError(
            f"Only {len(scored_peers)} peers with score >= {min_peer_score}, need at least {MIN_OUTGOING_CHANNELS}"
        )

    # Get existing channels
    channels = get_outgoing_channels(api, token)
    open_channels = {ch["peerAddress"]: ch for ch in channels if ch["status"] == "Open"}

    # Ensure channels to the top peers
    target_peers = scored_peers[: max(MIN_OUTGOING_CHANNELS, len(scored_peers))]
    funding_str = f"{min_channel_balance} wxHOPR"
    changed = False

    for peer in target_peers:
        addr = peer["address"]
        score = peer.get("score", 0)

        if addr in open_channels:
            log.info(f"  {addr[:16]}... score={score:.4f}: channel OK (balance={open_channels[addr]['balance']})")
        else:
            log.info(f"  {addr[:16]}... score={score:.4f}: opening channel with {funding_str}")
            try:
                open_channel(api, token, addr, funding_str)
                changed = True
            except Exception as e:
                log.warning(f"    Failed to open: {e}")

    if changed:
        log.info("Waiting 10s for channel state propagation...")
        time.sleep(10)

    # Re-fetch and return active channels
    channels = get_outgoing_channels(api, token)
    active_addresses = {p["address"] for p in scored_peers}
    active_channels = [ch for ch in channels if ch["status"] == "Open" and ch["peerAddress"] in active_addresses]

    log.info(f"Result: {len(active_channels)} open channels to scored peers (need >= {MIN_OUTGOING_CHANNELS})")
    for ch in active_channels:
        addr = ch["peerAddress"]
        peer = next((p for p in scored_peers if p["address"] == addr), {})
        score = peer.get("score", 0)
        log.info(f"  Channel {ch['id'][:16]}... -> {addr[:16]}... balance={ch['balance']} peer_score={score:.4f}")

    if len(active_channels) < MIN_OUTGOING_CHANNELS:
        raise RuntimeError(
            f"Need at least {MIN_OUTGOING_CHANNELS} open channels to scored peers, but only have {len(active_channels)}"
        )

    return active_channels


# ---------------------------------------------------------------------------
# Step 2: Network graph
# ---------------------------------------------------------------------------


def fetch_and_render_graph(api, token, base_name, skip_render):
    """Download the DOT graph and optionally render to PNG."""
    dot_content = get_network_graph(api, token)
    dot_path = f"{base_name}.dot"

    with open(dot_path, "w") as f:
        f.write(dot_content)
    log.info(f"Network graph saved to {dot_path} ({len(dot_content)} bytes)")

    # Count nodes and edges for a quick summary
    node_count = len(re.findall(r'"\w+" \[', dot_content))
    edge_count = len(re.findall(r'"\w+" -> "\w+"', dot_content))
    log.info(f"Graph contains ~{node_count} nodes and ~{edge_count} edges")

    if skip_render:
        log.info("Skipping DOT -> PNG rendering (--skip-graph-render)")
        return dot_path, None

    png_path = f"{base_name}.png"
    try:
        result = subprocess.run(
            ["dot", "-Tpng", dot_path, "-o", png_path],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if result.returncode != 0:
            log.warning(f"Graphviz rendering failed: {result.stderr}")
            return dot_path, None
        log.info(f"Graph rendered to {png_path}")
        return dot_path, png_path
    except FileNotFoundError:
        log.warning("Graphviz 'dot' not found in PATH; skipping PNG render")
        return dot_path, None
    except subprocess.TimeoutExpired:
        log.warning("Graphviz rendering timed out")
        return dot_path, None


# ---------------------------------------------------------------------------
# Step 3: Session + file download
# ---------------------------------------------------------------------------


def pick_destination(peers, open_channels, explicit_dest, min_peer_score):
    """Pick the destination peer: explicit if given, otherwise highest-scoring peer with an open channel."""
    if explicit_dest:
        return explicit_dest

    channel_peers = {ch["peerAddress"] for ch in open_channels}
    candidates = [p for p in peers if p["address"] in channel_peers and p.get("score", 0) >= min_peer_score]
    if not candidates:
        raise RuntimeError(
            f"No peers with score >= {min_peer_score} and an open channel. Channel peers: {channel_peers}"
        )

    best = max(candidates, key=lambda p: p.get("score", 0))
    return best["address"]


def create_session(base_url, token, destination, hops, target):
    body = {
        "capabilities": ["Segmentation"],
        "destination": destination,
        "listenHost": "127.0.0.1:0",
        "forwardPath": {"Hops": hops},
        "returnPath": {"Hops": hops},
        "target": {"Plain": target},
        "responseBuffer": "4MiB",
    }
    return api_request(base_url, "POST", "/session/tcp", body=body, token=token)


def close_session(base_url, token, session):
    protocol = session["protocol"]
    ip = session["ip"]
    port = session["port"]
    try:
        api_request(base_url, "DELETE", f"/session/{protocol}/{ip}/{port}", token=token)
    except Exception as e:
        log.warning(f"Failed to close session: {e}")


def download_through_session(session, url, output_path, timeout):
    """Download a file by sending an HTTP request through the HOPR session socket."""
    parsed = urlparse(url)
    host = parsed.hostname
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    path = parsed.path
    if parsed.query:
        path += f"?{parsed.query}"

    # For HTTPS URLs, we need to use the target as host:443 and handle TLS.
    # The HOPR session acts as a TCP tunnel, so for HTTPS we'd need a CONNECT proxy.
    # For simplicity, we do a plain HTTP request. If the URL is HTTPS, we note
    # that the exit node needs to handle TLS to the target.
    use_tls = parsed.scheme == "https"

    http_request = (f"GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nAccept: */*\r\n\r\n").encode()

    session_ip = session["ip"]
    session_port = session["port"]
    log.info(f"Connecting to session at {session_ip}:{session_port}")
    log.info(f"Downloading: {url}")

    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(timeout)
    try:
        sock.connect((session_ip, session_port))

        if use_tls:
            import ssl

            context = ssl.create_default_context()
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            sock = context.wrap_socket(sock, server_hostname=host)

        sock.sendall(http_request)

        response = b""
        while True:
            try:
                chunk = sock.recv(65536)
                if not chunk:
                    break
                response += chunk
            except socket.timeout:
                log.warning("Socket timeout while receiving data")
                break
    finally:
        sock.close()

    log.info(f"Received {len(response)} bytes total")

    # Split headers from body
    header_end = response.find(b"\r\n\r\n")
    if header_end == -1:
        raise RuntimeError(f"No HTTP headers found in response ({len(response)} bytes)")

    headers_raw = response[:header_end].decode("utf-8", errors="replace")
    body = response[header_end + 4 :]

    # Check for chunked transfer encoding
    if b"Transfer-Encoding: chunked" in response[:header_end]:
        body = decode_chunked(body)

    status_line = headers_raw.split("\r\n")[0]
    log.info(f"Response status: {status_line}")
    log.info(f"Response headers:\n{headers_raw}")

    if "200" not in status_line:
        raise RuntimeError(f"Expected HTTP 200, got: {status_line}")

    # Write body to file
    # Determine file extension from content-type or URL
    ext = Path(parsed.path).suffix or ".bin"
    if not ext or ext == ".bin":
        for header_line in headers_raw.split("\r\n"):
            if header_line.lower().startswith("content-type:"):
                ct = header_line.split(":", 1)[1].strip()
                ext_map = {
                    "image/png": ".png",
                    "image/jpeg": ".jpg",
                    "application/pdf": ".pdf",
                    "text/html": ".html",
                    "application/octet-stream": ".bin",
                }
                ext = ext_map.get(ct.split(";")[0].strip(), ".bin")
                break

    # If the URL has a format parameter, use that
    if "format=" in (parsed.query or ""):
        for param in parsed.query.split("&"):
            if param.startswith("format="):
                ext = f".{param.split('=')[1]}"
                break

    if not output_path.endswith(ext):
        output_path = f"{output_path}{ext}"

    with open(output_path, "wb") as f:
        f.write(body)

    log.info(f"Downloaded file saved to {output_path} ({len(body)} bytes)")
    return output_path, len(body)


def decode_chunked(data):
    """Decode HTTP chunked transfer encoding."""
    decoded = b""
    pos = 0
    while pos < len(data):
        # Find chunk size line
        line_end = data.find(b"\r\n", pos)
        if line_end == -1:
            break
        size_str = data[pos:line_end].decode("ascii", errors="replace").strip()
        if not size_str:
            pos = line_end + 2
            continue
        # Chunk size may include extensions after a semicolon
        size_str = size_str.split(";")[0].strip()
        try:
            chunk_size = int(size_str, 16)
        except ValueError:
            break
        if chunk_size == 0:
            break
        chunk_start = line_end + 2
        chunk_end = chunk_start + chunk_size
        decoded += data[chunk_start:chunk_end]
        pos = chunk_end + 2  # skip trailing \r\n
    return decoded


# ---------------------------------------------------------------------------
# Step 4: Log parsing
# ---------------------------------------------------------------------------


def parse_log(log_path):
    """Parse the hoprd log file and extract session-related events."""
    if not os.path.exists(log_path):
        log.warning(f"Log file not found: {log_path}")
        return

    log.info(f"Parsing log file: {log_path}")

    with open(log_path, "r", errors="replace") as f:
        content = f.read()

    lines = content.splitlines()
    log.info(f"Log file: {len(lines)} lines total")

    # Patterns of interest — tuned to avoid matching module paths in tracing spans
    patterns = {
        "session_open": re.compile(r"(?i)session.*open|open.*session|new.*session"),
        "session_close": re.compile(r"(?i)session.*clos|clos.*session|session.*drop"),
        "channel": re.compile(r"(?i)channel.*(open|clos|fund|balanc)|on-chain.*channel"),
        "error": re.compile(r"\bERROR\b"),
        "warning": re.compile(r"\bWARN\b"),
        "peer_connected": re.compile(r"(?i)peer.*(connect|disconnect)|connection.*(establish|lost)"),
        "path_resolution": re.compile(r"(?i)cannot find.*hop path|path error|routing.*fail|failed to resolve"),
        "probe": re.compile(r"(?i)probe successful|probe failed|pong for unknown"),
    }

    categorized = {k: [] for k in patterns}

    for i, line in enumerate(lines, 1):
        for category, pattern in patterns.items():
            if pattern.search(line):
                categorized[category].append((i, line.strip()))

    # Report
    log.info("=== Log Analysis ===")
    for category, matches in categorized.items():
        if matches:
            log.info(f"\n--- {category} ({len(matches)} occurrences) ---")
            # Show first and last few
            display = matches[:5]
            if len(matches) > 10:
                display += [("...", f"... ({len(matches) - 10} more) ...")]
                display += matches[-5:]
            elif len(matches) > 5:
                display += matches[5:]
            for lineno, text in display:
                log.info(f"  L{lineno}: {text[:200]}")

    # Summary counts
    log.info(
        f"\nSession lifecycle: {len(categorized['session_open'])} opens, {len(categorized['session_close'])} closes"
    )

    probe_count = len(categorized.get("probe", []))
    if probe_count:
        log.info(f"Probes: {probe_count} events logged")

    error_count = len(categorized["error"])
    if error_count:
        log.warning(f"Errors: {error_count} error(s) in log — review above for details")

    return categorized


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    args = parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    api = args.api
    token = args.token

    # --- Step 0: Wait for node ---
    log.info(f"Waiting for node at {api}...")
    wait_for_ready(api, token, args.ready_timeout)

    info = get_node_info(api, token)
    log.info(f"Node info: network={info.get('hoprNetworkName')}, connectivity={info.get('connectivityStatus')}")

    my_address = get_address(api, token)
    log.info(f"Client address: {my_address}")

    if args.address_file:
        with open(args.address_file, "w") as f:
            f.write(my_address.lower())

    # --- Step 1: Ensure channels to best-scoring peers ---
    log.info("\n=== Step 1: Ensure Channels to Best Peers ===")
    open_channels = ensure_channels_to_best_peers(api, token, args.min_peer_score, args.min_channel_balance)

    # --- Step 2: Download and render network graph ---
    log.info("\n=== Step 2: Network Graph ===")
    dot_path, png_path = fetch_and_render_graph(api, token, args.graph_output, args.skip_graph_render)

    # --- Step 3: Discover peers and open session ---
    log.info("\n=== Step 3: Open Session and Download File ===")
    peers = get_peers(api, token)
    log.info(f"Connected peers: {len(peers)}")
    for p in sorted(peers, key=lambda x: x.get("quality", x.get("score", 0)), reverse=True):
        score = p.get("score", 0)
        log.info(f"  {p['address'][:16]}... score={score:.2f}")

    destination = pick_destination(peers, open_channels, args.destination, args.min_peer_score)
    log.info(f"Destination: {destination}")

    # Determine target host:port from the download URL
    parsed_url = urlparse(args.url)
    target_host = parsed_url.hostname
    target_port = parsed_url.port or (443 if parsed_url.scheme == "https" else 80)
    target = f"{target_host}:{target_port}"

    output_path = None
    file_size = 0
    download_error = None

    log.info(f"Creating {args.hops}-hop TCP session -> {target}")
    session = create_session(api, token, destination, args.hops, target)
    if session is None:
        log.warning("Failed to create session — skipping download")
        download_error = "session creation failed"
    else:
        log.info(f"Session endpoint: {session['ip']}:{session['port']}")
        try:
            output_path, file_size = download_through_session(session, args.url, args.output, args.timeout)
            log.info(f"File downloaded successfully: {output_path} ({file_size} bytes)")
        except Exception as e:
            download_error = str(e)
            log.warning(f"Download failed: {e}")
        finally:
            close_session(api, token, session)
            log.info("Session closed")

    # --- Step 4: Parse log ---
    log.info(f"\n=== Step 4: Log Analysis ===")
    parse_log(args.log)

    # --- Summary ---
    log.info("\n" + "=" * 60)
    log.info("DEMO COMPLETE")
    log.info(f"  Channels: {len(open_channels)} open outgoing")
    log.info(f"  Graph: {dot_path}" + (f" + {png_path}" if png_path else ""))
    log.info(f"  Session: {args.hops}-hop to {destination[:16]}...")
    if output_path:
        log.info(f"  Downloaded: {output_path} ({file_size} bytes)")
    elif download_error:
        log.warning(f"  Download: FAILED ({download_error})")
    log.info("=" * 60)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        log.error(f"FAILED: {e}", exc_info=True)
        sys.exit(1)
