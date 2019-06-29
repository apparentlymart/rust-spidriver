#![no_std]

use embedded_hal::serial;

#[derive(Debug)]
pub struct SPIDriver<TX: serial::Write<u8>, RX: serial::Read<u8>> {
    ch: Channel<TX, RX>,
}

impl<TX, RX, TXErr, RXErr> SPIDriver<TX, RX>
where
    TX: serial::Write<u8, Error = TXErr>,
    RX: serial::Read<u8, Error = RXErr>,
{
    pub fn new(tx: TX, rx: RX) -> Self {
        Self {
            ch: Channel { tx: tx, rx: rx },
        }
    }

    pub fn echo(&mut self, ch: u8) -> Result<u8, Error<TXErr, RXErr>> {
        self.ch.write(b'e')?;
        self.ch.write(ch)?;
        self.ch.flush()?;
        self.ch.read()
    }

    pub fn select(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b's')?;
        self.ch.flush()
    }

    pub fn unselect(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'u')?;
        self.ch.flush()
    }

    pub fn set_a(&mut self, high: bool) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'a')?;
        if high {
            self.ch.write(b'1')?
        } else {
            self.ch.write(b'0')?
        }
        self.ch.flush()
    }

    pub fn set_b(&mut self, high: bool) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'b')?;
        if high {
            self.ch.write(b'1')?
        } else {
            self.ch.write(b'0')?
        }
        self.ch.flush()
    }

    pub fn disconnect(&mut self) -> Result<(), Error<TXErr, RXErr>> {
        self.ch.write(b'x')
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error<TXErr, RXErr>> {
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

    pub fn transfer<'v>(&mut self, data: &'v mut [u8]) -> Result<&'v [u8], Error<TXErr, RXErr>> {
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
