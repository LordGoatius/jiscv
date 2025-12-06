const SATP_PPN: usize = 0;
const SATP_ASID: usize = 44;
const SATP_MODE: usize = 60;

const SATP_SV39_ENABLE: usize = 0x8 << SATP_MODE;
/// "Valid" bit (entry is enabled)
const PAGE_V: usize = 1 << 0;   
/// Readable
const PAGE_R: usize = 1 << 1;   
/// Writable
const PAGE_W: usize = 1 << 2;   
/// Executable
const PAGE_X: usize = 1 << 3;   
/// User (accessible in user mode)
const PAGE_U: usize = 1 << 4;   
