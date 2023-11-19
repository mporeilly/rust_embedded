#![no_main]
#![no_std]

use panic_halt as _; // required as if the controller faces a error needs way to call panic; still not sure how that works as the "_" means it is unused so something to look into

use cortex_m_rt::entry; // since there is no "main" function need to specify the entry point to the program
use cortex_m_semihosting::hprintln; // allows communication to host; picked up by openOCD so not everything is in the gdb
use stm32f3_discovery::{stm32f3xx_hal::{self as hal, pac, prelude::*}, switch_hal::{IntoSwitch, OutputSwitch, ToggleableOutputSwitch}}; // need to access the hardware access layer as not advanced enough to start using unsafe to use registers

//const MAGNETOMETER: u8 = 0b0011_1100;       //using the WRONG address for the magnetometer has the read error out in openOCD
const MAGNETOMETER: u8 = 0b0001_1110; // Peripheral/slave address for LSM303AGR chip (confirmed when running that this is the correct based on the WHO_AM_I_REG RESPONSE)
const WHO_AM_I_REG: u8 = 0x4f; // Device id address

// Addresses of the magnetometer's registers page 45 of the LSM303AGR chip.pdf
//const 

#[entry]
fn main() -> ! {
    // function does not return
    hprintln!("Hello, world!").unwrap(); // unwrap if there is an error then need it to crash as something is very wrong

    // rust book https://docs.rust-embedded.org/discovery/f3discovery/14-i2c/index.html
    // from the example I2C code found https://github.com/stm32-rs/stm32f3xx-hal/blob/v0.7.0/examples/i2c_scanner.rs

    /* START OF THE  */
    let dp = pac::Peripherals::take().unwrap(); // need to take control of the peripherals as
    let mut flash = dp.FLASH.constrain(); // something to do with setting up the flash speed to be consistent so the data can be transfered without error
    let mut rcc = dp.RCC.constrain();       // RCC is the Reset and clock control - binds the RCC so it can work with the other peripherals
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    

    // Configure I2C1
    let mut scl =
        gpiob
            .pb6
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut sda =
        gpiob
            .pb7
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    scl.internal_pull_up(&mut gpiob.pupdr, true); // might not need this as there might be internal resistors that do this for us
    sda.internal_pull_up(&mut gpiob.pupdr, true); // might not need this as there might be internal resistors that do this for us
    let mut i2c1 = hal::i2c::I2c::new(  // creating 
        dp.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(), // setting the communication speed for the I2C bus? Example code has this included likely can increase
        clocks,
        &mut rcc.apb1,
    );
    /* END OF I2C SETUP - PULLED FROM THE DOCUMENTATION LINKED ABOVE */


    // referencing the rust learning book

    /*  need to ensure the correct registers are on to activate power to the GPIO
        "Reset and Clock Control (RCC) peripheral can be used to power on or off every other peripheral. 
            The registers that control the power status of other peripherals are:

                AHBENR
                APB1ENR
                APB2ENR"
                            - learning book
        page 123 of full manual has the RCC section
        page 166 of full manual has the RCC register map
        but I found the relation using the kit maunal on page 13 - "Figure 6 STM32F303VCT6 block diagram" 
         */


        // enable the GPIOE peripheral
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);   // I added for LEDs as they are on GPIOE and turned on by AHBENR (ahb) register  
        let mut led13 = gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).into_active_high_switch();
        // makes the pe13 pin (LED) into an output 
        // then need to "Section 11.4.12 - GPIO registers - Page 243 - Reference Manual"
        led13.off().unwrap();

        //let mut clock = dp.TIM1.


        // now time for the timer(s)
        /*
        https://docs.rust-embedded.org/discovery/f3discovery/09-clocks-and-timers/one-shot-timer.html
        https://dev.to/apollolabsbin/stm32f4-embedded-rust-at-the-hal-timer-interrupts-154e
         */
        




    //doing math
    let a = 32f32;
    let b = 4f32;
    let c = b / a;
    

    loop {
        let mut buffer = [0u8; 1];
        /* originally had the buffer as [0u8, 1] but this creates a buffer (array) of two values not a single
        the result was additional data coming back openOCD was "[64, 117]"
        need to have the "," replaced with the ";" which now has the proper response of [64]*/

        match i2c1.write_read(MAGNETOMETER, &[WHO_AM_I_REG], &mut buffer) {
            // Writes bytes to slave with address and then reads enough bytes to fill buffer in a single transaction
            Ok(_) => hprintln!("0x{:02x} - 0b{:?}", WHO_AM_I_REG, buffer).unwrap(),
            Err(_) => hprintln!("Error reading").unwrap(),
        }
        led13.toggle().unwrap();
        hprintln!("answer {:?}", c).unwrap();
    }
}
