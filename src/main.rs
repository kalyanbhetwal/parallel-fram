// The project aims to interface 28 pin FM18W08 F-RAM with stm32f303ze processor.
/*
The Address bus pins of FRAM are connected to corresponding pins in stm32f303ze.
FMC_A0 --> PH0
FMC_A1 --> PH1
FMC_A2 --> PF2
FMC_A3 --> PF3
FMC_A4 --> PF4
FMC_A5 --> PF5
FMC_A6 --> PF12
FMC_A7 --> PF13
FMC_A8 --> PF14
FMC_A9 --> PF15
FMC_A10 -->PG0
FMC_A11 -->PG1
FMC_A12 -->PG2
FMC_A13 -->PG3
FMC_A14 -->PG4
The Data bus pins of FRAM are connected to corresponding pins in stm32f303ze.
FMC_DQ0 -->PD14
FMC_DQ1 -->PD15
FMC_DQ2 -->PD0
FMC_DQ3 -->PD1
FMC_DQ4 -->PE7
FMC_DQ5 -->PE8
FMC_DQ6 -->PE9
FMC_DQ7 -->PE10

CS --> PG10
OE --> PD4
WE --> PD5
*/

/*
Typical use of FMC to interface with an SRAM
In this application note, the IS61WV102416BLL memory is used as the reference.
The IS61WV102416BLL memory is a non-multiplexed, asynchronous, 16-bit memory. Bank
1 - NOR/PSRAM sub-bank 1 is selected to support the SRAM device. Based on these data,
FMC is configured as follows:
• Bank 1 - NOR/PSRAM sub-bank 1 is enabled: BCR1_MBKEN bit set to ‘1’.
• Memory type is SRAM: BCR1_MTYP is set to ‘00’ to select the SRAM memory type.
• Data bus width is 16 bits: BCR1_MWID is set to ‘01’ to select the 16-bit width.
• The memory is non-multiplexed: BCR1_MUXEN is reset.
All remaining parameters must be kept cleared.
*/


#![allow(unsafe_code, unused, non_upper_case_globals)]
#![no_main]
#![no_std]
#![no_mangle]
use core::mem;
use core::ptr;
use cortex_m::asm::{nop, self};
use hal::delay::Delay;
use hal::gpio;

use panic_halt as _;

use cortex_m_rt::entry;
use::core::arch::asm;
use cortex_m_semihosting::{debug, hprintln};
use stm32f3xx_hal_v2::{self as hal, 
                        pac,
                        prelude::*,
                        flash::ACR, 
                        pac::Peripherals,
                        pac::FLASH};
#[link_section = ".fram_section"]
static mut test: u32 = 1234;

#[entry]
fn main() -> ! {

   let dp  = Peripherals::take().unwrap();
   let mut rcc = dp.RCC.constrain();
   let mut flash = dp.FLASH.constrain();
   let clocks = rcc.cfgr.sysclk(72.mhz()).freeze(&mut flash.acr);

   let mut fmc = dp.FMC;

   let gpiod = dp.GPIOD;
   let gpioe = dp.GPIOE;
   let gpiof = dp.GPIOF;
   let gpiog = dp.GPIOG;
   let gpioh = dp.GPIOH;

   let mut pd = gpiod.split(&mut rcc.ahb);
   let mut pe = gpioe.split(&mut rcc.ahb);
   let mut pf = gpiof.split(&mut rcc.ahb);
   let mut pg = gpiog.split(&mut rcc.ahb);
   let mut ph = gpioh.split(&mut rcc.ahb);

   let fmc_pins = (
        ph.ph0.into_af12(&mut ph.moder, &mut ph.afrl), //FMC_A0
        ph.ph1.into_af12(&mut ph.moder, &mut ph.afrl), //FMC_A1
        pf.pf2.into_af12(&mut pf.moder, &mut pf.afrl), //FMC_A2
        pf.pf3.into_af12(&mut pf.moder, &mut pf.afrl), //FMC_A3
        pf.pf4.into_af12(&mut pf.moder, &mut pf.afrl), //FMC_A4
        pf.pf5.into_af12(&mut pf.moder, &mut pf.afrl), //FMC_A5
        pf.pf12.into_af12(&mut pf.moder, &mut pf.afrh), //FMC_A6
        pf.pf13.into_af12(&mut pf.moder, &mut pf.afrh), //FMC_A7
        pf.pf14.into_af12(&mut pf.moder, &mut pf.afrh), //FMC_A8
        pf.pf15.into_af12(&mut pf.moder, &mut pf.afrh), //FMC_A9
        pg.pg0.into_af12(&mut pg.moder, &mut pg.afrl), //FMC_A10
        pg.pg1.into_af12(&mut pg.moder, &mut pg.afrl), //FMC_A11
        pg.pg2.into_af12(&mut pg.moder, &mut pg.afrl), //FMC_A12
        pg.pg3.into_af12(&mut pg.moder, &mut pg.afrl), //FMC_A13
        pg.pg4.into_af12(&mut pg.moder, &mut pg.afrl), //FMC_A14
   );
   

   let fmc_pins_data = (
    pd.pd14.into_af12(&mut pd.moder, &mut pd.afrh), // FMC_DQ0
    pd.pd15.into_af12(&mut pd.moder, &mut pd.afrh), // FMC_DQ1
    pd.pd0.into_af12(&mut pd.moder, &mut pd.afrl),  // FMC_DQ2
    pd.pd1.into_af12(&mut pd.moder, &mut pd.afrl),  // FMC_DQ3
    pe.pe7.into_af12(&mut pe.moder, &mut pe.afrl),  // FMC_DQ4
    pe.pe8.into_af12(&mut pe.moder, &mut pe.afrh),  // FMC_DQ5
    pe.pe9.into_af12(&mut pe.moder, &mut pe.afrh),  // FMC_DQ6
    pe.pe10.into_af12(&mut pe.moder, &mut pe.afrh), // FMC_DQ7
);

    let mut cs_pin = pg.pg10.into_af12(&mut pg.moder, &mut pg.afrh);// FMC_NE3 -> CS
    let mut oe_pin = pd.pd4.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NOE -> OE
    let mut we_pin = pd.pd5.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NWE -> WE

    // Set initial states of CS, OE, and WE pins
    // cs_pin.set_high().unwrap(); // Assuming active low for CS
    // oe_pin.set_high().unwrap(); // Assuming active low for OE
    // we_pin.set_high().unwrap(); // Assuming active low for WE


  // dp.RCC.ahbenr.modify(|_,w| w.fmcen().set_bit());
  //enable FMC
  //fmc.bcr1.modify(|_, w|w.fmcen().set_bit());

  // Configure FMC for SRAM memory(in our case F-RAM)
    unsafe{
        fmc.bcr1.modify(|_, w| {
        w.mbken().set_bit(); // Enable FRAM bank 1
        w.mtyp().bits(0b00); // FRAM memory type
        w.mwid().bits(0b00); // 8-bit width
        w.muxen().clear_bit(); // Non-multiplexed
        w
     });

     fmc.btr1.write(|w|  {
       // Set address setup time to 1 cycle
        w.addset().bits(0x1);
        // Set data setup time to 1 cycle
        w.datast().bits(0x1);
        w
    });


    }

    loop {
        // your code goes here
    }
}
