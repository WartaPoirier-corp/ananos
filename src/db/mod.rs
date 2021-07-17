use alloc::vec::Vec;
use adb::{Db, DbObject, DbValue};
use crate::{print, println};

pub static DB: spin::Mutex<Option<Db<Vec<u8>>>> = spin::Mutex::new(None);

pub fn init() {
    let mut db = DB.lock();
    *db = Some(
        Db::read_from(Vec::from(*include_bytes!("../../test.adb"))).unwrap()
    );
}

pub fn display_contents(db: &mut Db<Vec<u8>>) {
    for ty in db.all_type_ids() {
        println!("TYPE {}", ty.0);
        for item in db.iter_type(ty) {
            show_db_object(&item, 0);
        }
    }
}

fn show_db_object(obj: &DbObject, padding: usize) {
    for _ in 0..padding {
        print!("    ");
    }

    match *obj.value {
        DbValue::Unit => println!("()"),
        DbValue::U64(x) => println!("{}", x),
        DbValue::F64(x) => println!("{}", x),
        DbValue::Array(ref arr) => {
            println!("[");
            for item in arr {
                show_db_object(item, padding + 1)
            }
            for _ in 0..padding {
                print!("    ");
            }
            println!("]")
        },
        DbValue::Sum { ref variant, ref data } => {
            print!("{} : ", variant);
            show_db_object(data, padding + 1);
        },
        DbValue::Product { ref fields } => {
            println!("{{");
            for f in fields {
                show_db_object(f, padding + 1);
            }
            for _ in 0..padding {
                print!("    ");
            }
            println!("}}");
        },
    }
}

