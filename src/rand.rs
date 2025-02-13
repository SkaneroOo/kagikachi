
use std::{
    fs::File,
    io::Read,
    sync::Mutex
};

pub struct Rand {
    fd: Mutex<File>
}

impl Rand {
    pub fn new() -> Self {
        assert!(std::path::Path::new("/dev/urandom").exists(), "Sorry, current implementation doesn't work on this system.");
        Self {
            fd: Mutex::new(File::open("/dev/urandom").unwrap())
        }
    }

    pub fn get_mask(&self) -> [u8; 4] {
        let mut buff = [0; 4];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        buff
    }

    #[allow(unused)]
    pub fn next_u8(&self) -> u8 {
        let mut buff = [0; 1];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        buff[0]
    }

    #[allow(unused)]
    pub fn next_u16(&self) -> u16 {
        let mut buff = [0; 2];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        u16::from_be_bytes(buff)
    }

    #[allow(unused)]
    pub fn next_u32(&self) -> u32 {
        let mut buff = [0; 4];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        u32::from_be_bytes(buff)
    }

    #[allow(unused)]
    pub fn next_u64(&self) -> u64 {
        let mut buff = [0; 8];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        u64::from_be_bytes(buff)
    }

    #[allow(unused)]
    pub fn next_usize(&self) -> usize {
        let mut buff = [0; (usize::BITS / 8) as usize];
        self.fd.lock().unwrap().read(&mut buff).unwrap();
        usize::from_be_bytes(buff)
    }
}