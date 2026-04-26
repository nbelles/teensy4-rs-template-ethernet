#!/usr/bin/env python3
"""Simple UDP client to interact with the Teensy 4.1 echo socket."""

import socket
import sys

TEENSY_IP = "192.168.8.177"
TEENSY_PORT = 5000
TIMEOUT = 2.0


def main():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(TIMEOUT)

    print(f"Teensy UDP client — target {TEENSY_IP}:{TEENSY_PORT}")
    print("Type a message and press Enter (Ctrl-C to quit)\n")

    try:
        while True:
            msg = input("> ")
            if not msg:
                continue
            sock.sendto(msg.encode(), (TEENSY_IP, TEENSY_PORT))
            try:
                data, addr = sock.recvfrom(1024)
                print(
                    f"  echo from {addr[0]}:{addr[1]}: {data.decode(errors='replace')}"
                )
            except socket.timeout:
                print("  (no response)")
    except (KeyboardInterrupt, EOFError):
        print("\nBye!")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
