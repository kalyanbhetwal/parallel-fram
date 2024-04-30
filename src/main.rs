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
   let mut fmc = dp.FMC;
  // dp.RCC.ahbenr.modify(|_,w| w.fmcen().set_bit());

  // Configure FMC for SRAM memory(in our case F-RAM)
    unsafe{
        fmc.bcr1.modify(|_, w| {
        w.mbken().set_bit(); // Enable FRAM bank 1
        w.mtyp().bits(0b00); // FRAM memory type
        w.mwid().bits(0b00); // 8-bit width
        w.muxen().clear_bit(); // Non-multiplexed
        w
     });

        fmc.btr1.write(|w| unsafe {
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
