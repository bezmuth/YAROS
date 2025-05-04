use x86_64::instructions::port::Port;

#[repr(u8)]
#[allow(dead_code)]
enum Register {
    Seconds = 0x00,
    Minutes = 0x02,
    Hours = 0x04,
    Weekday = 0x06,
    DOM = 0x07,
    Month = 0x08,
    Year = 0x09,
    Century = 0x32, // not always supported
    StatusA = 0x0A,
    StatusB = 0x0B,
}

pub struct Time {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
}

fn get_update_in_progress_flag() -> bool {
    let status = get_register(Register::StatusA);
    return ((status >> 7) & 0b1) == 1;
}
fn get_register(register: Register) -> u8 {
    let mut cmos_register: Port<u8> = Port::new(0x70);
    let mut cmos_data: Port<u8> = Port::new(0x71);

    // nmi disable bit is set
    unsafe { cmos_register.write((1 << 7) | register as u8) };
    return unsafe { cmos_data.read() };
}

fn set_register(register: Register, val: u8) {
    let mut cmos_register: Port<u8> = Port::new(0x70);
    let mut cmos_data: Port<u8> = Port::new(0x71);
    unsafe {
        cmos_register.write((1 << 7) | register as u8);
        cmos_data.write(val);
    };
}

fn init() {
    // set registers into 24 hour mode and binary. This is not always possible
    // but we will presume it is
    set_register(Register::StatusB, 0b0110);
}

pub fn get_time() -> Time {
    init();
    // block until update is finished
    while get_update_in_progress_flag() {}
    return Time {
        seconds: get_register(Register::Seconds),
        minutes: get_register(Register::Minutes),
        hours: get_register(Register::Hours),
    };
}
