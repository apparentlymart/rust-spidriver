use embedded_hal::serial;
use serial_embedded_hal::{PortSettings, Serial};
use spidriver::SPIDriver;

fn main() {
    // This example demonstrates using the SPIDriver to configure an SSD1322
    // display driver that is driving a 256x64 pixel display and then rendering
    // a checkerboard pattern on it, assuming it's running in 4-bit grayscale mode.
    //
    // As well as the SPI signals, this example assumes:
    //    SPIDriver Port A is connected to the D/C signal on the driver.
    //    SPIDriver Port B is connected to the reset signal on the driver.
    //
    // This is not intended as a good example of a device driver implementation,
    // but rather as a real-world example of using the SPIDriver features.

    let port = Serial::new(
        "/dev/ttyUSB0",
        &PortSettings {
            baud_rate: serial_embedded_hal::BaudRate::BaudOther(460800),
            char_size: serial_embedded_hal::CharSize::Bits8,
            parity: serial_embedded_hal::Parity::ParityNone,
            stop_bits: serial_embedded_hal::StopBits::Stop1,
            flow_control: serial_embedded_hal::FlowControl::FlowNone,
        },
    )
    .unwrap();
    let (tx, rx) = port.split();

    let mut sd = SPIDriver::new(tx, rx);

    init(&mut sd).unwrap();

    // We'll allocate a buffer to render our checkerboard pattern into, and then
    // stream it over to the display.
    const WIDTH: usize = 256;
    const HEIGHT: usize = 64;
    const BPP: usize = 4;
    const BUF_SIZE: usize = HEIGHT * WIDTH / (8 / BPP);
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    for y in 0..HEIGHT {
        // each byte represents two pixels
        for x in 0..(WIDTH / 2) {
            buf[(y * WIDTH / 2 + x)] = if y % 2 == 0 { 0xf0 } else { 0x0f };
        }
    }
    cmd_n(&mut sd, 0x5c, &mut buf[..]).unwrap();
}

fn init<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    reset(sd)?;

    // These settings are for the NHD-3.12-25664UCY2 display module, and are
    // derived from its datasheet. Other display modules may need different
    // settings.
    cmd_1(sd, 0xfd, 0b00010010)?; // Disable command lock
    cmd_0(sd, 0xae)?; // Disable display (just during init; we'll enable it again later)
    cmd_2(sd, 0x15, 0x1C, 0x5B)?; // Set column address
    cmd_2(sd, 0x75, 0x00, 0x3F)?; // Set row address
    cmd_1(sd, 0xb3, 0x91)?; // Set display clock
    cmd_1(sd, 0xca, 0x3f)?; // Set multiplex ratio
    cmd_1(sd, 0xa2, 0x00)?; // Set display offset
    cmd_1(sd, 0xa1, 0x00)?; // Set start line
    cmd_2(sd, 0xa0, 0b00010100, 0b00010001)?; // Set remap format
    cmd_1(sd, 0xb5, 0x00)?; // Turn off all GPIO
    cmd_1(sd, 0xab, 0x01)?; // Enable on-board regulator
    cmd_2(sd, 0xb4, 0xA0, 0xFD)?; // Set display Enhancements register A
    cmd_1(sd, 0xc1, 0x9f)?; // Set contrast current
    cmd_1(sd, 0xc7, 0x0f)?; // Set master current
    cmd_0(sd, 0xb9)?; // Select linear grayscale table
    cmd_1(sd, 0xb1, 0xe2)?; // Set phase length
    cmd_2(sd, 0xd1, 0xa2, 0x20)?; // Set display Enhancements register B
    cmd_1(sd, 0xbb, 0x1d)?; // Set precharge voltage
    cmd_1(sd, 0xb6, 0x08)?; // Set precharge period
    cmd_1(sd, 0xbe, 0x07)?; // Set VCOMH
    cmd_0(sd, 0xa6)?; // Normal display mode (not "all on", "all off", or inverted)
    cmd_0(sd, 0xaf)?; // Enable display (turns power on)

    Ok(())
}

fn reset<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    // We'll pulse the B signal on the SPIDriver, which we assume is connected
    // to the reset line on the display driver.
    sd.set_b(false)?;
    sd.set_b(true)
}

fn cmd_0<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
    cmd: u8,
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    sd.select()?;
    sd.set_a(false)?; // Command mode
    sd.write_byte(cmd)?;
    sd.unselect()
}

fn cmd_1<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
    cmd: u8,
    a: u8,
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    sd.select()?;
    sd.set_a(false)?; // Command mode
    sd.write_byte(cmd)?;
    sd.set_a(true)?; // Data mode
    sd.write_byte(a)?;
    sd.unselect()
}

fn cmd_2<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
    cmd: u8,
    a: u8,
    b: u8,
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    sd.select()?;
    sd.set_a(false)?; // Command mode
    sd.write_byte(cmd)?;
    let msg: [u8; 2] = [a, b];
    sd.set_a(true)?; // Data mode
    sd.write(&msg[..])?;
    sd.unselect()
}

fn cmd_n<TX: serial::Write<u8>, RX: serial::Read<u8>>(
    sd: &mut SPIDriver<TX, RX>,
    cmd: u8,
    data: &mut [u8],
) -> Result<(), spidriver::Error<TX::Error, RX::Error>> {
    sd.select()?;
    sd.set_a(false)?; // Command mode
    sd.write_byte(cmd)?;
    let mut remain = data;
    sd.set_a(true)?; // Data mode
    while remain.len() > 0 {
        let len: usize = if remain.len() > 64 { 64 } else { remain.len() };
        let (this, next) = remain.split_at_mut(len);
        sd.write(this)?;
        remain = next;
    }
    sd.unselect()
}
