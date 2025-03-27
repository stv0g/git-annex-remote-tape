//! Linux Magnetic Tape Driver ioctl definitions
//!
//! from: /usr/include/x86_64-linux-gnu/sys/mtio.h
//!
//! also see: man 4 st

use nix;

#[repr(C)]
#[derive(Debug)]
pub struct mtop {
    pub mt_op: MTCmd,          // Operations defined below.
    pub mt_count: libc::c_int, // How many of them.
}

#[repr(i16)]
#[allow(dead_code)] // Do not warn about unused command
#[derive(Debug)]
pub enum MTCmd {
    MTRESET = 0,         // Reset drive in case of problems.
    MTFSF = 1,           // Forward space over FileMark position at first record of next file.
    MTBSF = 2,           // Backward space FileMark (position before FM).
    MTFSR = 3,           // Forward space record.
    MTBSR = 4,           // Backward space record.
    MTWEOF = 5,          // Write an end-of-file record (mark).
    MTREW = 6,           // Rewind.
    MTOFFL = 7,          // Rewind and put the drive offline (eject?).
    MTNOP = 8,           // No op, set status only (read with MTIOCGET).
    MTRETEN = 9,         // Retension tape.
    MTBSFM = 10,         // Backward space FileMark, position at FM.
    MTFSFM = 11,         // Forward space FileMark, position at FM.
    MTEOM = 12,          // Goto end of recorded media (for appending file).
    MTERASE = 13,        // Erase tape -- be careful!.
    MTRAS1 = 14,         // Run self test 1 (nondestructive).
    MTRAS2 = 15,         // Run self test 2 (destructive).
    MTRAS3 = 16,         // Reserved for self test 3.
    MTSETBLK = 20,       // Set block length (SCSI).
    MTSETDENSITY = 21,   // Set tape density (SCSI).
    MTSEEK = 22,         // Seek to block (Tandberg, etc.).
    MTTELL = 23,         // Tell block (Tandberg, etc.).
    MTSETDRVBUFFER = 24, // Set the drive buffering according to SCSI-2.
    MTFSS = 25,          // Space forward over setmarks.
    MTBSS = 26,          // Space backward over setmarks.
    MTWSM = 27,          // Write setmarks.
    MTLOCK = 28,         // Lock the drive door.
    MTUNLOCK = 29,       // Unlock the drive door.
    MTLOAD = 30,         // Execute the SCSI load command.
    MTUNLOAD = 31,       // Execute the SCSI unload command.
    MTCOMPRESSION = 32,  // Control compression with SCSI mode page 15.
    MTSETPART = 33,      // Change the active tape partition.
    MTMKPART = 34,       // Format the tape with one or two partitions.
    MTWEOFI = 35,        // Write an end-of-file record (mark) in immediate mode.
}

//#define	MTIOCTOP	_IOW('m', 1, struct mtop)	// Do a mag tape op.
nix::ioctl_write_ptr!(mtioctop, b'm', 1, mtop);

// From: /usr/include/x86_64-linux-gnu/sys/mtio.h
#[repr(C)]
#[derive(Debug)]
pub struct mtget {
    pub mt_type: MTType,        // Type of magtape device.
    pub mt_resid: libc::c_long, // Residual count: (not sure)
    // Number of bytes ignored, or
    // Number of files not skipped, or
    // Number of records not skipped.

    // The following registers are device dependent.
    pub mt_dsreg: libc::c_long,   // Status register.
    pub mt_gstat: GMTStatusFlags, // Generic (device independent) status.
    pub mt_erreg: libc::c_long,   // Error register.

    // The next two fields are not always used.
    pub mt_fileno: i32, // Number of current file on tape.
    pub mt_blkno: i32,  // Current block number.
}

//#define	MTIOCGET	_IOR('m', 2, struct mtget)	// Get tape status.
nix::ioctl_read!(mtiocget, b'm', 2, mtget);

#[repr(C)]
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct mtpos {
    pub mt_blkno: libc::c_long, // Current block number
}

//#define	MTIOCPOS	_IOR('m', 3, struct mtpos)	// Get tape position.
nix::ioctl_read!(mtiocpos, b'm', 3, mtpos);

pub const MT_ST_BLKSIZE_MASK: libc::c_long = 0x0ffffff;
pub const MT_ST_BLKSIZE_SHIFT: usize = 0;
pub const MT_ST_DENSITY_MASK: libc::c_long = 0xff000000;
pub const MT_ST_DENSITY_SHIFT: usize = 24;

#[repr(i64)]
#[derive(Default, Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum MTType {
    #[default]
    MT_ISUNKNOWN = 0x01,
    MT_ISQIC02 = 0x02,              /* Generic QIC-02 tape streamer */
    MT_ISWT5150 = 0x03,             /* Wangtek 5150EQ, QIC-150, QIC-02 */
    MT_ISARCHIVE_5945L2 = 0x04,     /* Archive 5945L-2, QIC-24, QIC-02? */
    MT_ISCMSJ500 = 0x05,            /* CMS Jumbo 500 (QIC-02?) */
    MT_ISTDC3610 = 0x06,            /* Tandberg 6310, QIC-24 */
    MT_ISARCHIVE_VP60I = 0x07,      /* Archive VP60i, QIC-02 */
    MT_ISARCHIVE_2150L = 0x08,      /* Archive Viper 2150L */
    MT_ISARCHIVE_2060L = 0x09,      /* Archive Viper 2060L */
    MT_ISARCHIVESC499 = 0x0A,       /* Archive SC-499 QIC-36 controller */
    MT_ISQIC02_ALL_FEATURES = 0x0F, /* Generic QIC-02 with all features */
    MT_ISWT5099EEN24 = 0x11,        /* Wangtek 5099-een24, 60MB, QIC-24 */
    MT_ISTEAC_MT2ST = 0x12,         /* Teac MT-2ST 155mb drive, Teac DC-1 card (Wangtek type) */
    MT_ISEVEREX_FT40A = 0x32,       /* Everex FT40A (QIC-40) */
    MT_ISDDS1 = 0x51,               /* DDS device without partitions */
    MT_ISDDS2 = 0x52,               /* DDS device with partitions */
    MT_ISONSTREAM_SC = 0x61, /* OnStream SCSI tape drives (SC-x0) SCSI = emulated (DI, DP, USB) */
    MT_ISSCSI1 = 0x71,       /* Generic ANSI SCSI-1 tape unit */
    MT_ISSCSI2 = 0x72,       /* Generic ANSI SCSI-2 tape unit */
}

// Generic Mag Tape (device independent) status macros for examining mt_gstat -- HP-UX compatible
// from: /usr/include/x86_64-linux-gnu/sys/mtio.h
bitflags::bitflags! {
    #[derive(Debug)]
    pub struct GMTStatusFlags: libc::c_long {
        const EOF = 0x80000000;
        const BOT = 0x40000000;
        const EOT = 0x20000000;
        const SM  = 0x10000000;  // DDS setmark.
        const EOD = 0x08000000;  // DDS EOD.
        const WR_PROT = 0x04000000;

        const ONLINE = 0x01000000;
        const D_6250 = 0x00800000;
        const D_1600 = 0x00400000;
        const D_800 = 0x00200000;
        const DRIVE_OPEN = 0x00040000;  // Door open (no tape).
        const IM_REP_EN =  0x00010000;  // Immediate report mode.
        const END_OF_STREAM = 0b00000001;
    }
}

bitflags::bitflags! {
    pub struct SetDrvBufferOptions: i32 {
        const MT_ST_BUFFER_WRITES =     1 << 0;
        const MT_ST_ASYNC_WRITES =      1 << 1;
        const MT_ST_READ_AHEAD =        1 << 2;
        const MT_ST_DEBUGGING =         1 << 3;
        const MT_ST_TWO_FM =            1 << 4;
        const MT_ST_FAST_MTEOM =        1 << 5;
        const MT_ST_AUTO_LOCK =         1 << 6;
        const MT_ST_DEF_WRITES =        1 << 7;
        const MT_ST_CAN_BSR =           1 << 8;
        const MT_ST_NO_BLKLIMS =        1 << 9;
        const MT_ST_CAN_PARTITIONS =    1 << 10;
        const MT_ST_SCSI2LOGICAL =      1 << 11;
        const MT_ST_SYSV =              1 << 12;
        const MT_ST_NOWAIT =            1 << 13;
        const MT_ST_SILI =  	        1 << 14;

        const MT_ST_BOOLEANS =          0x10000000;
        const MT_ST_SETBOOLEANS =       0x30000000;
        const MT_ST_CLEARBOOLEANS =     0x40000000;
        const MT_ST_WRITE_THRESHOLD =   0x20000000;
        const MT_ST_DEF_BLKSIZE =       0x50000000;
        const MT_ST_DEF_OPTIONS =       0x60000000;
        const MT_ST_SET_TIMEOUT =       0x70000000;
        const MT_ST_SET_LONG_TIMEOUT =  0x70100000;
        const MT_ST_SET_CLN =           0x80000000u32 as i32;
   }
}
