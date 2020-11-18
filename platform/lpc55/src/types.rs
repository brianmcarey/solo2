include!(concat!(env!("OUT_DIR"), "/build_constants.rs"));
use crate::hal;
use hal::drivers::{pins,pins::Pin,SpiMaster, timer};
use littlefs2::{
    const_ram_storage,
};
use trussed::types::{LfsResult, LfsStorage};
use trussed::{board, store};
use ctap_types::consts;
use fm11nc08::FM11NC08;
use hal::{
    typestates::{
        pin::flexcomm::{
            NoPio,
        },
        pin::{
            state,
            function,
            gpio::{
                direction,
            }
        }
    },
    peripherals::ctimer,
};

#[cfg(feature = "no-encrypted-storage")]
use hal::littlefs2_filesystem;
#[cfg(not(feature = "no-encrypted-storage"))]
use hal::littlefs2_prince_filesystem;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(PlainFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));
#[cfg(not(feature = "no-encrypted-storage"))]
littlefs2_prince_filesystem!(PrinceFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));

#[cfg(feature = "no-encrypted-storage")]
pub type FlashStorage = PlainFilesystem;
#[cfg(not(feature = "no-encrypted-storage"))]
pub type FlashStorage = PrinceFilesystem;

pub mod usb;
pub use usb::{UsbClasses, EnabledUsbPeripheral, SerialClass, CcidClass, CtapHidClass};

// 8KB of RAM
const_ram_storage!(
    name=VolatileStorage,
    trait=LfsStorage,
    erase_value=0xff,
    read_size=1,
    write_size=1,
    cache_size_ty=consts::U128,
    // this is a limitation of littlefs
    // https://git.io/JeHp9
    block_size=104,
    // block_size=128,
    block_count=8192/104,
    lookaheadwords_size_ty=consts::U8,
    filename_max_plus_one_ty=consts::U256,
    path_max_plus_one_ty=consts::U256,
    result=LfsResult,
);

// minimum: 2 blocks
// TODO: make this optional
const_ram_storage!(ExternalStorage, 1024);

store!(Store,
    Internal: FlashStorage,
    External: ExternalStorage,
    Volatile: VolatileStorage
);

pub type ThreeButtons = crate::board::button::ThreeButtons;
pub type RgbLed = crate::board::led::RgbLed;

board!(Board,
    R: hal::peripherals::rng::Rng<hal::Enabled>,
    S: Store,
    UI: crate::solo_trussed::UserInterface<ThreeButtons, RgbLed>,
);

#[derive(Default)]
pub struct Syscall {}

impl trussed::client::Syscall for Syscall {
    #[inline]
    fn syscall(&mut self) {
        rtic::pend(lpc55_hal::raw::Interrupt::OS_EVENT);
    }
}

pub type Trussed = trussed::Service<Board>;
pub type TrussedClient = trussed::DefaultClient<Syscall>;

pub type NfcSckPin = pins::Pio0_28;
pub type NfcMosiPin = pins::Pio0_24;
pub type NfcMisoPin = pins::Pio0_25;
pub type NfcCsPin = pins::Pio1_20;
pub type NfcIrqPin = pins::Pio0_19;

pub type NfcChip = FM11NC08<
            SpiMaster<
                NfcSckPin,
                NfcMosiPin,
                NfcMisoPin,
                NoPio,
                hal::peripherals::flexcomm::Spi0,
                (
                    Pin<NfcSckPin, state::Special<function::FC0_SCK>>,
                    Pin<NfcMosiPin, state::Special<function::FC0_RXD_SDA_MOSI_DATA>>,
                    Pin<NfcMisoPin, state::Special<function::FC0_TXD_SCL_MISO_WS>>,
                    hal::typestates::pin::flexcomm::NoCs,
                )
                >,
                Pin<NfcCsPin, state::Gpio<direction::Output>>,
                Pin<NfcIrqPin, state::Gpio<direction::Input>>,
            >;
pub type Iso14443 = iso14443::Iso14443<NfcChip>;

pub type ExternalInterrupt = hal::Pint<hal::typestates::init_state::Enabled>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch;

pub type HidDispatch = hid_dispatch::dispatch::Dispatch;

pub type Piv = piv_card::App<TrussedClient>;

pub type FidoApplet<UP> = applet_fido::Fido<UP, TrussedClient>;

pub type RootApp = applet_root::Root<TrussedClient>;

pub type PerfTimer = timer::Timer<ctimer::Ctimer4<hal::typestates::init_state::Enabled>>;

pub type DynamicClockController = crate::clock_controller::DynamicClockController;

pub type SignalPin = pins::Pio0_23;
pub type SignalButton = Pin<SignalPin, state::Gpio<direction::Output>>;

pub type HwScheduler = timer::Timer<ctimer::Ctimer0<hal::typestates::init_state::Enabled>>;