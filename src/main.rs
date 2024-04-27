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

fn write_to_fram(){
    
}
#[entry]
fn main() -> ! {

   let dp  = Peripherals::take().unwrap();
   let mut rcc = dp.RCC.constrain();
   let mut flash = dp.FLASH.constrain();
   let clocks = rcc.cfgr.freeze(&mut flash.acr);

   let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
   let mut we = gpiob.pb0.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
   let mut oe = gpiob.pb1.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
   let mut cs = gpiob.pb2.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);


    we.set_low().unwrap();
    oe.set_low().unwrap();
    cs.set_low().unwrap();

    // write to F-RAM
    // 1. Enable F-RAM (cs to low)
    // 2. Assert Write Enable (WE)

    unsafe {
        core::ptr::write_volatile(0x60000000 as *mut u8, 2); // Perform write operation
    }
    //4. Deassert Write Enable (WE)
    oe.set_high().unwrap();
   // 5. Disable F-RAM
    loop {
        // your code goes here
    }
}
