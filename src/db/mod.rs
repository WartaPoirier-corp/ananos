use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

pub static DB: spin::Mutex<Option<Db>> = spin::Mutex::new(None);

pub fn init() {
    let mut db = DB.lock();
    *db = Some(Db {
        locations: {
            let mem = Arc::new(MemoryLocation::new(LocationId(0)));
            let mut map: BTreeMap<_, Arc<dyn Location + Send + Sync>> = BTreeMap::new();
            map.insert(LocationId(0), mem);
            map
        },
        handle_map: BTreeMap::new(),
    });
}

pub struct Db {
    locations: BTreeMap<LocationId, Arc<dyn Location + Send + Sync>>,
    handle_map: BTreeMap<StreamHandle, LocationId>,
}

static HANDLES: AtomicU64 = AtomicU64::new(0);

impl Db {
    pub fn open(&mut self, ty: Type, loc: LocationId) -> StreamHandle {
        if let Some(ref mut location) = self.locations.get_mut(&loc) {
            let handle = StreamHandle(HANDLES.fetch_add(1, Ordering::SeqCst));
            self.handle_map.insert(handle, loc);
            location.open(ty, handle);
            handle
        } else {
            panic!("Invalid location ID");
        }
    }

    pub fn read(&self, handle: StreamHandle) -> &[u8] {
        if let Some(loc) = self.handle_map.get(&handle) {
            if let Some(loc) = self.locations.get(&loc) {
                loc.read(handle)
            } else {
                panic!("Invalid location ID in the stream handle map");
            }
        } else {
            panic!("Invalid stream handle");
        }
    }
    pub fn add_location() {}
    pub fn remove_location() {}
    pub fn find_memory_location(&self) -> LocationId {
        LocationId(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocationId(u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StreamHandle(u64);
pub enum Type {
    U8,
    Type,
}

impl Type {
    fn id(&self) -> u64 {
        match *self {
            Type::U8 => 0,
            Type::Type => 1,
        }
    }

    fn size(&self) -> Option<usize> {
        match *self {
            Type::U8 => Some(1),
            Type::Type => Some(1),
        }
    }

    pub fn byte_type() -> Type {
        Type::U8
    }
}

trait Location {
    fn new(id: LocationId) -> Self where Self: Sized;

    fn get_id(&self) -> LocationId;

    fn open(&self, ty: Type, handle: StreamHandle);

    fn read(&self, handle: StreamHandle) -> &[u8];
}

struct MemoryLocation {
    id: LocationId,
    type_index: Vec<u64>,
    data: Vec<u8>,
    handles: spin::Mutex<BTreeMap<StreamHandle, StreamStatus>>,
}

const MEMORY_DB_SIZE: usize = 1024;
const MEMORY_DB_BLOCK_SIZE: usize= 512;

impl Location for MemoryLocation {
    fn new(id: LocationId) -> Self {
        MemoryLocation {
            // This type index contains a first block storing types, and then only u8 blocks
            type_index: {
                let block_count = MEMORY_DB_SIZE / MEMORY_DB_BLOCK_SIZE;
                let mut idx = Vec::with_capacity(block_count);
                idx.push(1);
                for _ in 1..block_count {
                    idx.push(0);
                }                

                idx
            },
            data: {
                let mut vec = Vec::with_capacity(MEMORY_DB_SIZE);
                vec.push(42);
                vec
            },
            handles: spin::Mutex::new(BTreeMap::new()),
            id
        }
    }

    fn get_id(&self) -> LocationId {
        self.id
    }

    fn open(&self, ty: Type, handle: StreamHandle) {
        let ty_id = ty.id();
        let status = StreamStatus {
            ty_size: ty.size(),
            offset: 0,
            blocks: self.type_index.iter().filter_map(|x| if *x == ty_id { Some(*x) } else { None }).collect(),
        };
        self.handles.lock().insert(handle, status);   
    }

    fn read(&self, handle: StreamHandle) -> &[u8] {
        let mut handles = self.handles.lock();
        let mut status = handles.get_mut(&handle).unwrap();
        if let Some(size) = status.ty_size {
            let start_block_number = status.offset / MEMORY_DB_BLOCK_SIZE;
            let start_block = status.blocks[start_block_number] as usize;
            let start = (start_block * MEMORY_DB_BLOCK_SIZE) + (status.offset % MEMORY_DB_BLOCK_SIZE);
            let start = start as usize;
            let end = start + size;

            status.offset += size;

            &self.data[start..end]
        } else {
            panic!("Unsized types are not supported yet!")
        }
    }
}

struct StreamStatus {
    ty_size: Option<usize>,
    offset: usize,
    blocks: Vec<u64>,
}
