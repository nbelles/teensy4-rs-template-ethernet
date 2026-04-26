# Ethernet UDP Template — Teensy 4.1

Standalone template for the Teensy 4.1's onboard Ethernet using the DP83825I PHY. Runs a UDP echo server with a static IP using smoltcp.

## Hardware

- Teensy 4.1 with Ethernet kit (MagJack soldered to the bottom pads)
- Ethernet cable connected to your network/switch

## What It Does

1. Initializes the ENET peripheral, PLL, and RMII pin muxing
2. Resets the DP83825I PHY and configures it via MDIO
3. Binds a UDP socket to `192.168.1.177:5000`
4. Echoes any received UDP packet back to the sender
5. Logs traffic over USB serial

## Configuration

Edit `src/ethernet.rs` to change:
- `MAC` — MAC address (default: `02:00:00:00:00:01`)

Edit `src/main.rs` to change:
- IP address (default: `192.168.1.177/24`)
- UDP port (default: `5000`)

## Build & Flash

```bash
cargo objcopy --release -- -O ihex target/ethernet-template.hex
teensy_loader_cli --mcu=TEENSY41 -w target/ethernet-template.hex -v
```

## Testing

Send a UDP packet from your computer:

```bash
echo "hello" | nc -u 192.168.1.177 5000
```

### Interactive Client

`scripts/udp_client_input.py` opens an interactive prompt that sends each line you type as a UDP packet and prints the echoed response.

```bash
python3 scripts/udp_client_input.py
```

Type a message and press Enter. The Teensy echoes it back. Press Ctrl-C to quit.

### Throughput Test

`scripts/udp_client_profile.py` continuously sends 1024-byte random payloads and measures round-trip frequency, printing stats every second. 

```bash
python3 scripts/udp_client_profile.py
```

Press Ctrl-C to stop and see a final summary.

During testing, I was able to get an average of about 1900 rt/s (`1900 rount-trips/second * 2 trips/round-trip * 1024 bytes/trip * 8 bits/byte = 31,129,600 bytes/second = 31.129kbps`). I'm sure there's a lot more optimization to be done here, this was just a quick example to get it working.

## USB Serial Monitor

```bash
screen /dev/cu.usbmodem*
```

## Dependencies

- `teensy4-bsp` — Board support package
- `imxrt-enet` — ENET MAC driver for i.MX RT
- `smoltcp` — TCP/IP stack (UDP socket)
- `imxrt-log` — USB serial logging
- `rtic` v2 — Real-time task framework
