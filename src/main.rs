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
static mut abc: u32 = 4687;

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

   //dp.RCC.cr.write(f);

   //let mut fmc = &dp.FMC;

   let gpiod = dp.GPIOD;
   let gpioe = dp.GPIOE;
   let gpiof = dp.GPIOF;
   let gpiog = dp.GPIOG;
   let gpioh = dp.GPIOH;

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

gpioh.moder.modify(|_, w| {w.moder0().alternate()});
gpioh.afrl.modify(|_, w| {  w.afrl0().af12()});

gpioh.moder.modify(|_, w| {w.moder1().alternate()});
gpioh.afrl.modify(|_, w| {  w.afrl1().af12()});

gpiof.moder.modify(|_, w| {w.moder2().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl2().af12()});

gpiof.moder.modify(|_, w| {w.moder3().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl3().af12()});

gpiof.moder.modify(|_, w| {w.moder4().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl4().af12()});

gpiof.moder.modify(|_, w| {w.moder5().alternate()});
gpiof.afrl.modify(|_, w| {  w.afrl5().af12()});

    
    // pf.pf12.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A6
    // pf.pf13.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A7
    // pf.pf14.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A8
    // pf.pf15.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A9

gpiof.moder.modify(|_, w| {w.moder12().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh12().af12()});

gpiof.moder.modify(|_, w| {w.moder13().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh13().af12()});

gpiof.moder.modify(|_, w| {w.moder14().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh14().af12()});

gpiof.moder.modify(|_, w| {w.moder15().alternate()});
gpiof.afrh.modify(|_, w| {  w.afrh15().af12()});

  // pg.pg0.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A10
    // pg.pg1.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A11
    // pg.pg2.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A12
    // pg.pg3.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A13
    // pg.pg4.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A14

    gpiog.moder.modify(|_, w| {w.moder0().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl0().af12()});
    
    gpiog.moder.modify(|_, w| {w.moder1().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl1().af12()});
    
    gpiog.moder.modify(|_, w| {w.moder2().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl2().af12()});
    
    gpiog.moder.modify(|_, w| {w.moder3().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl3().af12()});
    
    gpiog.moder.modify(|_, w| {w.moder4().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl4().af12()});


    // pd.pd14.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ0
    // pd.pd15.into_af12(&mut pd.moder, &mut pd.afrh); // FMC_DQ1
    // pd.pd0.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ2
    // pd.pd1.into_af12(&mut pd.moder, &mut pd.afrl);  // FMC_DQ3
    // pe.pe7.into_af12(&mut pe.moder, &mut pe.afrl);  // FMC_DQ4
    // pe.pe8.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ5
    // pe.pe9.into_af12(&mut pe.moder, &mut pe.afrh);  // FMC_DQ6
    // pe.pe10.into_af12(&mut pe.moder, &mut pe.afrh); // FMC_DQ7

gpiod.moder.modify(|_, w| {w.moder14().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh14().af12()});

gpiod.moder.modify(|_, w| {w.moder15().alternate()});
gpiod.afrh.modify(|_, w| {  w.afrh15().af12()});


gpiod.moder.modify(|_, w| {w.moder0().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl0().af12()});

gpiod.moder.modify(|_, w| {w.moder1().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl1().af12()});

gpioe.moder.modify(|_, w| {w.moder7().alternate()});
gpioe.afrl.modify(|_, w| {  w.afrl7().af12()});

gpioe.moder.modify(|_, w| {w.moder8().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh8().af12()});

gpioe.moder.modify(|_, w| {w.moder9().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh9().af12()});

gpioe.moder.modify(|_, w| {w.moder10().alternate()});
gpioe.afrh.modify(|_, w| {  w.afrh10().af12()});


// pd.pd7.into_af12(&mut pd.moder, &mut pd.afrl);// FMC_NE3 -> CS
// pd.pd4.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NOE -> OE
// pd.pd5.into_af12(&mut pd.moder, &mut pd.afrl); // FMC_NWE -> WE

gpiod.moder.modify(|_, w| {w.moder7().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl7().af12()});

gpiod.moder.modify(|_, w| {w.moder4().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl4().af12()});

gpiod.moder.modify(|_, w| {w.moder5().alternate()});
gpiod.afrl.modify(|_, w| {  w.afrl5().af12()});

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
        w.mwid().bits(0b00); // 8-bit width
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
        w.addset().bits(0x1);
        // Set data setup time to 1 cycle
        w.datast().bits(0x5);
        // address hold time
        w.addhld().bits(0x1);
        // bus turn around
        w.busturn().bits(0x0);
        // clock division
        w.clkdiv().bits(0x0);
        //data latency
        w.datlat().bits(0x0);
        //access mode
        w.accmod().bits(0x0);

        w
    });
}

    let address = 0x6000_0008;
    unsafe{
        let data_ptr =  address as *mut u8;
        ptr::write_volatile(address as *mut u8, 12u8);

        let mut data = ptr::read_volatile(address as *mut u8);
        hprintln!("Read data is {}", data).unwrap();
    }
    loop {
        // your code goes here
    }
}
