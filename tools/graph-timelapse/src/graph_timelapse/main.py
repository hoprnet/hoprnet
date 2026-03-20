"""Generate timelapse GIFs of path activation and edge heatmaps from HOPR node logs.

Inputs:
    log_file  - Path to a log file from a hoprd client instance
    dot_file  - Path to a DOT graph file (from GET /network/graph)

Outputs:
    <out-dir>/path-activation.gif  - Timelapse of activated paths per time window
    <out-dir>/edge-heatmap.gif     - Cumulative edge usage intensity over time

Requires graphviz (`dot` and `neato` CLIs) to be on PATH.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Optional

from PIL import Image


# ── Data types ───────────────────────────────────────────────────────────────


@dataclass
class PathEvent:
    timestamp: datetime
    direction: str  # "forward" or "return"
    nodes: list[str]  # chain address hex or offchain key hex


@dataclass
class GraphState:
    nodes: set[str] = field(default_factory=set)
    edges: dict[tuple[str, str], str] = field(default_factory=dict)  # (src,dst) -> label


# ── DOT parsing ──────────────────────────────────────────────────────────────

EDGE_RE = re.compile(r'"([^"]+)"\s*->\s*"([^"]+)"\s*' r'(?:\[label="([^"]*)"\])?')


def parse_dot(dot_path: str) -> GraphState:
    """Parse a DOT file into nodes and edges."""
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

RESOLVED_PATH_RE = re.compile(r"\[(forward|return)\] resolved (?:return )?path.*?" r"path=validated path \[([^\]]+)\]")

CANDIDATE_PATH_RE = re.compile(r"\[(forward|return)\] candidate path.*?" r"path=\[([^\]]+)\]")


def parse_timestamp(line: str) -> Optional[datetime]:
    for pat in TS_PATTERNS:
        m = pat.search(line)
        if m:
            ts_str = m.group(1).replace("Z", "+00:00")
            for fmt in (
                "%Y-%m-%dT%H:%M:%S.%f%z",
                "%Y-%m-%dT%H:%M:%S%z",
                "%Y-%m-%d %H:%M:%S.%f",
                "%Y-%m-%d %H:%M:%S",
            ):
                try:
                    return datetime.strptime(ts_str, fmt)
                except ValueError:
                    continue
    return None


def parse_path_nodes(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


def _parse_json_line(line: str) -> tuple[Optional[datetime], Optional[str], Optional[str]]:
    try:
        obj = json.loads(line)
    except (json.JSONDecodeError, KeyError):
        return None, None, None

    ts_str = obj.get("timestamp") or obj.get("time") or obj.get("ts")
    ts = parse_timestamp(ts_str) if isinstance(ts_str, str) else None

    fields = obj.get("fields", {})
    if isinstance(fields, dict):
        msg = fields.get("message", "")
        path_val = fields.get("path", "")
    else:
        msg = obj.get("message", obj.get("msg", ""))
        path_val = obj.get("path", "")

    m = re.search(r"\[(forward|return)\] resolved", msg)
    if not m:
        return ts, None, None

    direction = m.group(1)
    pm = re.search(r"validated path \[([^\]]+)\]", str(path_val))
    nodes_raw = pm.group(1) if pm else None
    return ts, direction, nodes_raw


def parse_logs(log_path: str) -> list[PathEvent]:
    events: list[PathEvent] = []
    with open(log_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue

            ts: Optional[datetime] = None
            direction: Optional[str] = None
            nodes_raw: Optional[str] = None

            if line.startswith("{"):
                ts, direction, nodes_raw = _parse_json_line(line)

            if direction is None:
                ts = ts or parse_timestamp(line)
                m = RESOLVED_PATH_RE.search(line)
                if m:
                    direction, nodes_raw = m.group(1), m.group(2)
                else:
                    m = CANDIDATE_PATH_RE.search(line)
                    if m:
                        direction, nodes_raw = m.group(1), m.group(2)

            if direction and nodes_raw:
                nodes = parse_path_nodes(nodes_raw)
                if len(nodes) >= 2:
                    events.append(PathEvent(timestamp=ts or datetime.min, direction=direction, nodes=nodes))

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
        suffix = key[-n:] if len(key) > n else key
        if suffix in index:
            return index[suffix]
        prefix = key[:n] if len(key) > n else key
        if prefix in index:
            return index[prefix]
    return None


def path_to_edges(nodes: list[str], index: dict[str, str], graph: GraphState) -> list[tuple[str, str]]:
    resolved = [r for n in nodes if (r := resolve_node(n, index))]
    edges = []
    for i in range(len(resolved) - 1):
        pair = (resolved[i], resolved[i + 1])
        if pair in graph.edges:
            edges.append(pair)
        else:
            rev = (resolved[i + 1], resolved[i])
            if rev in graph.edges:
                edges.append(rev)
    return edges


# ── Fixed layout ─────────────────────────────────────────────────────────────

# Regex for `dot -Tplain` node lines: node "name" x y width height ...
PLAIN_NODE_RE = re.compile(r'^node\s+"?([^"\s]+)"?\s+([\d.]+)\s+([\d.]+)')


def compute_fixed_layout(graph: GraphState) -> dict[str, tuple[float, float]]:
    """Run `dot` once on the base graph and extract node positions.

    Returns a mapping from node id to (x, y) in points.
    """
    # Build a minimal DOT for layout computation
    lines = ["digraph hopr {", "  node [shape=box];"]
    for node in sorted(graph.nodes):
        short = node[:10] + "..." if len(node) > 13 else node
        lines.append(f'  "{node}" [label="{short}"];')
    for src, dst in graph.edges:
        lines.append(f'  "{src}" -> "{dst}";')
    lines.append("}")
    dot_str = "\n".join(lines)

    result = subprocess.run(
        ["dot", "-Tplain"],
        input=dot_str.encode(),
        capture_output=True,
        timeout=30,
    )
    if result.returncode != 0:
        return {}

    positions: dict[str, tuple[float, float]] = {}
    for line in result.stdout.decode().splitlines():
        # plain format: node name x y width height label style shape color fillcolor
        parts = line.split()
        if len(parts) >= 4 and parts[0] == "node":
            name = parts[1].strip('"')
            x, y = float(parts[2]), float(parts[3])
            # Convert from inches to points (72 dpi)
            positions[name] = (x * 72.0, y * 72.0)

    return positions


# ── DOT rendering with highlights and fixed positions ────────────────────────

FORWARD_COLOR = "#2196F3"
RETURN_COLOR = "#FF9800"
BOTH_COLOR = "#9C27B0"


def intensity_color(ratio: float) -> str:
    if ratio <= 0.5:
        r = int(255 * (ratio * 2))
        g = 255
    else:
        r = 255
        g = int(255 * (1 - (ratio - 0.5) * 2))
    return f"#{r:02x}{g:02x}00"


SRC_COLOR = "#4CAF50"  # green fill for source nodes
DST_COLOR = "#F44336"  # red fill for destination nodes
RELAY_COLOR = "#BBDEFB"  # light blue for active relay nodes


def render_dot_highlighted(
    graph: GraphState,
    active_edges: dict[tuple[str, str], str],
    positions: dict[str, tuple[float, float]],
    edge_widths: Optional[dict[tuple[str, str], float]] = None,
    title: str = "",
    sources: Optional[set[str]] = None,
    destinations: Optional[set[str]] = None,
) -> str:
    """Render DOT with highlighted edges, pinned positions, and src/dst markers."""
    sources = sources or set()
    destinations = destinations or set()

    out = [
        "digraph hopr {",
        '  bgcolor="white";',
        '  node [style=filled, fillcolor="#E0E0E0", fontsize=10, shape=box, pin=true];',
        '  edge [fontsize=8, color="#CCCCCC"];',
    ]

    if title:
        out.append('  labelloc="t";')
        out.append(f'  label="{title}";')
        out.append("  fontsize=14;")

    active_nodes: set[str] = set()
    for s, d in active_edges:
        active_nodes.add(s)
        active_nodes.add(d)

    for node in sorted(graph.nodes):
        short = node[:10] + "..." if len(node) > 13 else node
        if node in sources and node in destinations:
            fill = SRC_COLOR
            short = f"S/D {short}"
        elif node in sources:
            fill = SRC_COLOR
            short = f"SRC {short}"
        elif node in destinations:
            fill = DST_COLOR
            short = f"DST {short}"
        elif node in active_nodes:
            fill = RELAY_COLOR
        else:
            fill = "#E0E0E0"
        pos = positions.get(node)
        pos_attr = f', pos="{pos[0]:.1f},{pos[1]:.1f}!"' if pos else ""
        out.append(f'  "{node}" [label="{short}", fillcolor="{fill}"{pos_attr}];')

    for (src, dst), label in graph.edges.items():
        color = active_edges.get((src, dst), "#CCCCCC")
        width = (edge_widths or {}).get((src, dst), 1.0)
        if color != "#CCCCCC":
            out.append(f'  "{src}" -> "{dst}" [color="{color}", penwidth={width:.1f}];')
        else:
            out.append(f'  "{src}" -> "{dst}" [color="{color}", penwidth=1.0, label="{label}"];')

    out.append("}")
    return "\n".join(out)


# ── Frame rendering ──────────────────────────────────────────────────────────


def dot_to_png(dot_str: str, png_path: str, use_neato: bool = False) -> bool:
    """Render a DOT string to PNG.

    When `use_neato` is True, uses `neato -n` which respects pre-set `pos`
    attributes, keeping node positions fixed across frames.
    """
    cmd = (
        ["neato", "-n", "-Tpng", "-Gdpi=150", "-o", png_path]
        if use_neato
        else ["dot", "-Tpng", "-Gdpi=150", "-o", png_path]
    )
    try:
        result = subprocess.run(cmd, input=dot_str.encode(), capture_output=True, timeout=30)
        return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


def make_gif(png_paths: list[str], output_path: str, fps: int = 4) -> None:
    if not png_paths:
        print(f"  No frames to assemble for {output_path}", file=sys.stderr)
        return

    frames = []
    for p in png_paths:
        img = Image.open(p).convert("RGBA")
        frames.append(img.convert("P", palette=Image.ADAPTIVE))

    duration = 1000 // fps
    frames[0].save(output_path, save_all=True, append_images=frames[1:], duration=duration, loop=0)


# ── Time windowing ───────────────────────────────────────────────────────────


def compute_time_windows(events: list[PathEvent], window_secs: int) -> list[tuple[datetime, datetime, list[PathEvent]]]:
    if not events:
        return []

    timed = [e for e in events if e.timestamp != datetime.min]
    if not timed:
        return [(datetime.min, datetime.min, [e]) for e in events]

    timed.sort(key=lambda e: e.timestamp)
    start = timed[0].timestamp
    end = timed[-1].timestamp
    delta = timedelta(seconds=window_secs)

    windows = []
    win_start = start
    while win_start <= end:
        win_end = win_start + delta
        bucket = [e for e in timed if win_start <= e.timestamp < win_end]
        if bucket:
            windows.append((win_start, win_end, bucket))
        win_start = win_end

    return windows


# ── Main ─────────────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(description="Generate timelapse GIFs of HOPR path activation and edge heatmaps")
    parser.add_argument("log_file", help="Path to hoprd log file")
    parser.add_argument("dot_file", help="Path to DOT graph file")
    parser.add_argument("--window", type=int, default=1, help="Time window in seconds per frame (default: 1)")
    parser.add_argument("--fps", type=int, default=4, help="Frames per second in output GIF (default: 4)")
    parser.add_argument("--out-dir", default=".", help="Output directory for GIFs (default: current directory)")
    args = parser.parse_args()

    for cmd in ("dot", "neato"):
        try:
            subprocess.run([cmd, "-V"], capture_output=True, check=True)
        except (FileNotFoundError, subprocess.CalledProcessError):
            print(
                f"Error: graphviz '{cmd}' not found. Available in the nix dev shell or: brew install graphviz",
                file=sys.stderr,
            )
            sys.exit(1)

    print(f"Parsing graph: {args.dot_file}")
    graph = parse_dot(args.dot_file)
    print(f"  {len(graph.nodes)} nodes, {len(graph.edges)} edges")

    print(f"Parsing logs: {args.log_file}")
    events = parse_logs(args.log_file)
    fwd = sum(1 for e in events if e.direction == "forward")
    ret = sum(1 for e in events if e.direction == "return")
    print(f"  {len(events)} path events ({fwd} forward, {ret} return)")

    if not events:
        print("No path events found in logs. Ensure debug logging is enabled.", file=sys.stderr)
        sys.exit(1)
    if not graph.edges:
        print("No edges found in DOT file.", file=sys.stderr)
        sys.exit(1)

    key_index = build_key_index(graph)
    matched = sum(1 for e in events if path_to_edges(e.nodes, key_index, graph))
    print(f"  {matched}/{len(events)} events matched to graph edges")

    # Compute fixed layout once
    print("Computing fixed graph layout...")
    positions = compute_fixed_layout(graph)
    print(f"  {len(positions)} node positions pinned")

    windows = compute_time_windows(events, args.window)
    print(f"  {len(windows)} time windows ({args.window}s each, {args.fps}fps = {args.fps}x speed)")

    os.makedirs(args.out_dir, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="hopr-graph-") as tmpdir:
        # ── GIF 1: Path activation timelapse ─────────────────────────
        print("Rendering path activation frames...")
        activation_frames: list[str] = []

        for i, (win_start, win_end, bucket) in enumerate(windows):
            active: dict[tuple[str, str], str] = {}
            srcs: set[str] = set()
            dsts: set[str] = set()
            for event in bucket:
                edges = path_to_edges(event.nodes, key_index, graph)
                color = FORWARD_COLOR if event.direction == "forward" else RETURN_COLOR
                for edge in edges:
                    if edge in active and active[edge] != color:
                        active[edge] = BOTH_COLOR
                    else:
                        active[edge] = color
                if edges:
                    if event.direction == "forward":
                        srcs.add(edges[0][0])
                        dsts.add(edges[-1][1])
                    else:
                        srcs.add(edges[-1][1])
                        dsts.add(edges[0][0])

            if win_start != datetime.min:
                title = f"Path activation: {win_start:%H:%M:%S} - {win_end:%H:%M:%S}"
            else:
                title = f"Path activation: event {i + 1}"

            dot_str = render_dot_highlighted(graph, active, positions, title=title, sources=srcs, destinations=dsts)
            png_path = os.path.join(tmpdir, f"activation_{i:04d}.png")
            if dot_to_png(dot_str, png_path, use_neato=True):
                activation_frames.append(png_path)

            if (i + 1) % 10 == 0:
                print(f"  {i + 1}/{len(windows)} frames", end="\r")

        activation_out = os.path.join(args.out_dir, "path-activation.gif")
        print(f"\nAssembling {len(activation_frames)} frames -> {activation_out}")
        make_gif(activation_frames, activation_out, args.fps)

        # ── GIF 2: Cumulative edge heatmap ───────────────────────────
        print("Rendering edge heatmap frames...")
        heatmap_frames: list[str] = []
        cumulative: Counter[tuple[str, str]] = Counter()
        all_srcs: set[str] = set()
        all_dsts: set[str] = set()

        for i, (win_start, win_end, bucket) in enumerate(windows):
            for event in bucket:
                edges = path_to_edges(event.nodes, key_index, graph)
                for edge in edges:
                    cumulative[edge] += 1
                if edges:
                    if event.direction == "forward":
                        all_srcs.add(edges[0][0])
                        all_dsts.add(edges[-1][1])
                    else:
                        all_srcs.add(edges[-1][1])
                        all_dsts.add(edges[0][0])

            if not cumulative:
                continue

            max_count = max(cumulative.values())
            active_colors: dict[tuple[str, str], str] = {}
            edge_widths: dict[tuple[str, str], float] = {}
            for edge, count in cumulative.items():
                ratio = count / max_count
                active_colors[edge] = intensity_color(ratio)
                edge_widths[edge] = 1.0 + ratio * 5.0

            total = sum(cumulative.values())
            if win_start != datetime.min:
                title = f"Edge heatmap (cumulative): {win_start:%H:%M:%S} - total {total} uses"
            else:
                title = f"Edge heatmap: through event {i + 1}"

            dot_str = render_dot_highlighted(
                graph, active_colors, positions, edge_widths, title=title, sources=all_srcs, destinations=all_dsts
            )
            png_path = os.path.join(tmpdir, f"heatmap_{i:04d}.png")
            if dot_to_png(dot_str, png_path, use_neato=True):
                heatmap_frames.append(png_path)

            if (i + 1) % 10 == 0:
                print(f"  {i + 1}/{len(windows)} frames", end="\r")

        heatmap_out = os.path.join(args.out_dir, "edge-heatmap.gif")
        print(f"\nAssembling {len(heatmap_frames)} frames -> {heatmap_out}")
        make_gif(heatmap_frames, heatmap_out, args.fps)

    print("Done!")
    print(f"  {activation_out}")
    print(f"  {heatmap_out}")
