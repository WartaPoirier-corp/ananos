/// CMOS is mostly used to get datetime information
///
/// https://wiki.osdev.org/CMOS

pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub fn get_datetime() -> DateTime {
    let format = format_info();
    let century = convert(format.1, read_cmos_register(0x32)) as u16;
    let year = convert(format.1, read_cmos_register(0x09)) as u16;
    let year = (century * 100) + year;
    let month = convert(format.1, read_cmos_register(0x08));
    let day = convert(format.1, read_cmos_register(0x07));
    let hours = convert(
        format.1,
        if !format.0 {
            convert_to_24h(read_cmos_register(0x04))
        } else {
            read_cmos_register(0x04)
        }
    );
    let minutes = convert(format.1, read_cmos_register(0x02));
    let seconds = convert(format.1, read_cmos_register(0x00));
    DateTime {
        year,
        month,
        day,
        hours,
        minutes,
        seconds,
    }
}

fn convert_to_24h(value: u8) -> u8 {
    let is_pm = (value & 0x80) != 0;
    let mut value = value & 0x78;
    
    if is_pm {
        value = value + 12;
    }

    if value == 24 {
        value = 0;
    }

    value
}

fn convert(is_binary: bool, value: u8) -> u8 {
    if !is_binary {
        ((value & 0xf0) >> 1) + ((value & 0xf0) >> 3) + (value & 0xf)
    } else {
        value
    }
}

fn format_info() -> (bool, bool) {
    let status_reg_b = read_cmos_register(0x0b);
    let is_24h_format = ((status_reg_b >> 1) & 0x1) == 0x1;
    let is_binary_format = ((status_reg_b >> 2) & 0x1) == 0x1;
    (is_24h_format, is_binary_format)
}

fn read_cmos_register(reg: u8) -> u8 {
    x86_64::instructions::interrupts::disable();

    let mut port70 = x86_64::instructions::port::Port::new(0x70);
    let mut port71 = x86_64::instructions::port::Port::new(0x71);

    let result = unsafe {
        port70.write(reg);
        port71.read()
    };
    
    x86_64::instructions::interrupts::enable();
    result
}
