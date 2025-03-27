use crate::mtio;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const ST_NBR_MODE_BITS: i32 = 2;
const ST_NBR_MODES: i32 = 1 << ST_NBR_MODE_BITS;
const ST_MODE_SHIFT: i32 = 7 - ST_NBR_MODE_BITS;
const ST_MODE_MASK: i32 = (ST_NBR_MODES - 1) << ST_MODE_SHIFT;

fn tape_nr(minor: i32) -> i32 {
    (((minor) & !255) >> (ST_NBR_MODE_BITS + 1)) | ((minor) & ((1 << ST_MODE_SHIFT) - 1))
}

fn tape_mode(minor: i32) -> i32 {
    ((minor) & ST_MODE_MASK) >> ST_MODE_SHIFT
}

const ST_FORMATS: [&str; 16] = [
    "", "r", "k", "s", "l", "t", "o", "u", "m", "v", "p", "x", "a", "y", "q", "z",
];

pub struct Tape {
    fd: i32,
}

impl Tape {
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;

        let tape = Self {
            fd: file.as_raw_fd(),
        };

        tape.add_options(mtio::SetDrvBufferOptions::MT_ST_SCSI2LOGICAL)?;

        Ok(tape)
    }

    pub fn write_block(&self, block: &[u8]) -> io::Result<usize> {
        let bytes_written =
            unsafe { libc::write(self.fd, block.as_ptr() as *const libc::c_void, block.len()) };

        if bytes_written < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(bytes_written as usize)
    }

    pub fn read_block(&self, block: &mut [u8]) -> io::Result<usize> {
        let bytes_read = unsafe {
            libc::read(
                self.fd,
                block.as_mut_ptr() as *mut libc::c_void,
                block.len(),
            )
        };

        if bytes_read < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(bytes_read as usize)
    }

    pub fn get_position(&self) -> io::Result<i64> {
        let mut pos = mtio::mtpos::default();

        unsafe {
            mtio::mtiocpos(self.fd, &mut pos)?;
        }

        Ok(pos.mt_blkno)
    }

    pub fn get_status(&self) -> io::Result<mtio::mtget> {
        let mut status = mtio::mtget::default();

        unsafe {
            mtio::mtiocget(self.fd, &mut status)?;
        }

        Ok(status)
    }

    fn op(&self, cmd: mtio::MTCmd, count: i32) -> nix::Result<i32> {
        unsafe {
            mtio::mtioctop(
                self.fd,
                &mtio::mtop {
                    mt_op: cmd,
                    mt_count: count,
                },
            )
        }
    }

    /// Reset drive in case of problems.
    pub fn reset(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTRESET, 0)
    }

    /// Forward space over FileMark position at first record of next file.
    pub fn fsf(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTFSF, count)
    }

    /// Backward space FileMark (position before FM).
    pub fn bsf(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTBSF, count)
    }

    /// Forward space record.
    pub fn fsr(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTFSR, count)
    }

    /// Backward space record.
    pub fn bsr(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTBSR, count)
    }

    /// Write an end-of-file record (mark).
    pub fn weof(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTWEOF, count)
    }

    /// Rewind.
    pub fn rewind(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTREW, 0)
    }

    // Rewind and put the drive offline (eject?).
    pub fn offline(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTOFFL, 0)
    }

    /// No op, set status only (read with MTIOCGET).
    pub fn flush_drive_buffer(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTNOP, 0)
    }

    /// Retension tape.
    pub fn retension(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTRETEN, 0)
    }

    /// Backward space FileMark, position at FM.
    pub fn bsfm(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTBSFM, count)
    }

    /// Forward space FileMark, position at FM.
    pub fn fsfm(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTFSFM, count)
    }

    /// Goto end of recorded media (for appending file).
    pub fn eom(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTEOM, 0)
    }

    /// Erase tape.
    pub fn erase(&self, fast: bool) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTERASE, if fast { 1 } else { 0 })
    }

    /// Set block length.
    pub fn set_block_length(&self, length: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTSETBLK, length)
    }

    /// Set tape density.
    pub fn set_density(&self, density: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTSETDENSITY, density)
    }

    /// Seek to block.
    pub fn seek(&self, block: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTSEEK, block)
    }

    /// Tell block.
    pub fn tell(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTTELL, 0)
    }

    // Set the drive buffering according to SCSI-2.
    pub fn set_drive_buffer(&self, opts: mtio::SetDrvBufferOptions) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTSETDRVBUFFER, opts.bits())
    }

    /// Get the drives boolean options.
    pub fn get_options(&self) -> nix::Result<mtio::SetDrvBufferOptions> {
        let metadata = std::fs::metadata(format!("/proc/self/fd/{}", self.fd))?;
        if !metadata.file_type().is_char_device() {
            return Err(nix::errno::Errno::ENOTTY);
        }

        let minor = metadata.rdev() as i32;
        let no = tape_nr(minor);
        let mode = tape_mode(minor) << (4 - ST_NBR_MODE_BITS);
        let fname = format!("/sys/class/scsi_tape/st{}{}", no, ST_FORMATS[mode as usize]);

        let buf = read_to_string(&fname)?;
        let options_int = i32::from_str_radix(buf.trim(), 16)?;
        let options = mtio::SetDrvBufferOptions::from_bits_truncate(options_int);

        Ok(options)
    }

    /// Set the drives boolean options.
    pub fn set_options(&self, opts: mtio::SetDrvBufferOptions) -> nix::Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_BOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Add the drives boolean options.
    pub fn add_options(&self, opts: mtio::SetDrvBufferOptions) -> nix::Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_SETBOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Clear the drives boolean options.
    pub fn clear_options(&self, opts: mtio::SetDrvBufferOptions) -> nix::Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_CLEARBOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Space forward over setmarks.
    pub fn fss(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTFSS, count)
    }

    /// Space backward over setmarks.
    pub fn bss(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTBSS, count)
    }

    /// Write setmarks.
    pub fn wsm(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTWSM, count)
    }

    /// Lock the drive door.
    pub fn lock(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTLOCK, 0)
    }

    /// Unlock the drive door.
    pub fn unlock(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTUNLOCK, 0)
    }

    /// Execute the SCSI load command.
    pub fn load(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTLOAD, 0)
    }

    /// Execute the SCSI unload command.
    pub fn unload(&self) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTUNLOAD, 0)
    }

    /// Control compression with SCSI mode page 15.
    pub fn set_compression(&self, enabled: bool) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTCOMPRESSION, if enabled { 1 } else { 0 })
    }

    /// Change the active tape partition.
    pub fn set_partition(&self, partition: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTSETPART, partition)
    }

    /// Format the tape with one or two partitions.
    pub fn make_partition(&self, part_size: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTMKPART, part_size)
    }

    /// Write an end-of-file record (mark) in immediate mode.
    pub fn weof_immediate(&self, count: i32) -> nix::Result<i32> {
        self.op(mtio::MTCmd::MTWEOFI, count)
    }
}
