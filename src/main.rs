#![no_main]
#![no_std]
use cortex_m_rt::entry; // since there is no "main" function need to specify the entry point to the program
// A. START Part 1 of 2 ----------------------------------------------- //
// A. taken from https://github.com/rust-embedded/embedded-alloc/blob/master/examples/global_alloc.rs
extern crate alloc;// added so vec can be used

use alloc::vec::Vec;    // added so vec can be used

// Linked-List First Fit Heap allocator (feature = "llff")
//use embedded_alloc::LlffHeap as Heap;       // <--------------------------- Line Errors out... "cargo add embedded_alloc"? Yes but of course it is named" "embedded-alloc" not the underscored one
// Two-Level Segregated Fit Heap allocator (feature = "tlsf")
// use embedded_alloc::TlsfHeap as Heap;
// looking at the documentation I found that there is a struct created called Heap hence the line below, this did catch that the vec's were not tagged as mutable so it gave me hope but...
use embedded_alloc::Heap as Heap;   // causes a massive error ----> "error: linking with `rust-lld` failed: exit status: 1"
// from the libs.rs within embedded_alloc: 
/*      ...
        #[cfg(feature = "llff")]
        pub use llff::Heap as LlffHeap;
        ...
*/
// but the language server seems to not like "LlffHeap" or "TlsfHeap" as the only one to allow it to see past to the missing "mut" was just "...::Heap"
// at this point I can't even get the example (https://github.com/rust-embedded/embedded-alloc/blob/master/examples/global_alloc.rs) to run in a clean directory
// two possible problems I think right now is it the naming of the heap structs (unlikley) that is causing this issue or is it related to the "config.toml" and the "memory.x" (most likley based on the errors I'm getting)
// tried verifying the "memory.x" file is correctly written but the discovery example doesn't seem to have it 
// in reviewing "https://github.com/jhford/rust-stm32f303-binary-counter/blob/master/memory.x" I might need to dig further into the memory layout
// pasted the error into "error_likely_memory_related"
//
// error "rust-lld: warning: address (0x8012378) of section .rodata is not a multiple of alignment (16)"
//
// googling I found "https://github.com/rust-embedded/cortex-m-rt/issues/26" however this is a closed issue on an archived repo but it looks to confirm memory issue
//
// this (https://docs.rust-embedded.org/cortex-m-quickstart/cortex_m_rt/index.html) breaks down how everything is organized


#[global_allocator]
static HEAP: Heap = Heap::empty();

// A. END Part 1 of 2 ---------------------------------------------- //

use panic_halt as _; // required as if the controller faces a error needs way to call panic; still not sure how that works as the "_" means it is unused so something to look into

//use cortex_m_rt::entry; // since there is no "main" function need to specify the entry point to the program
use cortex_m_semihosting::hprintln; // allows communication to host; picked up by openOCD so not everything is in the gdb
use stm32f3_discovery::{stm32f3xx_hal::{self as hal, pac, prelude::*}, switch_hal::{IntoSwitch, OutputSwitch, ToggleableOutputSwitch}}; // need to access the hardware access layer as not advanced enough to start using unsafe to use registers

//const MAGNETOMETER: u8 = 0b0011_1100;       //using the WRONG address for the magnetometer has the read error out in openOCD
const MAGNETOMETER: u8 = 0b0001_1110; // Peripheral/slave address for LSM303AGR chip (confirmed when running that this is the correct based on the WHO_AM_I_REG RESPONSE)
const WHO_AM_I_REG: u8 = 0x4f; // Device id address

// Addresses of the magnetometer's registers page 45 of the LSM303AGR chip.pdf
//const 



#[entry] // need to mark the entry
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

    // embed file
    let filecontents = include_bytes!("../aocday01t1"); // what is the increase in the binary???
    hprintln!("{:?}",filecontents).unwrap();


    // A. START Part 2 of 2 ----------------------------------------------- //

    // initializing the allocator
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    // A. END Part 2 of 2 ----------------------------------------------- //
    
    //vector for each lines values (requires allocator hence the addition above)
    let mut sum_vector = Vec::new();
    let mut result_vector = Vec::new();

    // want to pull the numbers out
    for i in 0..filecontents.len(){
        if filecontents[i] >= 48 && filecontents[i] <= 57{
            hprintln!("found number").unwrap();
            sum_vector.push(filecontents[i]);

        } // need to filter based on ascii number cutoff 48 is "0" and 57 is "9"
        if filecontents[i] == b'\n'{
            if sum_vector.len() < 3{
                result_vector.push(sum_vector.iter().sum());
            }else {
                result_vector.push(sum_vector[0]+sum_vector[sum_vector.len()]);
            }
            sum_vector.clear();

        }
    }
    let finalresult: u8 = result_vector.iter().sum();
    hprintln!("Value of the input is: {:?}", finalresult).unwrap();
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
