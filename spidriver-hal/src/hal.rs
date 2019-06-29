use core::marker::PhantomData;
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
pub struct Parts<'a, SD: Comms, MUT: mutex::Mutex<SD>> {
    m: MUT,
    pub spi: SPI<'a, SD, MUT>,
    pub cs: CS<'a, SD, MUT>,
    pub pin_a: PinA<'a, SD, MUT>,
    pub pin_b: PinB<'a, SD, MUT>,
    _0: PhantomData<SD>,
}

impl<'a, SD: 'a, MUT: 'a> Parts<'a, SD, MUT>
where
    SD: Comms,
    MUT: mutex::Mutex<SD>,
{
    pub(crate) fn new(sd: SD) -> Self {
        let m = MUT::wrap(sd);
        Self {
            m: m,
            spi: SPI::new(&m),
            cs: CS::new(&m),
            pin_a: PinA::new(&m),
            pin_b: PinB::new(&m),
            _0: PhantomData,
        }
    }
}

/// `SPI` implements some of the SPI-related traits from `embedded-hal` in terms
/// of an SPIDriver device.
pub struct SPI<'a, SD: Comms, MUT: mutex::Mutex<SD>> {
    sd: &'a MUT,
    _0: PhantomData<SD>,
}

impl<'a, SD: 'a, MUT: 'a> SPI<'a, SD, MUT>
where
    SD: Comms,
    MUT: mutex::Mutex<SD>,
{
    fn new(sd: &'a MUT) -> Self {
        Self {
            sd: sd,
            _0: PhantomData,
        }
    }
}

impl<'a, SD: 'a, MUT: 'a, E> spi::Transfer<u8> for SPI<'a, SD, MUT>
where
    SD: Comms<Error = E>,
    MUT: mutex::Mutex<SD>,
{
    type Error = E;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], E> {
        self.sd.borrow(|sd| sd.transfer(data))
    }
}

impl<'a, SD: 'a, MUT: 'a, E> spi::Write<u8> for SPI<'a, SD, MUT>
where
    SD: Comms<Error = E>,
    MUT: mutex::Mutex<SD>,
{
    type Error = E;

    fn write(&mut self, data: &[u8]) -> Result<(), E> {
        self.sd.borrow(|sd| sd.write(data))
    }
}

/// `CS` implements some of the digital IO traits from `embedded-hal` in
/// terms of an SPIDriver device's Chip Select pin.
pub struct CS<'a, SD: Comms, MUT: mutex::Mutex<SD>> {
    sd: &'a MUT,
    _0: PhantomData<SD>,
}

impl<'a, SD: 'a, MUT: 'a> CS<'a, SD, MUT>
where
    SD: Comms,
    MUT: mutex::Mutex<SD>,
{
    fn new(sd: &'a MUT) -> Self {
        Self {
            sd: sd,
            _0: PhantomData,
        }
    }
}

impl<'a, SD: 'a, MUT: 'a, E> gpiov2::OutputPin for CS<'a, SD, MUT>
where
    SD: Comms<Error = E>,
    MUT: mutex::Mutex<SD>,
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
pub struct PinA<'a, SD: Comms, MUT: mutex::Mutex<SD>> {
    sd: &'a MUT,
    _0: PhantomData<SD>,
}

impl<'a, SD: 'a, MUT: 'a> PinA<'a, SD, MUT>
where
    SD: Comms,
    MUT: mutex::Mutex<SD>,
{
    fn new(sd: &'a MUT) -> Self {
        Self {
            sd: sd,
            _0: PhantomData,
        }
    }
}

impl<'a, SD: 'a, MUT: 'a, E> gpiov2::OutputPin for PinA<'a, SD, MUT>
where
    SD: Comms<Error = E>,
    MUT: mutex::Mutex<SD>,
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
pub struct PinB<'a, SD: Comms, MUT: mutex::Mutex<SD>> {
    sd: &'a MUT,
    _0: PhantomData<SD>,
}

impl<'a, SD: 'a, MUT: 'a> PinB<'a, SD, MUT>
where
    SD: Comms,
    MUT: mutex::Mutex<SD>,
{
    fn new(sd: &'a MUT) -> Self {
        Self {
            sd: sd,
            _0: PhantomData,
        }
    }
}

impl<'a, SD: 'a, MUT: 'a, E> gpiov2::OutputPin for PinB<'a, SD, MUT>
where
    SD: Comms<Error = E>,
    MUT: mutex::Mutex<SD>,
{
    type Error = E;

    fn set_low(&mut self) -> Result<(), E> {
        panic!("not yet");
    }

    fn set_high(&mut self) -> Result<(), E> {
        panic!("not yet");
    }
}

/// Module `mutex` contains helper traits for handling safe concurrent access
/// to the separate parts of an SPIDriver.
pub mod mutex {
    /// `Mutex<T>` is an intermediary that ensures that only one thread can be
    /// working with a particular object at a time.
    pub trait Mutex<'a, T> {
        fn wrap(v: &'a T) -> Self;
        fn borrow<R, F: core::ops::FnOnce(&'a T) -> R>(&self, f: F) -> R;
    }

    pub struct NoOpMutex<'a, T> {
        v: &'a T,
    }

    impl<'a, T> Mutex<'a, T> for NoOpMutex<'a, T> {
        fn wrap(v: T) -> Self {
            Self { v: v }
        }

        fn borrow<R, F: core::ops::FnOnce(&T) -> R>(&self, f: F) -> R {
            f(&self.v)
        }
    }
}
