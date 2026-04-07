"""Generate animated graph visualizations of HOPR path usage from node logs.

Inputs:
    log_file  - Path to a log file from a hoprd client instance
    dot_file  - Path to a DOT graph file (from GET /network/graph)

Outputs:
    <out-dir>/graph.html      - Interactive HTML with packet animation + heatmap
    <out-dir>/graph.gif       - Animated GIF of the packet flow

Renders with D3.js and captures via Playwright.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
import webbrowser
from collections import Counter
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from importlib import resources
from typing import Optional

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")

from PIL import Image


# ── Data types ───────────────────────────────────────────────────────────────


@dataclass
class PathEvent:
    timestamp: datetime
    direction: str  # "forward" or "return"
    nodes: list[str]
    surb: bool = False  # True for extra return paths (SURBs)


@dataclass
class GraphState:
    nodes: set[str] = field(default_factory=set)
    edges: dict[tuple[str, str], str] = field(default_factory=dict)


# ── DOT parsing ──────────────────────────────────────────────────────────────

EDGE_RE = re.compile(r'"([^"]+)"\s*->\s*"([^"]+)"\s*' r'(?:\[label="([^"]*)"\])?')


def parse_dot(dot_path: str) -> GraphState:
    gs = GraphState()
    with open(dot_path) as f:
        for line in f:
            m = EDGE_RE.search(line)
            if m:
                src, dst, label = m.group(1), m.group(2), m.group(3) or ""
                gs.nodes.add(src)
                gs.nodes.add(dst)
                gs.edges[(src, dst)] = label
    return gs


# ── Log parsing ──────────────────────────────────────────────────────────────

TS_PATTERNS = [
    re.compile(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)"),
    re.compile(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(?:\.\d+)?)"),
]

RESOLVED_PATH_RE = re.compile(
    r"direction=\"?(forward|return)\"?\s+"
    r"destination=(\S+)\s*"
    r"(?:index=(\d+)\s*)?"
    r".*?path=validated path \[([^\]]+)\]"
)
LOOPBACK_PATH_RE = re.compile(
    r"direction=\"?loopback\"?\s+"
    r"source=(\S+)\s+"
    r"destination=(\S+)\s+"
    r"explicit_path=\[([^\]]*)\]"
)
CANDIDATE_PATH_RE = re.compile(r"\[(forward|return)\] candidate path.*?" r"path=\[([^\]]+)\]")


def parse_timestamp(line: str) -> Optional[datetime]:
    for pat in TS_PATTERNS:
        m = pat.search(line)
        if m:
            ts_str = m.group(1).replace("Z", "+00:00")
            for fmt in ("%Y-%m-%dT%H:%M:%S.%f%z", "%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%d %H:%M:%S.%f", "%Y-%m-%d %H:%M:%S"):
                try:
                    return datetime.strptime(ts_str, fmt)
                except ValueError:
                    continue
    return None


def parse_path_nodes(raw: str) -> list[str]:
    return [p.strip() for p in raw.split(",") if p.strip()]


def _parse_json_line(line: str) -> tuple[Optional[datetime], Optional[str], Optional[str]]:
    try:
        obj = json.loads(line)
    except (json.JSONDecodeError, KeyError):
        return None, None, None
    ts_str = obj.get("timestamp") or obj.get("time") or obj.get("ts")
    ts = parse_timestamp(ts_str) if isinstance(ts_str, str) else None
    fields = obj.get("fields", {})
    if isinstance(fields, dict):
        msg, path_val = fields.get("message", ""), fields.get("path", "")
    else:
        msg, path_val = obj.get("message", obj.get("msg", "")), obj.get("path", "")
    m = re.search(r"\[(forward|return)\] resolved", msg)
    if not m:
        return ts, None, None
    direction = m.group(1)
    pm = re.search(r"validated path \[([^\]]+)\]", str(path_val))
    return ts, direction, pm.group(1) if pm else None


def parse_logs(log_path: str, me: Optional[str] = None) -> list[PathEvent]:
    events: list[PathEvent] = []
    # Track offchain destination key → onchain address from forward paths
    dest_key_to_onchain: dict[str, str] = {}

    with open(log_path, errors="replace") as f:
        for line in f:
            line = ANSI_RE.sub("", line.strip())
            if not line:
                continue
            ts, direction, nodes_raw, dest_key, surb_index = None, None, None, None, None
            if line.startswith("{"):
                ts, direction, nodes_raw = _parse_json_line(line)
            if direction is None:
                ts = ts or parse_timestamp(line)
                m = RESOLVED_PATH_RE.search(line)
                if m:
                    direction, dest_key = m.group(1), m.group(2)
                    surb_index = int(m.group(3)) if m.group(3) else None
                    nodes_raw = m.group(4)
                else:
                    m = CANDIDATE_PATH_RE.search(line)
                    if m:
                        direction, nodes_raw = m.group(1), m.group(2)
            if direction and nodes_raw:
                nodes = parse_path_nodes(nodes_raw)
                if not nodes:
                    continue

                is_surb = direction == "return" and surb_index is not None and surb_index > 0

                if me:
                    if direction == "forward":
                        # Forward: me -> intermediate... -> dest
                        nodes = [me] + nodes
                        # Learn the offchain→onchain mapping for the destination
                        if dest_key:
                            dest_key_to_onchain[dest_key] = nodes[-1]
                    elif direction == "return":
                        # Return: dest -> intermediate... -> me
                        # The path already ends at us; prepend the remote peer
                        if dest_key and dest_key in dest_key_to_onchain:
                            nodes = [dest_key_to_onchain[dest_key]] + nodes
                        else:
                            continue  # Can't resolve return source, skip

                if len(nodes) >= 2:
                    events.append(
                        PathEvent(
                            timestamp=ts or datetime.min,
                            direction=direction,
                            nodes=nodes,
                            surb=is_surb,
                        )
                    )
    return events


# ── Key matching ─────────────────────────────────────────────────────────────


def build_key_index(graph: GraphState) -> dict[str, str]:
    index: dict[str, str] = {}
    for node in graph.nodes:
        index[node] = node
        for n in (8, 12, 16, 20, 40):
            if len(node) > n:
                index[node[-n:]] = node
                index[node[:n]] = node
    return index


def resolve_node(key: str, index: dict[str, str]) -> Optional[str]:
    key = key.strip()
    if key in index:
        return index[key]
    for n in (8, 12, 16, 20, 40):
        for sub in (key[-n:] if len(key) > n else key, key[:n] if len(key) > n else key):
            if sub in index:
                return index[sub]
    return None


def path_to_edge_ids(nodes: list[str], index: dict[str, str], graph: GraphState) -> tuple[list[str], list[str]]:
    """Convert a path's nodes to edge IDs and resolved node sequence.

    Returns (edge_ids, resolved_nodes). Edge IDs may be flipped to match
    graph direction; resolved_nodes preserves the original traversal order.
    Edges and nodes not yet in the graph are added automatically so that
    all log-observed paths can be visualized.
    """
    resolved = [r for n in nodes if (r := resolve_node(n, index))]
    edge_ids = []
    for i in range(len(resolved) - 1):
        src, dst = resolved[i], resolved[i + 1]
        pair = (src, dst)
        if pair in graph.edges:
            edge_ids.append(f"{src}->{dst}")
        else:
            rev = (dst, src)
            if rev in graph.edges:
                edge_ids.append(f"{dst}->{src}")
            else:
                # Edge not in graph — add it from log evidence
                graph.nodes.add(src)
                graph.nodes.add(dst)
                graph.edges[pair] = "from-log"
                # Update key index for new nodes
                for node in (src, dst):
                    index[node] = node
                    for n in (8, 12, 16, 20, 40):
                        if len(node) > n:
                            index[node[-n:]] = node
                            index[node[:n]] = node
                edge_ids.append(f"{src}->{dst}")
    return edge_ids, resolved


# ── Build event data for template ────────────────────────────────────────────


def build_path_events_json(
    events: list[PathEvent],
    key_index: dict[str, str],
    graph: GraphState,
    me: Optional[str] = None,
) -> tuple[list[dict], Optional[str], Optional[str]]:
    """Convert parsed events to JSON-serializable dicts with edge IDs.

    Also determines the SRC node (log owner = source of forward paths)
    and DST node (session target = destination of forward paths).
    """
    result = []
    src_node = None
    dst_node = None
    first_ts: Optional[datetime] = None

    for event in events:
        edge_ids, resolved_nodes = path_to_edge_ids(event.nodes, key_index, graph)
        if not edge_ids:
            continue

        time_str = event.timestamp.strftime("%H:%M:%S.%f")[:-3] if event.timestamp != datetime.min else ""

        # Compute ms offset from first event for real-time scheduling
        ts_ms = 0
        if event.timestamp != datetime.min:
            if first_ts is None:
                first_ts = event.timestamp
            ts_ms = int((event.timestamp - first_ts).total_seconds() * 1000)

        # Detect multi-hop paths (loopback probes or multi-hop sessions)
        is_loopback = event.direction == "forward" and len(edge_ids) >= 2

        result.append(
            {
                "direction": event.direction,
                "edges": edge_ids,
                "nodes": resolved_nodes,
                "time": time_str,
                "ts_ms": ts_ms,
                "surb": event.surb,
                "loopback": is_loopback,
            }
        )

        # Determine SRC/DST from forward paths:
        # SRC = source of the first edge in a forward path (the log owner's first relay)
        # DST = target of the last edge in a forward path
        if event.direction == "forward" and edge_ids:
            first_edge = edge_ids[0].split("->")
            last_edge = edge_ids[-1].split("->")
            if src_node is None:
                src_node = first_edge[0]
            if dst_node is None:
                dst_node = last_edge[1]

    # Compute path distribution stats
    fwd_counts: Counter[str] = Counter()
    ret_counts: Counter[str] = Counter()
    for evt in result:
        key = " → ".join(evt["nodes"])
        if evt["direction"] == "forward":
            fwd_counts[key] += 1
        else:
            ret_counts[key] += 1

    def dist_list(counts: Counter[str]) -> list[dict]:
        total = sum(counts.values()) or 1
        return [{"path": p, "count": c, "pct": round(c / total, 4)} for p, c in counts.most_common()]

    path_stats = {"forward": dist_list(fwd_counts), "return": dist_list(ret_counts)}

    return result, path_stats, src_node, dst_node


# ── HTML generation ──────────────────────────────────────────────────────────


def build_html(
    graph: GraphState,
    path_events: list[dict],
    path_stats: dict,
    src_node: Optional[str],
    dst_node: Optional[str],
    interactive: bool = False,
) -> str:
    template_path = resources.files("graph_timelapse").joinpath("template.html")
    template = template_path.read_text()

    graph_data = {
        "nodes": sorted(graph.nodes),
        "edges": [[s, t, label] for (s, t), label in graph.edges.items()],
    }

    html = template.replace("__GRAPH_DATA__", json.dumps(graph_data))
    html = html.replace("__PATH_EVENTS__", json.dumps(path_events))
    html = html.replace("__PATH_STATS__", json.dumps(path_stats))
    html = html.replace("__SRC_NODE__", json.dumps(src_node))
    html = html.replace("__DST_NODE__", json.dumps(dst_node))
    html = html.replace("__INTERACTIVE__", "true" if interactive else "false")
    return html


# ── Playwright rendering ─────────────────────────────────────────────────────


def render_events_to_pngs(html: str, num_events: int, tmpdir: str) -> list[str]:
    """Render each path event animation to a PNG snapshot."""
    from playwright.sync_api import sync_playwright

    html_path = os.path.join(tmpdir, "graph.html")
    with open(html_path, "w") as f:
        f.write(html)

    png_paths: list[str] = []
    with sync_playwright() as p:
        browser = p.chromium.launch()
        page = browser.new_page(viewport={"width": 800, "height": 600})
        page.goto(f"file://{html_path}")
        page.wait_for_function("typeof window.playEvent === 'function'")

        for i in range(num_events):
            # Play the event and wait for animation to complete
            page.evaluate(f"window.playEvent({i})")
            page.wait_for_timeout(1200)  # packet animation + settle
            png_path = os.path.join(tmpdir, f"frame_{i:04d}.png")
            page.locator("#graph").screenshot(path=png_path)
            png_paths.append(png_path)

        browser.close()

    return png_paths


def make_gif(png_paths: list[str], output_path: str, fps: int = 2) -> None:
    if not png_paths:
        print(f"  No frames to assemble for {output_path}", file=sys.stderr)
        return
    frames = []
    for p in png_paths:
        img = Image.open(p).convert("RGBA")
        frames.append(img.convert("P", palette=Image.ADAPTIVE))
    duration = 1000 // fps
    frames[0].save(output_path, save_all=True, append_images=frames[1:], duration=duration, loop=0)


# ── Main ─────────────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(description="Animate HOPR packet flow through the network graph")
    parser.add_argument("log_file", help="Path to hoprd log file")
    parser.add_argument("dot_file", help="Path to DOT graph file")
    parser.add_argument("--fps", type=int, default=2, help="GIF frames per second (default: 2)")
    parser.add_argument("--out-dir", default=".", help="Output directory (default: current directory)")
    parser.add_argument("--no-open", action="store_true", help="Don't auto-open browser")
    parser.add_argument("--no-gif", action="store_true", help="Skip GIF generation (HTML only)")
    parser.add_argument(
        "--me", default=None, help="Our node's onchain address (prepended/appended to paths for edge matching)"
    )
    args = parser.parse_args()

    print(f"Parsing graph: {args.dot_file}")
    graph = parse_dot(args.dot_file)
    print(f"  {len(graph.nodes)} nodes, {len(graph.edges)} edges")

    print(f"Parsing logs: {args.log_file}")
    events = parse_logs(args.log_file, me=args.me)
    fwd = sum(1 for e in events if e.direction == "forward")
    ret = sum(1 for e in events if e.direction == "return")
    print(f"  {len(events)} path events ({fwd} forward, {ret} return)")

    if not events:
        print("No path events found. Ensure debug logging is enabled.", file=sys.stderr)
        sys.exit(1)
    if not graph.edges:
        print("No edges found in DOT file.", file=sys.stderr)
        sys.exit(1)

    key_index = build_key_index(graph)
    path_events, path_stats, src_node, dst_node = build_path_events_json(events, key_index, graph, me=args.me)
    print(f"  {len(path_events)} matched events")
    if src_node:
        short_src = src_node[:10] + "..." if len(src_node) > 13 else src_node
        print(f"  SRC (you): {short_src}")
    if dst_node:
        short_dst = dst_node[:10] + "..." if len(dst_node) > 13 else dst_node
        print(f"  DST: {short_dst}")

    if not path_events:
        print("No events matched graph edges.", file=sys.stderr)
        sys.exit(1)

    os.makedirs(args.out_dir, exist_ok=True)

    # Save interactive HTML
    html_out = os.path.join(args.out_dir, "graph.html")
    with open(html_out, "w") as f:
        f.write(build_html(graph, path_events, path_stats, src_node, dst_node, interactive=True))
    print(f"Saved: {html_out}")

    # Auto-open browser
    if not args.no_open:
        abs_path = os.path.abspath(html_out)
        print(f"Opening browser...")
        webbrowser.open(f"file://{abs_path}")

    # Generate GIF
    if not args.no_gif:
        print("Ensuring Playwright Chromium is available...")
        subprocess.run([sys.executable, "-m", "playwright", "install", "chromium"], capture_output=True)

        # Limit GIF to first N events to keep size reasonable
        max_gif_events = min(len(path_events), 30)

        with tempfile.TemporaryDirectory(prefix="hopr-graph-") as tmpdir:
            headless_html = build_html(
                graph, path_events[:max_gif_events], path_stats, src_node, dst_node, interactive=False
            )
            print(f"Rendering {max_gif_events} events to GIF...")
            pngs = render_events_to_pngs(headless_html, max_gif_events, tmpdir)

            gif_out = os.path.join(args.out_dir, "graph.gif")
            print(f"Assembling {len(pngs)} frames -> {gif_out}")
            make_gif(pngs, gif_out, args.fps)
            print(f"Saved: {gif_out}")

    print("Done!")
