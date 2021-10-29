use crate::{print, println};
use adb::{Db, DbValue, TypeInfo};
use alloc::sync::Arc;
use alloc::vec::Vec;

pub static DB: spin::Mutex<Option<Db<Vec<u8>>>> = spin::Mutex::new(None);

fn db_logger(args: core::fmt::Arguments) {
    crate::println!("{}", args);
}

pub fn init() {
    let mut db = DB.lock();
    *db = Some({
        let mut datab = Db::read_from(Vec::from(*include_bytes!("../../test.adb"))).unwrap();
        datab.set_logger(db_logger);
        datab
    });
}

pub fn display_contents(db: &mut Db<Vec<u8>>) {
    for ty in db.all_type_ids() {
        let items: Vec<_> = db.iter_type(ty).collect();
        for item in items {
            show_db_object(&db, item.value, item.type_info, 0);
        }
    }
}

fn show_db_object(db: &Db<Vec<u8>>, value: Arc<DbValue>, ty: Arc<TypeInfo>, padding: usize) {
    for _ in 0..padding {
        print!("    ");
    }

    match *value {
        DbValue::Unit => println!("()"),
        DbValue::U8(x) => println!("{}", x),
        DbValue::U64(x) => println!("{}", x),
        DbValue::F64(x) => println!("{}", x),
        DbValue::Array(ref arr) => {
            println!("[");
            let mut str = alloc::vec::Vec::with_capacity(arr.len());
            for item in arr {
                let arr_ty = match ty.definition {
                    adb::TypeDef::Array(id) => db.get_type_info(id).unwrap(),
                    _ => unreachable!(),
                };
                if arr_ty.id == adb::type_ids::U8 {
                    str.push(match item.as_ref() {
                        adb::DbValue::U8(b) => *b,
                        _ => unreachable!(),
                    })
                } else {
                    show_db_object(db, Arc::clone(item), arr_ty, padding + 1)
                }
            }
            if !str.is_empty() {
                println!(
                    "\"{}\"",
                    alloc::string::String::from_utf8(str).unwrap_or_default()
                );
            }
            for _ in 0..padding {
                print!("    ");
            }
            println!("]")
        }
        DbValue::Sum {
            ref variant,
            ref data,
        } => {
            print!("{} : ", variant);
            let var_ty = match ty.definition {
                adb::TypeDef::Sum { ref variants } => {
                    db.get_type_info(variants[*variant as usize].1).unwrap()
                }
                _ => unreachable!(),
            };
            show_db_object(db, Arc::clone(data), var_ty, padding + 1);
        }
        DbValue::Product { ref fields } => {
            println!("{{");
            let fields_ty: Vec<_> = match ty.definition {
                adb::TypeDef::Product { ref fields } => fields.iter().map(|x| x.1).collect(),
                _ => unreachable!(),
            };
            for (f, f_ty) in fields.iter().zip(fields_ty.iter()) {
                let f_ty = db.get_type_info(*f_ty).unwrap();
                show_db_object(db, Arc::clone(f), f_ty, padding + 1);
            }
            for _ in 0..padding {
                print!("    ");
            }
            println!("}}");
        }
    }
}
