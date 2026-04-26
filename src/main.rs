//! Ethernet UDP echo template for the Teensy 4.1.
//!
//! Sets up the onboard DP83825I PHY, configures a static IP,
//! and runs a UDP echo server on port 5000 using smoltcp.

#![no_std]
#![no_main]

mod ethernet;

use teensy4_panic as _;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use board::t41 as my_board;
    use bsp::board;
    use imxrt_log as logging;
    use rtic_monotonics::systick::{Systick, *};
    use teensy4_bsp as bsp;

    use imxrt_enet::smoltcp;
    use smoltcp::iface::{Config, Interface, SocketSet, SocketStorage};
    use smoltcp::socket::udp;
    use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: board::Led,
        poller: logging::Poller,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            pins,
            usb,
            ..
        } = my_board(cx.device);

        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();
        let led = board::led(&mut gpio2, pins.p13);

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        udp_echo::spawn().unwrap();

        (Shared {}, Local { led, poller })
    }

    /// UDP echo server on 192.168.1.177:5000.
    #[task(local = [led])]
    async fn udp_echo(cx: udp_echo::Context) {
        unsafe { crate::ethernet::init_hardware() };
        Systick::delay(25.millis()).await;

        let txdt = cortex_m::singleton!(
            : imxrt_enet::TransmitBuffers<2, 1520> = imxrt_enet::TransmitBuffers::new()
        )
        .unwrap();
        let rxdt = cortex_m::singleton!(
            : imxrt_enet::ReceiveBuffers<2, 1520> = imxrt_enet::ReceiveBuffers::new()
        )
        .unwrap();
        let tx = txdt.take();
        let rx = rxdt.take();

        let enet_inst = unsafe { imxrt_ral_v6::enet::ENET1::instance() };
        let mut enet = imxrt_enet::Enet::new(
            enet_inst,
            tx,
            rx,
            crate::ethernet::IPG_FREQ,
            &crate::ethernet::MAC,
        );

        crate::ethernet::init_phy(&mut enet);

        enet.enable_rmii_mode(true);
        enet.set_duplex(imxrt_enet::Duplex::Full);
        enet.enable_mac(true);

        let config = Config::new(EthernetAddress(crate::ethernet::MAC).into());
        let mut iface = Interface::new(config, &mut enet, smoltcp::time::Instant::from_millis(0));
        iface.update_ip_addrs(|addrs| {
            addrs
                .push(IpCidr::new(IpAddress::v4(192, 168, 1, 177), 24))
                .unwrap();
        });

        let sock_store = cortex_m::singleton!(
            : [SocketStorage<'static>; 1] = [SocketStorage::EMPTY]
        )
        .unwrap();
        let udp_rx_meta = cortex_m::singleton!(
            : [udp::PacketMetadata; 4] = [udp::PacketMetadata::EMPTY; 4]
        )
        .unwrap();
        let udp_rx_data = cortex_m::singleton!(: [u8; 1024] = [0u8; 1024]).unwrap();
        let udp_tx_meta = cortex_m::singleton!(
            : [udp::PacketMetadata; 4] = [udp::PacketMetadata::EMPTY; 4]
        )
        .unwrap();
        let udp_tx_data = cortex_m::singleton!(: [u8; 1024] = [0u8; 1024]).unwrap();

        let mut sockets = SocketSet::new(&mut sock_store[..]);
        let rx_buf = udp::PacketBuffer::new(&mut udp_rx_meta[..], &mut udp_rx_data[..]);
        let tx_buf = udp::PacketBuffer::new(&mut udp_tx_meta[..], &mut udp_tx_data[..]);
        let udp_handle = sockets.add(udp::Socket::new(rx_buf, tx_buf));
        sockets
            .get_mut::<udp::Socket>(udp_handle)
            .bind(5000)
            .unwrap();

        log::info!("Ethernet up — 192.168.1.177:5000");

        let mut micros: i64 = 0;
        loop {
            cx.local.led.toggle();

            let time = smoltcp::time::Instant::from_micros(micros);
            iface.poll(time, &mut enet, &mut sockets);

            let socket = sockets.get_mut::<udp::Socket>(udp_handle);
            let mut buf = [0u8; 1024];
            while socket.can_recv() {
                if let Ok((n, sender)) = socket.recv_slice(&mut buf) {
                    // log::info!("UDP from {}: {} bytes", sender, n);
                    socket.send_slice(&buf[..n], sender).ok();
                }
            }

            Systick::delay(10.micros()).await;
            micros += 10;
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
