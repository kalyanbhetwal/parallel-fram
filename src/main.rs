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
  PD4   ------> FMC_NOE (OE)
  PD5   ------> FMC_NWE (WE)
  PD7   ------> FMC_NE1 (CS)
  */

/*
Typical use of FMC to interface with an SRAM
In this application note, the IS61WV102416BLL memory is used as the reference.
The IS61WV102416BLL memory is a non-multiplexed, asynchronous, 16-bit memory. Bank
1 - NOR/PSRAM sub-bank 1 is selected to support the SRAM device. Based on these data,
FMC is configured as follows:
• Bank 1 - NOR/PSRAM sub-bank 1 is enabled: BCR1_MBKEN bit set to ‘1’.
• Memory type is SRAM: BCR1_MTYP is set to ‘00’ 
to select the SRAM memory type.
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
use stm32f3xx_hal_v2::delay;
use::core::arch::asm;
use cortex_m_semihosting::{debug, hprintln};
use stm32f3xx_hal_v2::{self as hal, 
                        pac,
                        prelude::*,
                        flash::ACR, 
                        pac::Peripherals,
                        pac::FLASH};

// #[link_section = ".fram_section"]
// static mut DATA_ARRAY: [u32; 5] = [0x341234, 0x3FF4, 0xCDAB, 0x12CD, 0x45EF];


#[cfg(target_arch = "arm")]
type Numeric = i32;
#[cfg(not(target_arch = "arm"))]
type Numeric = i16;

#[derive(Debug)]
pub struct Tensor2D<const H: usize, const W: usize> {
    tensor: [[Numeric; W]; H],
}

impl<const H: usize, const W: usize> Tensor2D<H, W> {
    pub const fn new(tensor: [[Numeric; W]; H]) -> Self {
        Self { tensor }
    }

    #[inline(always)]
    pub fn at(&self, rol: usize, col: usize) -> &Numeric {
        hprintln!("{:p}", &self.tensor[rol][col]);
        &self.tensor[rol][col]
    }

    #[inline(always)]
    pub fn mut_at(&mut self, rol: usize, col: usize) -> &mut Numeric {
        hprintln!("{:p}", &self.tensor[rol][col]);
        &mut self.tensor[rol][col]
    }
}

pub struct Tensor1D<const W: usize> {
    tensor: [Numeric; W],
}

#[link_section=".fram_section"]
static mut PARAM_1: Tensor2D<10, 50> = Tensor2D::new([
    [
        7, 0, 2, 5, 4, 4, 5, 7, 9, 2, 9, 4, 9, 3, 0, 8, 4, 0, 2, 9, 3, 8, 1, 6, 6, 6, 5, 3, 3, 2,
        4, 0, 6, 9, 3, 7, 6, 3, 4, 9, 2, 5, 0, 5, 7, 3, 5, 8, 7, 5,
    ],
    [
        8, 0, 6, 0, 3, 6, 0, 6, 0, 0, 6, 3, 3, 0, 0, 0, 5, 4, 5, 9, 8, 4, 5, 8, 8, 5, 5, 9, 1, 7,
        0, 3, 8, 8, 5, 9, 5, 5, 2, 4, 2, 7, 1, 7, 2, 5, 0, 7, 6, 8,
    ],
    [
        2, 0, 6, 9, 4, 9, 8, 7, 0, 6, 4, 8, 1, 5, 5, 3, 6, 8, 4, 8, 8, 4, 7, 8, 4, 2, 4, 8, 0, 7,
        0, 7, 5, 3, 9, 7, 1, 6, 2, 1, 5, 8, 5, 9, 1, 8, 7, 5, 8, 9,
    ],
    [
        9, 1, 9, 7, 4, 1, 8, 3, 2, 5, 3, 9, 2, 8, 3, 1, 8, 8, 1, 4, 1, 3, 2, 4, 0, 5, 9, 5, 3, 9,
        2, 9, 1, 9, 5, 0, 2, 7, 0, 7, 3, 9, 1, 4, 6, 0, 2, 4, 6, 7,
    ],
    [
        4, 9, 0, 4, 7, 8, 3, 4, 4, 2, 2, 0, 5, 7, 0, 2, 7, 2, 3, 5, 0, 3, 2, 0, 3, 0, 4, 8, 1, 9,
        8, 2, 4, 5, 3, 1, 8, 0, 7, 1, 8, 1, 9, 1, 6, 8, 9, 3, 8, 5,
    ],
    [
        4, 4, 0, 3, 5, 7, 1, 9, 2, 2, 6, 6, 5, 0, 6, 5, 0, 3, 0, 9, 2, 6, 0, 0, 6, 6, 2, 5, 4, 8,
        7, 9, 4, 5, 6, 4, 8, 9, 3, 6, 3, 4, 3, 4, 4, 4, 6, 8, 6, 1,
    ],
    [
        5, 7, 8, 4, 6, 2, 0, 7, 9, 1, 3, 6, 0, 6, 8, 3, 4, 8, 9, 1, 9, 0, 3, 4, 6, 6, 7, 4, 5, 1,
        6, 0, 9, 9, 8, 6, 5, 5, 4, 8, 6, 4, 5, 9, 6, 7, 9, 8, 7, 8,
    ],
    [
        5, 0, 8, 2, 6, 3, 0, 1, 9, 9, 4, 9, 6, 0, 6, 6, 5, 8, 3, 4, 5, 5, 7, 9, 0, 8, 2, 8, 9, 4,
        0, 1, 7, 6, 7, 8, 8, 7, 7, 9, 1, 4, 9, 7, 2, 9, 0, 7, 8, 7,
    ],
    [
        3, 0, 0, 1, 0, 4, 7, 2, 9, 5, 6, 8, 6, 4, 3, 6, 2, 1, 5, 4, 5, 1, 4, 8, 6, 3, 5, 8, 0, 8,
        0, 3, 0, 1, 9, 0, 9, 8, 0, 9, 0, 5, 2, 8, 1, 6, 1, 9, 5, 9,
    ],
    [
        3, 7, 8, 5, 9, 8, 7, 4, 6, 9, 9, 1, 4, 1, 6, 2, 3, 4, 8, 9, 8, 0, 5, 6, 5, 3, 8, 2, 1, 4,
        3, 1, 6, 9, 5, 9, 1, 1, 9, 3, 0, 9, 6, 3, 3, 0, 8, 5, 6, 6,
    ],
]);

#[link_section=".fram_section"]
static PARAM_2: Tensor2D<2, 10> = Tensor2D::new([
    [ 0xFFFFFFFu32 as i32, 0xFFFDFFF, 0xABCD, 0x4567, 9, 4, 9, 0, 1, 4],
    [2, 9, 2, 3, 2, 2, 8, 0, 8, 4],
]);

fn initialization(){
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
 
    //   dp.RCC.ahbenr.modify(|_, w| w.iopden().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopeen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopfen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopgen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iophen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.sramen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.flitfen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.fmcen().set_bit());  

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

  let mut gpiod = dp.GPIOD;
  let mut gpioe = dp.GPIOE;
  let mut gpiof = dp.GPIOF;
  let mut gpiog = dp.GPIOG;
  let mut gpioh = dp.GPIOH;

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

    //PF15   ------> FMC_A9
    gpiof.moder.modify(|_, w| {w.moder15().alternate()});
    gpiof.afrh.modify(|_, w| {  w.afrh15().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

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
    gpioe.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());

    //PE14   ------> FMC_D11
    gpioe.moder.modify(|_, w| {w.moder14().alternate()});
    gpioe.afrh.modify(|_, w| {  w.afrh14().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

    //PE15   ------> FMC_D12
    gpioe.moder.modify(|_, w| {w.moder15().alternate()});
    gpioe.afrh.modify(|_, w| {  w.afrh15().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

    //PD8   ------> FMC_D13
    gpiod.moder.modify(|_, w| {w.moder8().alternate()});
    gpiod.afrh.modify(|_, w| {  w.afrh8().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());

    //PD9   ------> FMC_D14
    gpiod.moder.modify(|_, w| {w.moder9().alternate()});
    gpiod.afrh.modify(|_, w| {  w.afrh9().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());

    //PD10   ------> FMC_D15
    gpiod.moder.modify(|_, w| {w.moder10().alternate()});
    gpiod.afrh.modify(|_, w| {  w.afrh10().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());


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


   
     // Configure FMC for SRAM memory(in our case F-RAM)
       unsafe{
           dp.FMC.bcr1.modify(|_, w| {
           w.mbken().set_bit(); // Enable FRAM bank 1
           w.mtyp().bits(0b00); // FRAM memory type
           w.mwid().bits(0b01); // 16-bit width
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
           w.addset().bits(0x1);
           // Set data setup time to 5 cycle
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
}

#[entry]
fn main() -> ! {

    initialization();

    // Use the `at` method to access the last element (9th row, 49th column)
    // let last_element = PARAM_1.at(9, 49);

    // // Get the raw pointer to the last element
    // let ptr: *const Numeric = last_element;

    // // Print the address of the last element
    // hprintln!("The address of the last element is: {:p}", ptr);



    hprintln!("test test ...").unwrap();

    //hprintln!("{:p}", &PARAM_1).unwrap();

    unsafe {
        *PARAM_1.mut_at(0, 0) = 32345678 as i32;
        hprintln!("{:?}", PARAM_2);
    }

    loop {
        // your code goes here
    }
}


fn delay(duration: u32) {
    for _ in 0..duration {
        // Perform some NOP operation or just loop
        asm::nop(); // Assembly NOP instruction
    }
}