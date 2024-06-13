import argparse
import json
import os
import socket
import re
import sys


def check_socket(address: str, port: int, timeout: float):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(timeout)
    try:
        s.connect((address, int(port)))
        return True
    except Exception:
        return False
    finally:
        s.close()


def validate_file(f):
    if not os.path.exists(f):
        # Argparse uses the ArgumentTypeError to give a rejection message like:
        # error: argument input: x does not exist
        raise argparse.ArgumentTypeError("{0} does not exist".format(f))
    return f


def main(records, timeout: float):
    for count, record in enumerate(records):
        if record["multiaddr"] is not None and record["multiaddr"] != "":
            m = re.match("^/.*/(.*)/tcp/(.*)$", record["multiaddr"])
            if m:
                record["status"] = "connected" if check_socket(m.group(1), m.group(2), timeout) else "unavailable"
        else:
            record["status"] = "skipped"

        updt(len(records), count + 1)

    return records


def updt(total, progress):
    """
    Displays or updates a console progress bar.

    Original source: https://stackoverflow.com/a/15860757/1391441
    """
    barLength, status = 20, ""
    progress = float(progress) / float(total)
    if progress >= 1.0:
        progress, status = 1, "\r\n"
    block = int(round(barLength * progress))
    text = "\r[{}] {:.0f}% {}".format("#" * block + "-" * (barLength - block), round(progress * 100, 0), status)
    sys.stdout.write(text)
    sys.stdout.flush()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="A JSON file containing a list of dicts that must have 'multiaddr' keyword"
    )
    parser.add_argument(
        "-i", "--input", dest="filename", required=True, type=validate_file, help="input file", metavar="FILE"
    )
    parser.add_argument("-o", "--output", dest="fileout", required=True, help="output file", metavar="FILE")
    parser.add_argument(
        "-t",
        "--timeout",
        dest="timeout",
        default=3.0,
        help="seconds before declaring a timeout on a connection",
        metavar="SECONDS",
    )
    args = parser.parse_args()

    with open(args.filename) as f:
        data = json.load(f)

    records = main(data, args.timeout)

    with open(args.fileout, "w") as f:
        f.write(json.dumps(records))
