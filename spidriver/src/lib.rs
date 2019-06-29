//! SPIDriver client library
//!
//! This library implements the [SPIDriver](https://spidriver.com/) protocol,
//! allowing Rust programs to interact with an SPIDriver device and in turn
//! to interact with SPI devices.
//!
//! The entry point is `SPIDriver::new`, which takes (and consumes) a serial
//! writer and a serial reader as defined by
//! [`embedded_hal::serial`](https://docs.rs/embedded-hal/0.2.3/embedded_hal/serial/).
//! If you are running on a general computing platform then you can use
//! [`serial_embedded_hal`](https://docs.rs/serial-embedded-hal/0.1.2/serial_embedded_hal/struct.Serial.html)
//! to connect with a serial port provided by your operating system:
//!
//! ```rust
//! let port = Serial::new(
//!     "/dev/ttyUSB0",
//!     &PortSettings {
//!         baud_rate: serial_embedded_hal::BaudRate::BaudOther(460800),
//!         char_size: serial_embedded_hal::CharSize::Bits8,
//!         parity: serial_embedded_hal::Parity::ParityNone,
//!         stop_bits: serial_embedded_hal::StopBits::Stop1,
//!         flow_control: serial_embedded_hal::FlowControl::FlowNone,
//!     },
//! )?;
//! let (tx, rx) = port.split();
//! let sd = SPIDriver::new(tx, rx);
//! ```

#![no_std]

use embedded_hal::serial;

/// `SPIDriver` represents a connected SPIDriver device.
#[derive(Debug)]
pub struct SPIDriver<TX: serial::Write<u8>, RX: serial::Read<u8>> {
    ch: Channel<TX, RX>,
}

impl<TX, RX, TXErr, RXErr> SPIDriver<TX, RX>
where
    TX: serial::Write<u8, Error = TXErr>,
    RX: serial::Read<u8, Error = RXErr>,
{
    /// `new` consumes a serial `Write` and `Read` implementation to produce
    /// an `SPIDriver` object.
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            ch: Channel { tx: tx, rx: rx },
        }
    }

    /// `echo` asks the SPIDriver to echo back the given character.
    ///
    /// This method can be useful for detecting whether the remote device on
    /// the serial line is actually a SPIDriver: ask it to echo back a few
    /// bytes and verify that it does.
    pub fn echo(&mut self, ch: u8) -> Result<u8, Error<TXErr, RXErr>> {
        self.ch.write(b'e')?;
        self.ch.write(ch)?;
        self.ch.flush()?;
        self.ch.read()
    }

    /// `select` asserts the chip select signal by driving it low.
    pub fn select(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b's')?;
        self.ch.flush()
    }

    /// `unselect` de-asserts the chip select signal by driving it high.
    pub fn unselect(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'u')?;
        self.ch.flush()
    }

    /// `set_a` sets the active state of the auxillary "A" pin on the SPIDriver.
    pub fn set_a(&mut self, high: bool) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'a')?;
        if high {
            self.ch.write(b'1')?
        } else {
            self.ch.write(b'0')?
        }
        self.ch.flush()
    }

    /// `set_b` sets the active state of the auxillary "B" pin on the SPIDriver.
    pub fn set_b(&mut self, high: bool) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'b')?;
        if high {
            self.ch.write(b'1')?
        } else {
            self.ch.write(b'0')?
        }
        self.ch.flush()
    }

    /// `disconnect` requests that the SPIDriver disconnect from the SPI signals,
    pub fn disconnect(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'x')
    }

    /// `write` sends up to 64 bytes out over the SPIDriver's MOSI line.
    ///
    /// If the given slice is longer than 64 bytes then `write` will return
    /// the `Request` error.
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error<TXErr, RXErr>> {
        if data.len() == 0 {
            return Ok(()); // nothing to do
        }
        if data.len() > 64 {
            return Err(Error::Request);
        }
        let len = data.len() as u8;
        self.ch.write(0xc0 - 1 + len)?;
        for c in data {
            self.ch.write(*c)?;
        }
        Ok(())
    }

    /// `transfer` sends up to 64 bytes out over the SPIDriver's MOSI line,
    /// and returns the data returned by the target device.
    ///
    /// `transfer` modifies the given array in-place, replacing each byte
    /// with the corresponding byte returned from the device. It then returns
    /// a slice with the same backing array.
    ///
    /// If the given slice is longer than 64 bytes then `write` will return
    /// the `Request` error.
    pub fn transfer<'v>(&mut self, data: &'v mut [u8]) -> Result<&'v [u8], Error<TXErr, RXErr>> {
        if data.len() == 0 {
            return Ok(data); // nothing to do
        }
        if data.len() > 64 {
            return Err(Error::Request);
        }
        let len = data.len() as u8;
        self.ch.write(0x80 - 1 + len)?;
        for i in 0..data.len() {
            self.ch.write(data[i])?;
        }
        for i in 0..data.len() {
            data[i] = self.ch.read()?;
        }
        Ok(data)
    }

    // `write_byte` is like `write` but writes only a single byte.
    //
    // This is a convenience helper to avoid constructing an array and a slice
    // from that array just to send one byte.
    pub fn write_byte(&mut self, b: u8) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(0xc0)?;
        self.ch.write(b)
    }
}

#[derive(Debug)]
struct Channel<TX: serial::Write<u8>, RX: serial::Read<u8>> {
    tx: TX,
    rx: RX,
}

impl<TX, RX, TXErr, RXErr> Channel<TX, RX>
where
    TX: serial::Write<u8, Error = TXErr>,
    RX: serial::Read<u8, Error = RXErr>,
{
    pub fn read(&mut self) -> Result<u8, Error<TXErr, RXErr>> {
        nb::block!(self.rx.read()).map_err(Error::rx)
    }

    pub fn write(&mut self, c: u8) -> Result<(), Error<TXErr, RXErr>> {
        nb::block!(self.tx.write(c)).map_err(Error::tx)
    }

    pub fn flush(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        nb::block!(self.tx.flush()).map_err(Error::tx)
    }
}

/// `Error` represents communication errors.
#[derive(Debug)]
pub enum Error<TXErr, RXErr> {
    /// `Protocol` indicates that the library receieved an invalid or unexpected
    /// response from the Bus Pirate in response to a request.
    Protocol,

    /// `Request` indicates that the caller provided invalid arguments that
    /// could not be checked at compile time.
    Request,

    /// `Write` indicates that the underlying serial write object returned an
    /// error.
    ///
    /// The data is the error returned by the underlying serial implementation.
    Write(TXErr),

    /// `Read` indicates that the underlying serial read object returned an
    /// error.
    ///
    /// The data is the error returned by the underlying serial implementation.
    Read(RXErr),
}

impl<TXErr, RXErr> Error<TXErr, RXErr> {
    fn tx(got: TXErr) -> Self {
        Error::Write(got)
    }

    fn rx(got: RXErr) -> Self {
        Error::Read(got)
    }
}
