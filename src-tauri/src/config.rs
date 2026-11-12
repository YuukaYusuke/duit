use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db(pub Mutex<Connection>);
pub const CLASSES: [&str; 5] = ["An Naas", "Al Ikhlas", "Al Kautsar", "An Nahl", "An-Nisa"];
pub const TYPES: [&str; 7] = [
    "SPP",
    "Pengembangan",
    "Buku",
    "Foto",
    "UIM",
    "Infaq",
    "Kegiatan",
];
