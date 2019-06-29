//! SPIDriver implementations of some embedded-hal traits.
//!
//! This library provides implementations of several `embedded-hal` crates
//! in terms of the [SPIDriver](https://spidriver.com/) protocol. This allows
//! driver crates that are written in terms of those traits to control their
//! corresponding devices via an SPIDriver module.
//!
//! Specifically, this library provides:
//! - Implementations of the blocking SPI `Write` and `Transfer` traits that
//!   transmit data via the SPIDriver.
//! - An implementation of the v2 Digital IO `OutputPin` trait for the chip
//!   select output of the SPIDriver.
//! - Implementations of the v2 Digital IO `OutputPin` trait for the auxillary
//!   output pins A and B on the SPIDriver.
//!
//! To use it, first instantiate and configure an `SPIDriver` object from the
//! `spidriver` crate, and then pass it to `SPIDriverHAL::new` before calling
//! `split` to obtain the individual interface objects:
//!
//! ```rust
//! let sd = SPIDriver::new(rx, tx); // rx and tx obtained from some underlying platform crate
//! let parts = SPIDriverHAL::new(sd).split();
//! ```

#![no_std]

extern crate embedded_hal;

pub mod hal;

use spidriver::SPIDriver;

use hal::{Comms, Parts};

/// `SPIDriverHAL` is the entry point for this library.
pub struct SPIDriverHAL<
    UARTTX: embedded_hal::serial::Write<u8>,
    UARTRX: embedded_hal::serial::Read<u8>,
> {
    sd: SPIDriver<UARTTX, UARTRX>,
}

impl<TX, RX> SPIDriverHAL<TX, RX>
where
    TX: embedded_hal::serial::Write<u8>,
    RX: embedded_hal::serial::Read<u8>,
{
    pub fn new(sd: SPIDriver<TX, RX>) -> Self {
        Self { sd: sd }
    }

    pub fn split<'a>(&'a self) -> Parts<'a, Self> {
        Parts::new(&self)
    }
}

impl<TX, RX, TXErr, RXErr> Comms for SPIDriverHAL<TX, RX>
where
    TX: embedded_hal::serial::Write<u8, Error = TXErr>,
    RX: embedded_hal::serial::Read<u8, Error = RXErr>,
{
    type Error = spidriver::Error<TXErr, RXErr>;

    fn set_cs(&mut self, high: bool) -> Result<(), Self::Error> {
        if high {
            self.sd.unselect() // SPI is active low, so high means unselected
        } else {
            self.sd.select()
        }
    }

    fn set_a(&mut self, high: bool) -> Result<(), Self::Error> {
        self.sd.set_a(high)
    }

    fn set_b(&mut self, high: bool) -> Result<(), Self::Error> {
        self.sd.set_b(high)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut remain = data;
        while remain.len() > 0 {
            let len: usize = if remain.len() > 64 { 64 } else { remain.len() };
            let (this, next) = remain.split_at(len);
            self.sd.write(this)?;
            remain = next;
        }
        Ok(())
    }

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let mut remain = &mut data[..];
        while remain.len() > 0 {
            let len: usize = if remain.len() > 64 { 64 } else { remain.len() };
            let (this, next) = remain.split_at_mut(len);
            self.sd.transfer(this)?;
            remain = next;
        }
        Ok(data)
    }
}
