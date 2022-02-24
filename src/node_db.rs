use byte_slice_cast::AsMutSliceOf;
use byte_slice_cast::AsSliceOf;
use memmap::MmapMut;
use std::fs::OpenOptions;
use std::ops::Deref;
use std::path::Path;

pub struct NodeDB {
    mmap: MmapMut,
}

pub fn new(path: &Path, nodes: u64) -> NodeDB {
    let size = (nodes + 1) * 8 * 2;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    file.set_len(size).unwrap();
    unsafe {
        NodeDB {
            mmap: MmapMut::map_mut(&file).unwrap(),
        }
    }
}

pub fn _open(path: &Path) -> NodeDB {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(path)
        .unwrap();
    unsafe {
        NodeDB {
            mmap: MmapMut::map_mut(&file).unwrap(),
        }
    }
}

impl NodeDB {
    pub fn lookup(&self, node_id: i64) -> (i64, i64) {
        let slice: &[u8] = self.mmap.deref();
        let slice: &[i64] = slice.as_slice_of::<i64>().unwrap();
        let lat = slice[(node_id * 2) as usize];
        let lon = slice[(node_id * 2 + 1) as usize];
        (lat, lon)
    }

    pub fn set(&self, node_id: i64, lat: i64, lon: i64) {
        let slice: &[u8] = self.mmap.deref();
        let len = slice.len();
        let ptr = slice.as_ptr() as *mut u8;
        let slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(ptr, len) };

        let slice: &mut [i64] = slice.as_mut_slice_of::<i64>().unwrap();
        slice[(node_id * 2) as usize] = lat;
        slice[(node_id * 2 + 1) as usize] = lon;
    }
}
