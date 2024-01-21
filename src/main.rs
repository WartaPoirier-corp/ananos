#![feature(abi_efiapi)]
#![no_std]
#![no_main]

#[macro_use] extern crate alloc;

mod apps;
mod framework;
mod vm;

use log::info;
use uefi::prelude::*;
use uefi::proto::console::text::Key;

const FONT: &[u8] = include_bytes!("../assets/InriaSerif-Regular.ttf");
const COMIC_SANS: &[u8] = include_bytes!("../assets/ComicSans.ttf");

macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", $crate::file!(), $crate::line!());
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                log::info!("[{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[entry]
fn efi_main(_handle: uefi::Handle, system_table: SystemTable<Boot>) -> Status {
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

    render_text(FONT, &mut fb, stride, "Bienvenue dans ananOS !", 32.0, 800, 100);
    render_text(FONT, &mut fb, stride, "Un super système d'exploitation vraiment très très bien.", 24.0, 800, 140);
    render_text(COMIC_SANS, &mut fb, stride, "Écrit en Rust (et en Comic Sans MS)!!!", 24.0, 800, 180);

    let mut ctx = vm::Context::new();
    framework::setup(&mut ctx);
    let calc = apps::calc();
    let res = ctx.run(&calc);
    render_text(FONT, &mut fb, stride, &format!("{} = {}", calc.to_string(), res.unwrap().to_string()), 24.0, 800, 300);

    let mut events = vec![
        system_table.stdin().wait_for_key_event(),
    ];
    let mut pos_x = 800;
    let mut pos_y = 350;
    loop {
        system_table.boot_services().wait_for_event(&mut events).expect_success("event");
        match system_table.stdin().read_key().expect_success("stdin") {
            Some(Key::Printable(c)) => {
                render_text(FONT, &mut fb, stride, &format!("{}", c), 32.0, pos_x, pos_y);
                pos_x += 25;
                if pos_x > 1500 {
                    pos_x = 800;
                    pos_y += 30;
                }
            },
            _ => {},
        }
    }
    // Status::SUCCESS
}

fn render_text<'a>(font_data: &[u8], fb: &mut uefi::proto::console::gop::FrameBuffer, stride: usize, text: &'a str, size: f32, mut x: i32, y: i32) {
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();
    let fonts = &[font];
    let mut layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
    layout.append(fonts, &fontdue::layout::TextStyle::new(text, size, 0));
    for glyph in layout.glyphs() {
        let (metrics, bitmap) = fonts[0].rasterize_config(glyph.key);
        for i in 0..metrics.height {
            let complement = size as i32 - metrics.height as i32;
            for j in 0..metrics.width {
                let col = (x + metrics.xmin + j as i32) as usize;
                let row = (y + complement - metrics.ymin + i as i32) as usize;
                let pixel_index = (row * stride) + col;
                let color = 255 - bitmap[j + i * metrics.width];
                unsafe { fb.write_value(pixel_index * 4, [color, color, color]); }
            }
        }
        x += metrics.advance_width as i32;
    }
}
