#!/usr/bin/env python3
"""UDP round-trip throughput test against the Teensy 4.1 echo socket.

Sends 1024 random bytes, waits for the echo, then immediately sends the next
batch.  Prints round-trip frequency every second.
"""

import os
import socket
import time

TEENSY_IP = "192.168.1.177"
TEENSY_PORT = 5000
TIMEOUT = 2.0
PAYLOAD_SIZE = 1024


def main():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(TIMEOUT)

    print(f"UDP echo throughput test — {TEENSY_IP}:{TEENSY_PORT}")
    print(f"Payload: {PAYLOAD_SIZE} bytes   (Ctrl-C to stop)\n")

    trips = 0
    errors = 0
    t_start = time.monotonic()
    t_report = t_start

    try:
        while True:
            payload = os.urandom(PAYLOAD_SIZE)
            sock.sendto(payload, (TEENSY_IP, TEENSY_PORT))
            try:
                data, _ = sock.recvfrom(2048)
                if data == payload:
                    trips += 1
                else:
                    trips += 1
                    errors += 1
            except socket.timeout:
                errors += 1
                continue

            now = time.monotonic()
            if now - t_report >= 1.0:
                elapsed = now - t_start
                freq = trips / elapsed
                print(
                    f"  {trips} round-trips in {elapsed:.1f}s  "
                    f"— {freq:.1f} rt/s  "
                    f"({errors} errors)"
                )
                t_report = now
    except KeyboardInterrupt:
        elapsed = time.monotonic() - t_start
        freq = trips / elapsed if elapsed > 0 else 0
        print(
            f"\n{trips} round-trips in {elapsed:.1f}s — {freq:.1f} rt/s  ({errors} errors)"
        )
        print("Bye!")
    finally:
        sock.close()


if __name__ == "__main__":
    main()
