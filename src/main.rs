#![no_std]
#![no_main]

extern crate alloc;

use alloc::rc::Rc;
use alloc::string::ToString;
use core::cell::RefCell;
use noli::*;
use saba_core::browser::Browser;
use saba_core::http::HttpResponse;

fn main() -> u64 {
    let browser = Browser::new();

    let ui = Rc::new(RefCell::new(WasabiUI::new(browser)));

    match ui.borrow_mut().start() {
        Ok(_) => {}
        Err(e) => {
            println!("failed fails to start: {:?}", e);
            return 1;
        }
    }

    0
}

entry_point!(main);
