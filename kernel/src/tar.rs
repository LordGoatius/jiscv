use core::{
    ascii::Char,
};

use ralloc::{borrow::ToOwned, format, string::ToString, vec::Vec};

use crate::virtio::{read_write_disk, SECTOR_SIZE};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Type {
    File = b'0',
    Link = b'1',
    SymLink = b'2',
    CharSpec = b'3',
    BlockSpec = b'4',
    Dir = b'5',
    FIFO = b'6',
    ContigFile = b'7',
    GlobalExtHeader = b'g',
    ExtHeaderNext = b'x',
    // VendorExt    = b'A'..=b'Z',
    // Enums with ranges for discriminants would be fascinating
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Tar {
    path_name: [Char; 100],
    mode: [Char; 8],
    owner_uid: [Char; 8],
    group_uid: [Char; 8],
    size: [Char; 12],
    last_mod: [Char; 12],
    chksum: [u8; 8],
    ty: Type,
    name: [u8; 100],
    ustar: [Char; 6],
    usver: [Char; 2],
    owner_username: [Char; 32],
    owner_groupname: [Char; 32],
    dev_major_num: [u8; 8],
    dev_minor_num: [u8; 8],
    filename_prefix: [u8; 155],
    _pad: [u8; 12],
}

#[derive(Copy, Clone, Debug)]
pub struct File {
    pub name: [Char; 100],
    in_use: bool,
    data: [u8; 1024],
    size: usize,
    offset: usize,
}

pub fn init_fs_tar() -> Vec<File> {
    const FILE_MAX: usize = 4;
    const DISK_REQ_SPACE: usize = (size_of::<File>() * FILE_MAX) + size_of::<Tar>();
    const DISK_SIZE: usize = DISK_REQ_SPACE + (512 - (DISK_REQ_SPACE % 512));

    let mut files = Vec::new();

    let mut buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
    let sector = &mut buf[0..SECTOR_SIZE];
    read_write_disk(sector.as_mut_ptr(), 0, false);

    let mut traversed_sectors = 1;
    let mut tar = unsafe { *(buf.as_mut_ptr() as *mut Tar) };

    while tar.ustar[0..5].as_str() == "ustar" {
        let size = usize::from_ascii_radix(&tar.size.as_bytes().trim_suffix(&[0]), 8).unwrap();
        let sectors = size.div_ceil(SECTOR_SIZE);

        if size > 1024 {
            panic!("tar-fs: Files larger than 1024 bytes are not currently supported");
        }
        let name = tar.path_name;
        let mut data: [u8; 1024] = [0; 1024];
        let offset = traversed_sectors - 1;

        for i in 0..sectors {
            let sect = &mut data[(i * SECTOR_SIZE)..((i + 1) * SECTOR_SIZE)];
            read_write_disk(sect.as_mut_ptr(), traversed_sectors, false);
            traversed_sectors += 1;
        }

        files.push(File {
            in_use: false,
            name,
            data,
            size,
            offset,
        });

        read_write_disk(buf.as_mut_ptr(), traversed_sectors, false);
        traversed_sectors += 1;
        tar = unsafe { *(buf.as_mut_ptr() as *mut Tar) };
    }

    files
}

pub fn tar_fs_flush(files: &mut [File], index: usize) {
    let mut buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
    let mut file = files[index];

    read_write_disk(buf.as_mut_ptr(), file.offset, false);

    let mut tar = unsafe { *(buf.as_mut_ptr() as *mut Tar) };
    tar.path_name = file.name;
    let size = usize::from_ascii_radix(&tar.size.as_bytes().trim_suffix(&[0]), 8).unwrap();
    let sectors = size.div_ceil(SECTOR_SIZE);
    let oct = format!("{:011o}", file.size);
    let oct = unsafe { oct.as_bytes().as_ascii_unchecked() };
    tar.size[0..11].copy_from_slice(&oct[0..11]);

    for i in 1..=sectors {
        let data = &mut file.data[((i-1) * SECTOR_SIZE)..(i * SECTOR_SIZE)];
        read_write_disk(data.as_mut_ptr(), file.offset + i, true);
    }

    read_write_disk(&raw mut tar as *mut u8, file.offset, true);
}
