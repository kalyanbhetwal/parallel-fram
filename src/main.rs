// The project aims to interface 28 pin FM18W08 F-RAM with stm32f303ze processor.
/*
The Address bus pins of FRAM are connected to corresponding pins in stm32f303ze.
FMC_A0 --> PH0 (this pin is confusing)
FMC_A1 --> PH1 (this pin is confusing as well)
FMC_A2 --> PF2 (good)
FMC_A3 --> PF3  (good)
FMC_A4 --> PF4 (good)
FMC_A5 --> PF5  (good)
FMC_A6 --> PF12 (good)
FMC_A7 --> PF13 (good)

FMC_A8 --> PF14 (good)
FMC_A9 --> PF15 (good)

FMC_A10 -->PG0 (good)
FMC_A11 -->PG1  (good)

FMC_A12 -->PG2 (good)

FMC_A13 -->PG3 (good)
FMC_A14 -->PG4 (good)

The Data bus pins of FRAM are connected to corresponding pins in stm32f303ze.

FMC_DQ0 -->PD14 (good)
FMC_DQ1 -->PD15 (good)
FMC_DQ2 -->PD0 (good)

FMC_DQ3 -->PD1  --> (good)

FMC_DQ4 -->PE7  --> (good)
FMC_DQ5 -->PE8  --> (good)
FMC_DQ6 -->PE9  --> (good)
FMC_DQ7 -->PE10 --> (good)

CS --> PD7  (FMC_NE1)
OE --> PD4
WE --> PD5
*/


  /* FMC GPIO Configuration
  PH0   ------> FMC_A0
  PH1   ------> FMC_A1
  PF2   ------> FMC_A2
  PF3   ------> FMC_A3
  PF4   ------> FMC_A4
  PF5   ------> FMC_A5
  PF12   ------> FMC_A6
  PF13   ------> FMC_A7
  PF14   ------> FMC_A8
  PF15   ------> FMC_A9
  PG0   ------> FMC_A10
  PG1   ------> FMC_A11

  PE7   ------> FMC_D4
  PE8   ------> FMC_D5
  PE9   ------> FMC_D6
  PE10   ------> FMC_D7
  PE11   ------> FMC_D8
  PE12   ------> FMC_D9
  PE13   ------> FMC_D10
  PE14   ------> FMC_D11
  PE15   ------> FMC_D12
  PD8   ------> FMC_D13
  PD9   ------> FMC_D14
  PD10   ------> FMC_D15
  PD14   ------> FMC_D0
  PD15   ------> FMC_D1


  PG2   ------> FMC_A12
  PG3   ------> FMC_A13
  PG4   ------> FMC_A14
  PG5   ------> FMC_A15
  PD0   ------> FMC_D2
  PD1   ------> FMC_D3
  PD4   ------> FMC_NOE
  PD5   ------> FMC_NWE
  PD7   ------> FMC_NE1
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
static DATA_ARRAY: [u8; 5] = [0x12, 0x34, 0xAB, 0xCD, 0xEF];

unsafe fn write_16bit(ptr: *mut u16, value: u16) {
    // Split the 16-bit value into two 8-bit values
    let low_byte = (value & 0xFF) as u8;
    let high_byte = (value >> 8) as u8;

    // Write the high byte next
    ptr::write_volatile((ptr as *mut u8).add(2), high_byte);
    // Write the low byte first
    ptr::write_volatile(ptr as *mut u8, low_byte);

}

unsafe fn read_16bit(ptr: *const u16) -> u16 {
    // Read the low byte first
    let low_byte = ptr::read_volatile(ptr as *const u8);

    // Read the high byte next
    let high_byte = ptr::read_volatile((ptr as *const u8).offset(2));

    // Combine the two bytes into a 16-bit value
    ((high_byte as u16) << 8) | (low_byte as u16)
}

#[entry]
fn main() -> ! {

   let dp  = Peripherals::take().unwrap();
   
    //enable HSI
   dp.RCC.cr.write(|w| w.hsion().set_bit());
   while dp.RCC.cr.read().hsirdy().bit_is_clear() {}

    //configure PLL
    // Step 1: Disable the PLL by setting PLLON to 0
    dp.RCC.cr.modify(|_r, w| w.pllon().clear_bit());

    // Step 2: Wait until PLLRDY is cleared
    while dp.RCC.cr.read().pllrdy().bit_is_set() {}

    // Step 3: Change the desired parameter
    // For example, modify PLL multiplier (PLLMUL)

    dp.RCC.cfgr.modify(|_, w| w.pllsrc().hsi_div_prediv());

    // Set PLL Prediv to /1
    dp.RCC.cfgr2.modify(|_, w| w.prediv().div1());

    // Set PLL MUL to x9
    dp.RCC.cfgr.modify(|_, w| w.pllmul().mul9());

    // Step 4: Enable the PLL again by setting PLLON to 1
   // dp.RCC.cr.modify(|_r, w| w.pllon().set_bit());

    dp.RCC.cr.modify(|_, w| w.pllon().on());

    while dp.RCC.cr.read().pllrdy().bit_is_clear(){}

       // Configure prescalar values for HCLK, PCLK1, and PCLK2
       dp.RCC.cfgr.modify(|_, w| {
        w.hpre().div1() // HCLK prescaler: no division
        .ppre1().div2() // PCLK1 prescaler: divide by 2
        .ppre2().div1() // PCLK2 prescaler: no division
    });


    // Enable FLASH Prefetch Buffer and set Flash Latency (required for high speed)
    // was crashing just because this was missing
    dp.FLASH.acr
        .modify(|_, w| w.prftbe().enabled().latency().ws1());

     // Select PLL as system clock source
     dp.RCC.cfgr.modify(|_, w| w.sw().pll());

     while dp.RCC.cfgr.read().sw().bits() != 0b10 {}

      // Wait for system clock to stabilize
      while dp.RCC.cfgr.read().sws().bits() != 0b10 {}

     dp.RCC.ahbenr.modify(|_, w| w.iopden().set_bit());
     dp.RCC.ahbenr.modify(|_, w| w.iopeen().set_bit());
     dp.RCC.ahbenr.modify(|_, w| w.iopfen().set_bit());
     dp.RCC.ahbenr.modify(|_, w| w.iopgen().set_bit());
     dp.RCC.ahbenr.modify(|_, w| w.iophen().set_bit());  
     dp.RCC.ahbenr.modify(|_, w| w.sramen().set_bit());  
     dp.RCC.ahbenr.modify(|_, w| w.flitfen().set_bit());  
     dp.RCC.ahbenr.modify(|_, w| w.fmcen().set_bit());  


     dp.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
     dp.RCC.apb1enr.modify(|_, w| w.pwren().set_bit());


   //dp.RCC.cr.write(f);

   //let mut fmc = &dp.FMC;

   let mut gpiod = dp.GPIOD;
   let mut gpioe = dp.GPIOE;
   let mut gpiof = dp.GPIOF;
   let mut gpiog = dp.GPIOG;
   let mut gpioh = dp.GPIOH;

//    let mut rcc = dp.RCC.constrain();

//    let mut pd = gpiod.split(&mut rcc.ahb);
//    let mut pe = gpioe.split(&mut rcc.ahb);
//    let mut pf = gpiof.split(&mut rcc.ahb);
//    let mut pg = gpiog.split(&mut rcc.ahb);
//    let mut ph = gpioh.split(&mut rcc.ahb);

    // ph.ph0.into_af12(&mut ph.moder, &mut ph.afrl); //FMC_A0
    // ph.ph1.into_af12(&mut ph.moder, &mut ph.afrl); //FMC_A1
    // pf.pf2.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A2
    // pf.pf3.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A3
    // pf.pf4.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A4
    // pf.pf5.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A5


//    PH0   ------> FMC_A0
gpioh.moder.modify(|_, w| {w.moder0().alternate()});
gpioh.afrl.modify(|_, w| {  w.afrl0().af12()});
gpioh.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

// PH1   ------> FMC_A1
gpioh.moder.modify(|_, w| {w.moder1().alternate()});
gpioh.afrl.modify(|_, w| {  w.afrl1().af12()});
gpioh.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

//  PF2   ------> FMC_A2
gpiof.moder.modify(|_, w| {w.moder2().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl2().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

//   PF3   ------> FMC_A3
gpiof.moder.modify(|_, w| {w.moder3().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl3().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

//   PF4   ------> FMC_A4
gpiof.moder.modify(|_, w| {w.moder4().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl4().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());


// PF5   ------> FMC_A5
gpiof.moder.modify(|_, w| {w.moder5().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl5().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());

    
    // pf.pf12.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A6
    // pf.pf13.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A7
    // pf.pf14.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A8
    // pf.pf15.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A9


//    PF12   ------> FMC_A6
gpiof.moder.modify(|_, w| {w.moder12().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh12().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//   PF13   ------> FMC_A7
gpiof.moder.modify(|_, w| {w.moder13().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh13().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());

//   PF14   ------> FMC_A8
gpiof.moder.modify(|_, w| {w.moder14().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh14().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

//  PF15   ------> FMC_A9
gpiof.moder.modify(|_, w| {w.moder15().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh15().af12()});
gpiof.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

  // pg.pg0.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A10
    // pg.pg1.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A11
    // pg.pg2.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A12
    // pg.pg3.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A13
    // pg.pg4.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A14

    // PG0   ------> FMC_A10
    gpiog.moder.modify(|_, w| {w.moder0().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl0().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

    //  PG1   ------> FMC_A11
    gpiog.moder.modify(|_, w| {w.moder1().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl1().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

    //  PG2   ------> FMC_A12
    gpiog.moder.modify(|_, w| {w.moder2().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl2().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

    //    PG3   ------> FMC_A13
    gpiog.moder.modify(|_, w| {w.moder3().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl3().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

    //   PG4   ------> FMC_A14
    gpiog.moder.modify(|_, w| {w.moder4().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl4().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());

    //PG5   ------> FMC_A15
    gpiog.moder.modify(|_, w| {w.moder5().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl5().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());



    // pd.pd14.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ0
    // pd.pd15.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ1
    // pd.pd0.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ2
    // pd.pd1.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ3
    // pe.pe7.into_af12(&mut pe.moder, &mut pe.afrl);  // FMC_DQ4
    // pe.pe8.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ5
    // pe.pe9.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ6
    // pe.pe10.into_af12(&mut pe.moder, &mut pe.afrh); // FMC_DQ7

//  PD14   ------> FMC_D0
gpiod.moder.modify(|_, w| {w.moder14().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh14().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

//  PD15   ------> FMC_D1
gpiod.moder.modify(|_, w| {w.moder15().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh15().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

// PD0   ------> FMC_D2
gpiod.moder.modify(|_, w| {w.moder0().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl0().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());


// PD1   ------> FMC_D3
gpiod.moder.modify(|_, w| {w.moder1().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl1().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());


//PE7   ------> FMC_D4

gpioe.moder.modify(|_, w| {w.moder7().alternate()});
gpioe.afrl.modify(|_, w| {  w.afrl7().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());


//PE8   ------> FMC_D5
gpioe.moder.modify(|_, w| {w.moder8().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh8().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());

// PE9   ------> FMC_D6
gpioe.moder.modify(|_, w| {w.moder9().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh9().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());

//PE10   ------> FMC_D7
gpioe.moder.modify(|_, w| {w.moder10().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh10().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());

//PE11   ------> FMC_D8
gpioe.moder.modify(|_, w| {w.moder11().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh11().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr11().very_high_speed());

//PE12   ------> FMC_D9
gpioe.moder.modify(|_, w| {w.moder12().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh12().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//PE13   ------> FMC_D10
gpioe.moder.modify(|_, w| {w.moder13().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh13().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//PE14   ------> FMC_D11
gpioe.moder.modify(|_, w| {w.moder14().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh14().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

//PE15   ------> FMC_D12
gpioe.moder.modify(|_, w| {w.moder15().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh15().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

//PD8   ------> FMC_D13
gpioe.moder.modify(|_, w| {w.moder13().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh13().af12()});
gpioe.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//PD9   ------> FMC_D14
gpiod.moder.modify(|_, w| {w.moder9().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh9().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//PD10   ------> FMC_D15
gpiod.moder.modify(|_, w| {w.moder13().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh13().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());


// pd.pd7.into_af12(&mut pd.moder, &mut pd.afrl);// FMC_NE3 -> CS
// pd.pd4.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NOE -> OE
// pd.pd5.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NWE -> WE

// PD4   ------> FMC_NOE
// PD5   ------> FMC_NWE
// PD7   ------> FMC_NE1

gpiod.moder.modify(|_, w| {w.moder7().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl7().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());


gpiod.moder.modify(|_, w| {w.moder4().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl4().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());


gpiod.moder.modify(|_, w| {w.moder5().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl5().af12()});
gpiod.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());




    // ph.ph0.into_af12(&mut ph.moder, &mut ph.afrl); //FMC_A0
    // ph.ph1.into_af12(&mut ph.moder, &mut ph.afrl); //FMC_A1
    // pf.pf2.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A2
    // pf.pf3.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A3
    // pf.pf4.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A4
    // pf.pf5.into_af12(&mut pf.moder, &mut pf.afrl); //FMC_A5

    // pf.pf12.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A6
    // pf.pf13.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A7
    // pf.pf14.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A8
    // pf.pf15.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A9

    // pg.pg0.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A10
    // pg.pg1.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A11
    // pg.pg2.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A12
    // pg.pg3.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A13
    // pg.pg4.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A14

    // pd.pd14.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ0
    // pd.pd15.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ1
    // pd.pd0.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ2
    // pd.pd1.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ3
    // pe.pe7.into_af12(&mut pe.moder, &mut pe.afrl);  // FMC_DQ4
    // pe.pe8.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ5
    // pe.pe9.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ6
    // pe.pe10.into_af12(&mut pe.moder, &mut pe.afrh); // FMC_DQ7

    // pd.pd7.into_af12(&mut pd.moder, &mut pd.afrl);// FMC_NE3 -> CS
    // pd.pd4.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NOE -> OE
    // pd.pd5.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NWE -> WE


  // dp.RCC.ahbenr.modify(|_,w| w.fmcen().set_bit());

  // Configure FMC for SRAM memory(in our case F-RAM)
    unsafe{
        dp.FMC.bcr1.modify(|_, w| {
        w.mbken().set_bit(); // Enable FRAM bank 1
        w.mtyp().bits(0b00); // FRAM memory type
        w.mwid().bits(0b01); // 8-bit width
        w.bursten().clear_bit(); //disable brust access mode
        w.wren().clear_bit(); // wrap disable
        w.muxen().clear_bit(); // Non-multiplexed
        w.extmod().clear_bit(); // extended mode
        w.asyncwait().clear_bit(); //disable async wait
        w
     });

     /*
        Timing.AddressSetupTime = 1;
        Timing.AddressHoldTime = 1;
        Timing.DataSetupTime = 5;
        Timing.BusTurnAroundDuration = 0;
        Timing.CLKDivision = 0;
        Timing.DataLatency = 0;
        Timing.AccessMode = FMC_ACCESS_MODE_A;
   */
     dp.FMC.btr1.modify(|_,w|  {
       // Set address setup time to 1 cycle
        w.addset().bits(0x0F);
        // Set data setup time to 5 cycle
        w.datast().bits(0xFF);
        // address hold time
        w.addhld().bits(0x0F);
        // bus turn around
        w.busturn().bits(0x0);
        // clock division
        w.clkdiv().bits(0x10);
        //data latency
        w.datlat().bits(0x11);
        //access mode
        w.accmod().bits(0x0);

        w
    });
}
 let mut ans = [0;60];
    unsafe{
        for i in (0..60).step_by(2){
            unsafe { ptr::write_volatile((0x6000_0000 +i+1) as *mut u8, (i+1) as u8) };
        }


        for i in (0..60).step_by(2){
            ans[i] = unsafe { ptr::read_volatile((0x6000_0000 +i+1) as *mut u8) };
            hprintln!("Value at index {}: {}", i, ans[i]).unwrap();
        }
        hprintln!("Value at index {:?}", ans).unwrap();


   // let a =     ptr::write_volatile(0x6000_0009 as *mut u8, 41);  // write_16bit(0x6000_0000 as *mut u16, 0xBEEF);
    let a =   ptr::read_volatile(0x6000_0008 as *mut u16);//read_16bit(0x6000_0000 as *mut u16);
    hprintln!("{:0x}", a);
    }
    loop {
        // your code goes here
    }
}
