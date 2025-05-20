#!/usr/bin/env python
import argparse
import random
import socket
import sys


def find_available_port_block(min_port, max_port, skip, block_size=None):
    """
    Find a randomly selected available port on localhost within the specified range,
    checking only every nth port based on the skip parameter, and ensuring that
    a contiguous block of ports following the found port are also available.

    Args:
        min_port (int): The minimum port number to check (inclusive)
        max_port (int): The maximum port number to check (exclusive)
        skip (int): Check only every nth port (e.g. skip=2 checks every second port)
        block_size (int, optional): Number of consecutive ports that must be free.
                                   If None, defaults to the same value as skip.

    Returns:
        int: The starting port number of an available block, or None if no suitable block found
    """
    # Validate port range
    if not (0 <= min_port < max_port <= 65535):
        raise ValueError("Invalid port range. Ensure 0 <= min_port < max_port <= 65535")
    # Ensure skip is at least 0
    # Ensure skip is at least 1
    skip = max(1, skip)

    # If block_size is not specified, use the same value as skip
    if block_size is None:
        block_size = skip

    # Adjust max_port to ensure we can fit a block of ports at the end
    adjusted_max = max_port - block_size

    # Create a list of potential starting ports in the specified range, applying the skip
    potential_starts = list(range(min_port, adjusted_max + 1, skip))

    # Randomize the port order
    random.shuffle(potential_starts)

    # Variable to store our result
    result = None

    for start_port in potential_starts:
        # Check if all ports in the block are available
        block_available = True

        # Check each port in the block
        for offset in range(block_size):
            port = start_port + offset

            # Skip checking if port is beyond the max_port
            if port > max_port:
                block_available = False
                break

            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                try:
                    # connect_ex returns 0 if the connection succeeds,
                    # and a non-zero error code otherwise
                    if s.connect_ex(("127.0.0.1", port)) == 0:
                        # Port is in use
                        block_available = False
                        break
                except socket.error:
                    # If we get a socket error, assume the port is unusable
                    block_available = False
                    break

        # If all ports in the block are available, set the result
        if block_available:
            result = start_port
            break  # Exit the loop once we find a valid block

    # Return the starting port of the available block, or None if not found
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Find an available port block.")
    parser.add_argument("--min-port", type=int, default=8000, help="Minimum port number")
    parser.add_argument("--max-port", type=int, default=9000, help="Maximum port number")
    parser.add_argument("--skip", type=int, default=20, help="Port skip interval")
    parser.add_argument("--block-size", type=int, help="Size of port block (defaults to skip value if not specified)")

    args = parser.parse_args()

    result = find_available_port_block(
        min_port=args.min_port, max_port=args.max_port, skip=args.skip, block_size=args.block_size
    )

    if result is None:
        sys.stderr.write("No available port block found\n")
        sys.exit(1)
    else:
        sys.stdout.write(f"{result}\n")
