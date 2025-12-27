#![rustfmt::skip]

use core::fmt::Debug;

#[derive(Debug)]
pub struct Ext2 {
    blck_size: u32,
    frag_size: u32,
    inode_total: u32,
    block_total: u32,
    inode_per_bg: u32,
    blk_per_bg: u32,
    block_group_total: u32,
}

#[repr(C, packed(4))]
pub struct Superblock {
    inode_num:                  u32, // Total number of inodes in file system
    block_num:                  u32, // Total number of blocks in file system
    superuser_block_num:        u32,
    unallocated_blocks:         u32,
    unallocated_inodes:         u32,
    superblock_block_num:       u32,
    blck_size_shift:            u32, // the number to shift 1,024 to the left by to obtain the block size
    frag_size_shift:            u32, // the number to shift 1,024 to the left by to obtain the fragment size
    blocks_per_block_group:     u32,
    fragments_per_block_group:  u32,
    inodes_per_block_group:     u32,
    last_mount_time_posix:      u32,
    last_written_time_posix:    u32,
    mounts_since_fsck:          u16,
    mounts_allowed_before_fsck: u16,
    ext2_magic:                 u16, // (0xef53)
    fs_state:                   FsState,
    error_procedure:            ErrProc,
    minor_version:              u16,
    last_fsck_posix:            u32,
    interval_fsck_posix:        u32,
    fs_os_id:                   OsId,
    major_version:              u32,
    user_id_use_res_blocks:     u16,
    group_id_use_res_blocks:    u16,
    first_nonres_inode:         u32,
    inode_size:                 u16,
    block_group:                u16, // Block group that this superblock is part of
    opt_feat:                   u32,
    req_feat:                   u32,
    read_only_feat:             u32, // if not supported, the volume must be mounted read-only
    fsid:                       [u8; 16],
    vol_name:                   [u8; 16],
    last_mount_path:            [u8; 64],
    comp_alg:                   u32,
    prealloc_blocks:            u8,
    prealloc_dir:               u8,
    _0:                         u16,
    journal_id:                 [u8; 16],
    journal_inode:              u32,
    journal_dev:                u32,
    orphan_inode_head:          u32,
    _1:                         [u8; 788], // (Unused)
}

impl Superblock {
    pub fn get_ext2(&self) -> Ext2 {
        assert_eq!(
            self.block_num / self.blocks_per_block_group,
            self.inode_num / self.inodes_per_block_group,
            "Make sure Ext2 FS is consistent"
        );
        Ext2 {
            blck_size: 1024 >> self.blck_size_shift,
            frag_size: 1024 >> self.frag_size_shift,
            inode_total: self.inode_num,
            block_total: self.block_num,
            inode_per_bg: self.inodes_per_block_group,
            blk_per_bg: self.blocks_per_block_group,
            block_group_total: self.block_num / self.blocks_per_block_group
        }
    }
}

#[repr(C)]
struct BlockGroupDescriptorTable {
    /// Block address of block usage bitmap
    block_usage_bitmap_addr: u32,
    /// Block address of inode usage bitmap
    inode_usage_bitmap_addr: u32,
    /// Starting block address of inode table
    block_addr_inode_table: u32,
    /// Number of unallocated blocks in group
    unallocated_blocks: u16,
    /// Number of unallocated inodes in group
    unallocated_inodes: u16,
    /// Number of directories in group
    num_dirs: u16,
    _0: [u8; 13] 
}

#[repr(C)]
struct Inode {
    ty_perm: u16,                 // Type and Permissions (see below)
    user_id: u16,                 // User ID
    size_lb: u32,                 // Lower 32 bits of size in bytes
    last_access_posix: u32,       // Last Access Time (in POSIX time)
    creation_time_posix: u32,     // Creation Time (in POSIX time)
    last_mod_posix: u32,          // Last Modification time (in POSIX time)
    deletion_time_posix: u32,     // Deletion time (in POSIX time)
    group_id: u16,                // Group ID
    num_hard_links: u16,          // Count of hard links (directory entries) to this inode. When this reaches 0, the data blocks are marked as unallocated.
    num_disk_sectors: u32,        // Count of disk sectors (not Ext2 blocks) in use by this inode, not counting the actual inode structure nor directory entries linking to the inode.
    flags: u32,                   // Flags (see below)
    os_val_1: u32,                // Operating System Specific value #1
    direct_block_ptr_0: u32,      // Direct Block Pointer 0
    direct_block_ptr_1: u32,      // Direct Block Pointer 1
    direct_block_ptr_2: u32,      // Direct Block Pointer 2
    direct_block_ptr_3: u32,      // Direct Block Pointer 3
    direct_block_ptr_4: u32,      // Direct Block Pointer 4
    direct_block_ptr_5: u32,      // Direct Block Pointer 5
    direct_block_ptr_6: u32,      // Direct Block Pointer 6
    direct_block_ptr_7: u32,      // Direct Block Pointer 7
    direct_block_ptr_8: u32,      // Direct Block Pointer 8
    direct_block_ptr_9: u32,      // Direct Block Pointer 9
    direct_block_ptr_10: u32,     // Direct Block Pointer 10
    direct_block_ptr_11: u32,     // Direct Block Pointer 11
    single_indirect_blk_ptr: u32, // Singly Indirect Block Pointer (Points to a block that is a list of block pointers to data)
    doubly_indirect_blk_ptr: u32, // Doubly Indirect Block Pointer (Points to a block that is a list of block pointers to Singly Indirect Blocks)
    triply_indirect_blk_ptr: u32, // Triply Indirect Block Pointer (Points to a block that is a list of block pointers to Doubly Indirect Blocks)
    gen_num: u32,                 // Generation number (Primarily used for NFS)
    _0: u32,                      // In Ext2 version 0, this field is reserved. In version >= 1, Extended attribute block (File ACL).
    _1: u32,                      // In Ext2 version 0, this field is reserved. In version >= 1, Upper 32 bits of file size (if feature bit set) if it's a file, Directory ACL if it's a directory
    blk_addr_frag: u32,           // Block address of fragment
    os_val_2: [u8; 12],           // Operating System Specific Value #2
}

/// The name IMMEDIATELY FOLLOWS this struct
#[repr(C)]
struct DirEntry {
    inode: u32,
    size: u16,
    name_len_lsb: u8,
    name_len_msb_or_ty_ind: u8,
    name_first_byte: u8,
}

#[allow(non_snake_case)]
mod DirEntryType {
pub const UNKNOWN:   u8 = 0;
pub const FILE:      u8 = 1;
pub const DIR:       u8 = 2;
pub const CHAR_DEV:  u8 = 3;
pub const BLK_DEV:   u8 = 4;
pub const PIPE:      u8 = 5;
pub const SOCKET:    u8 = 6;
pub const SYM_LINK:  u8 = 7;
}

#[allow(non_snake_case)]
mod InodeTyPerms {
// Types
pub const PIPE:     u16 = 0x1 << 12;
pub const CHAR_DEV: u16 = 0x2 << 12;
pub const DIR:      u16 = 0x4 << 12;
pub const BLK_DEV:  u16 = 0x6 << 12;
pub const FILE:     u16 = 0x8 << 12;
pub const SYM_LINK: u16 = 0xA << 12;
pub const SOCKET:   u16 = 0xC << 12;
// Perms
pub const X_OTHER:      u16 = 0x001;
pub const W_OTHER:      u16 = 0x002;
pub const R_OTHER:      u16 = 0x004;
pub const X_GROUP:      u16 = 0x008;
pub const W_GROUP:      u16 = 0x010;
pub const R_GROUP:      u16 = 0x020;
pub const X_USER:       u16 = 0x040;
pub const W_USER:       u16 = 0x080;
pub const R_USER:       u16 = 0x100;
pub const STICKY:       u16 = 0x200;
pub const SET_GROUP_ID: u16 = 0x400;
pub const SET_USER_ID:  u16 = 0x800;
}

#[allow(non_snake_case)]
mod InodeFlags {
pub const SEC_DEL:                u32 = 0x00000001; // Secure deletion (not used)
pub const COP_DEL:                u32 = 0x00000002; // Keep a copy of data when deleted (not used)
pub const FS_COMP:                u32 = 0x00000004; // File compression (not used)
pub const SYNC_UP:                u32 = 0x00000008; // Synchronous updates—new data is written immediately to disk
pub const FILE_IMM:               u32 = 0x00000010; // Immutable file (content cannot be changed)
pub const APPEND_ONLY:            u32 = 0x00000020; // Append only
pub const FILE_NOT_DUMP:          u32 = 0x00000040; // File is not included in 'dump' command
pub const LAST_ACCESS_NOT_UPDATE: u32 = 0x00000080; // Last accessed time should not updated
pub const HAD_INDEX_DIR:          u32 = 0x00010000; // Hash indexed directory
pub const AFS_DIR:                u32 = 0x00020000; // AFS directory
pub const JOUNRAL_FILE_DATA:      u32 = 0x00040000; // Journal file data 
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum FsState {
    Clean = 1,
    Error = 2,
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum ErrProc {
    Ignore    = 1,
    ReMntRead = 2,
    Panic     = 3
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum OsId {
    Linux   = 0,
    Herd    = 1,
    MASIX   = 2,
    FreeBSD = 3,
    Lite    = 4,
}

#[allow(non_snake_case)]
pub(self) mod OptFeatFlags {
/// Preallocate some number of (contiguous?) blocks (see byte 205 in the superblock) to a directory when creating a new one (to reduce fragmentation?)
pub const PREALLOC_DIR: u16 = 1 << 0;
/// AFS server inodes exist
pub const AFS_INODES_EXIST: u16 = 1 << 1;
/// File system has a journal (Ext3)
pub const FS_HAS_JOURNAL: u16 = 1 << 2;
/// Inodes have extended attributes
pub const INODES_EXT: u16 = 1 << 3;
/// File system can resize itself for larger partitions
pub const FS_RESIZE_PART: u16 = 1 << 4;
/// Directories use hash index 
pub const DIR_HASH_IND: u16 = 1 << 5;
}

#[allow(non_snake_case)]
mod ReadReqFeatFlags {
/// Compression is used
pub const COMPRESSED: u16 = 1 << 0; 
/// Directory entries contain a type field
pub const DIR_CONTAIN_TYPE: u16 = 1 << 1; 
/// File system needs to replay its journal
pub const FS_REPLAY_JOURNAL: u16 = 1 << 2; 
/// File system uses a journal device 
pub const FS_USE_JOURNAL: u16 = 1 << 3; 
}

#[allow(non_snake_case)]
mod WriteReqFeatFlags {
/// Sparse superblocks and group descriptor tables
pub const SPARSE_SUPERBLOCK_DESC_TABLE: u16 = 1 << 0;
/// File system uses a 64-bit file size
pub const FILE_SIZE_64_BIT: u16 = 1 << 1;
/// Directory contents are stored in the form of a Binary Tree
pub const DIR_CONTENTS_BIN_TREE: u16 = 1 << 2; 
}

// Ignore extra space unsed in Ext2
impl Debug for Superblock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Superblock")
            .field("inode_num", &self.inode_num)
            .field("block_num", &self.block_num)
            .field("superuser_block_num", &self.superuser_block_num)
            .field("unallocated_blocks", &self.unallocated_blocks)
            .field("unallocated_inodes", &self.unallocated_inodes)
            .field("superblock_block_num", &self.superblock_block_num)
            .field("blck_size_shift", &self.blck_size_shift)
            .field("frag_size_shift", &self.frag_size_shift)
            .field("blocks_per_block_group", &self.blocks_per_block_group)
            .field("fragments_per_block_group", &self.fragments_per_block_group)
            .field("inodes_per_block_group", &self.inodes_per_block_group)
            .field("last_mount_time_posix", &self.last_mount_time_posix)
            .field("last_written_time_posix", &self.last_written_time_posix)
            .field("mounts_since_fsck", &self.mounts_since_fsck)
            .field("mounts_allowed_before_fsck", &self.mounts_allowed_before_fsck)
            .field("ext2_magic", &self.ext2_magic)
            .field("fs_state", &self.fs_state)
            .field("error_procedure", &self.error_procedure)
            .field("minor_version", &self.minor_version)
            .field("last_fsck_posix", &self.last_fsck_posix)
            .field("interval_fsck_posix", &self.interval_fsck_posix)
            .field("fs_os_id", &self.fs_os_id)
            .field("major_version", &self.major_version)
            .field("user_id_use_res_blocks", &self.user_id_use_res_blocks)
            .field("group_id_use_res_blocks", &self.group_id_use_res_blocks)
            .field("first_nonres_inode", &self.first_nonres_inode)
            .field("inode_size", &self.inode_size)
            .field("block_group", &self.block_group)
            .field("opt_feat", &self.opt_feat)
            .field("req_feat", &self.req_feat)
            .field("read_only_feat", &self.read_only_feat)
            .field("fsid", &self.fsid)
            .field("vol_name", &self.vol_name)
            .field("last_mount_path", &self.last_mount_path)
            .field("comp_alg", &self.comp_alg)
            .field("prealloc_blocks", &self.prealloc_blocks)
            .field("prealloc_dir", &self.prealloc_dir)
            .field("journal_id", &self.journal_id)
            .field("journal_inode", &self.journal_inode)
            .field("journal_dev", &self.journal_dev)
            .field("orphan_inode_head", &self.orphan_inode_head)
            .finish_non_exhaustive()
    }
}
