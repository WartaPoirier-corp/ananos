#![feature(abi_efiapi)]
#![no_std]
#![no_main]

use log::info;
use uefi::prelude::*;

#[entry]
fn efi_main(handle: uefi::Handle, system_table: SystemTable<Boot>) -> Status {
    use uefi::proto::console::gop::*;
    use uefi::prelude::ResultExt;
    uefi_services::init(&system_table).expect_success("Failed to initialize utils");

    // reset console before doing anything else
    system_table
        .stdout()
        .reset(false)
        .expect_success("Failed to reset output buffer");

    // Print out UEFI revision number
    {
        let rev = system_table.uefi_revision();
        let (major, minor) = (rev.major(), rev.minor());

        info!("UEFI {}.{}", major, minor);
    }

    let gop = unsafe { &mut *system_table.boot_services()
        .locate_protocol::<GraphicsOutput>()
        .expect_success("gop")
        .get()
    };

    let mut mode: Option<Mode> = None;
    for out in gop.modes() {
        let out = out.unwrap();
        let info = out.info();
        if let Some(ref m) = mode {
            if info.resolution().0 > m.info().resolution().0 {
                mode = Some(out);
            }
        } else {
            mode = Some(out);
        }
    }
    let mode = mode.unwrap();
    gop.set_mode(&mode).expect_success("cant set mode");
    let info = mode.info();
    let res = info.resolution();

    info!("res: {:?}, format: {:?}", res, info.pixel_format());
    let mut fb = gop.frame_buffer();
    let stride = info.stride();
    let white = [255_u8, 255, 255];
    let gray = [228_u8, 228, 228];
    for row in 0..(res.1 - 1) {
        for col in 0..(res.0 - 1) {
            let pixel_index = (row * stride) + col;
            unsafe { fb.write_value(pixel_index * 4, if (col % 50 == 0) || (row % 50 == 0) { gray } else { white }); }
        }
    }

    info!("Bienvenue dans ananOS !");

    loop {}
    Status::SUCCESS
}
