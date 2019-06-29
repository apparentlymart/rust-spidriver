use embedded_hal::blocking::spi;
use embedded_hal::digital::v2 as gpiov2;

pub trait Comms {
    type Error;

    fn set_cs(&mut self, active: bool) -> Result<(), Self::Error>;
    fn set_a(&mut self, active: bool) -> Result<(), Self::Error>;
    fn set_b(&mut self, active: bool) -> Result<(), Self::Error>;
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error>;
}

/// `Parts` is a container for the various parts of a SPIDriver that can be
/// used separately via distinct HAL traits.
pub struct Parts<'a, SD: Comms> {
    pub spi: SPI<'a, SD>,
    pub cs: CS<'a, SD>,
    pub pin_a: PinA<'a, SD>,
    pub pin_b: PinB<'a, SD>,
}

impl<'a, SD: 'a> Parts<'a, SD>
where
    SD: Comms,
{
    pub(crate) fn new(sd: &'a SD) -> Self {
        Self {
            spi: SPI::new(sd),
            cs: CS::new(sd),
            pin_a: PinA::new(sd),
            pin_b: PinB::new(sd),
        }
    }
}

/// `SPI` implements some of the SPI-related traits from `embedded-hal` in terms
/// of an SPIDriver device.
pub struct SPI<'a, SD: Comms> {
    sd: &'a SD,
}

impl<'a, SD: 'a> SPI<'a, SD>
where
    SD: Comms,
{
    fn new(sd: &'a SD) -> Self {
        Self { sd: sd }
    }
}

impl<'a, SD: 'a, E> spi::Transfer<u8> for SPI<'a, SD>
where
    SD: Comms<Error = E>,
{
    type Error = E;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], E> {
        panic!("not yet");
    }
}

/// `CS` implements some of the digital IO traits from `embedded-hal` in
/// terms of an SPIDriver device's Chip Select pin.
pub struct CS<'a, SD: Comms> {
    sd: &'a SD,
}

impl<'a, SD: 'a> CS<'a, SD>
where
    SD: Comms,
{
    fn new(sd: &'a SD) -> Self {
        Self { sd: sd }
    }
}

impl<'a, SD: 'a, E> gpiov2::OutputPin for CS<'a, SD>
where
    SD: Comms<Error = E>,
{
    type Error = E;

    fn set_low(&mut self) -> Result<(), E> {
        panic!("not yet");
    }

    fn set_high(&mut self) -> Result<(), E> {
        panic!("not yet");
    }
}

/// `PinA` implements some of the digital IO traits from `embedded-hal` in
/// terms of an SPIDriver device's auxillary output pin A.
pub struct PinA<'a, SD: Comms> {
    sd: &'a SD,
}

impl<'a, SD: 'a> PinA<'a, SD>
where
    SD: Comms,
{
    fn new(sd: &'a SD) -> Self {
        Self { sd: sd }
    }
}

impl<'a, SD: 'a, E> gpiov2::OutputPin for PinA<'a, SD>
where
    SD: Comms<Error = E>,
{
    type Error = E;

    fn set_low(&mut self) -> Result<(), E> {
        panic!("not yet");
    }

    fn set_high(&mut self) -> Result<(), E> {
        panic!("not yet");
    }
}

/// `PinB` implements some of the digital IO traits from `embedded-hal` in
/// terms of an SPIDriver device's auxillary output pin B.
pub struct PinB<'a, SD: Comms> {
    sd: &'a SD,
}

impl<'a, SD: 'a> PinB<'a, SD>
where
    SD: Comms,
{
    fn new(sd: &'a SD) -> Self {
        Self { sd: sd }
    }
}

impl<'a, SD: 'a, E> gpiov2::OutputPin for PinB<'a, SD>
where
    SD: Comms<Error = E>,
{
    type Error = E;

    fn set_low(&mut self) -> Result<(), E> {
        panic!("not yet");
    }

    fn set_high(&mut self) -> Result<(), E> {
        panic!("not yet");
    }
}
