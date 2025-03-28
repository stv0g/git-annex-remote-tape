use nix::fcntl;
use nix::fcntl::OFlag;

use crate::mtio;
use std::fs::read_to_string;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::num::ParseIntError;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const ST_NBR_MODE_BITS: i32 = 2;
const ST_NBR_MODES: i32 = 1 << ST_NBR_MODE_BITS;
const ST_MODE_SHIFT: i32 = 7 - ST_NBR_MODE_BITS;
const ST_MODE_MASK: i32 = (ST_NBR_MODES - 1) << ST_MODE_SHIFT;

fn make_file_blocking(file: &File) -> Result<()> {
    let flags = fcntl::fcntl(file.as_raw_fd(), fcntl::FcntlArg::F_GETFD)?;

    let mut flags = OFlag::from_bits_truncate(flags);
    flags.remove(OFlag::O_NONBLOCK);

    fcntl::fcntl(file.as_raw_fd(), fcntl::FcntlArg::F_SETFL(flags))?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Errno(nix::Error),
    IO(io::Error),
    ParseIntError(ParseIntError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<nix::Error> for Error {
    fn from(value: nix::Error) -> Self {
        Self::Errno(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn tape_nr(minor: i32) -> i32 {
    (((minor) & !255) >> (ST_NBR_MODE_BITS + ST_MODE_SHIFT + 1))
        | ((minor) & ((1 << ST_MODE_SHIFT) - 1))
}

fn tape_mode(minor: i32) -> i32 {
    ((minor) & ST_MODE_MASK) >> ST_MODE_SHIFT
}

const ST_FORMATS: [&str; 16] = [
    "", "r", "k", "s", "l", "t", "o", "u", "m", "v", "p", "x", "a", "y", "q", "z",
];

pub struct MagneticTape {
    file: File,
}

impl MagneticTape {
    pub fn new(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK).open(path)?;

        make_file_blocking(&file)?;

        if !file.metadata()?.file_type().is_char_device() {
            return Err(Error::Errno(nix::errno::Errno::ENOTTY));
        }

        Ok(Self { file })
    }

    fn op(&self, cmd: mtio::MTCmd, count: i32) -> Result<i32> {
        unsafe {
            Ok(mtio::mtioctop(
                self.file.as_raw_fd(),
                &mtio::mtop {
                    mt_op: cmd,
                    mt_count: count,
                },
            )?)
        }
    }

    /// Write a block of data to the tape.
    pub fn write_block(&self, block: &[u8]) -> Result<usize> {
        let bytes_written = unsafe {
            libc::write(
                self.file.as_raw_fd(),
                block.as_ptr() as *const libc::c_void,
                block.len(),
            )
        };

        if bytes_written < 0 {
            return Err(io::Error::last_os_error().into());
        }

        Ok(bytes_written as usize)
    }

    /// Read a block of data from the tape.
    pub fn read_block(&self, block: &mut [u8]) -> Result<usize> {
        let bytes_read = unsafe {
            libc::read(
                self.file.as_raw_fd(),
                block.as_mut_ptr() as *mut libc::c_void,
                block.len(),
            )
        };

        if bytes_read < 0 {
            return Err(io::Error::last_os_error().into());
        }

        Ok(bytes_read as usize)
    }

    /// Get current tape position.
    pub fn get_position(&self) -> Result<i64> {
        let mut pos = mtio::mtpos::default();

        unsafe {
            mtio::mtiocpos(self.file.as_raw_fd(), &mut pos)?;
        }

        Ok(pos.mt_blkno)
    }

    /// Get drive status.
    pub fn get_status(&self) -> Result<mtio::mtget> {
        let mut status = mtio::mtget::default();

        unsafe {
            mtio::mtiocget(self.file.as_raw_fd(), &mut status)?;
        }

        Ok(status)
    }

    /// Reset drive in case of problems.
    pub fn reset(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTRESET, 0)
    }

    /// Forward space over FileMark position at first record of next file.
    pub fn fsf(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTFSF, count)
    }

    /// Backward space FileMark (position before FM).
    pub fn bsf(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTBSF, count)
    }

    /// Forward space record.
    pub fn fsr(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTFSR, count)
    }

    /// Backward space record.
    pub fn bsr(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTBSR, count)
    }

    /// Write an end-of-file record (mark).
    pub fn weof(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTWEOF, count)
    }

    /// Rewind.
    pub fn rewind(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTREW, 0)
    }

    /// Rewind and put the drive offline (eject?).
    pub fn offline(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTOFFL, 0)
    }

    /// No op, set status only (read with MTIOCGET).
    pub fn flush_drive_buffer(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTNOP, 0)
    }

    /// Retension tape.
    pub fn retension(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTRETEN, 0)
    }

    /// Backward space FileMark, position at FM.
    pub fn bsfm(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTBSFM, count)
    }

    /// Forward space FileMark, position at FM.
    pub fn fsfm(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTFSFM, count)
    }

    /// Goto end of recorded media (for appending file).
    pub fn eom(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTEOM, 0)
    }

    /// Erase tape.
    pub fn erase(&self, fast: bool) -> Result<i32> {
        self.op(mtio::MTCmd::MTERASE, if fast { 1 } else { 0 })
    }

    /// Set block length.
    pub fn set_block_length(&self, length: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTSETBLK, length)
    }

    /// Set tape density.
    pub fn set_density(&self, density: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTSETDENSITY, density)
    }

    /// Seek to block.
    pub fn seek(&self, block: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTSEEK, block)
    }

    /// Tell block.
    pub fn tell(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTTELL, 0)
    }

    /// Set the drive buffering according to SCSI-2.
    pub fn set_drive_buffer(&self, opts: mtio::SetDrvBufferOptions) -> Result<i32> {
        self.op(mtio::MTCmd::MTSETDRVBUFFER, opts.bits())
    }

    /// Get the drives boolean options.
    pub fn get_options(&self) -> Result<mtio::SetDrvBufferOptions> {
        let minor = nix::sys::stat::minor(self.file.metadata()?.st_rdev());
        let no = tape_nr(minor as i32);
        let mode = tape_mode(minor as i32) << (4 - ST_NBR_MODE_BITS);
        let fname = format!(
            "/sys/class/scsi_tape/st{}{}/options",
            no, ST_FORMATS[mode as usize]
        );

        let buf = read_to_string(&fname)?;

        let options_int = i32::from_str_radix(buf.trim().trim_start_matches("0x"), 16)?;
        let options = mtio::SetDrvBufferOptions::from_bits_truncate(options_int);

        Ok(options)
    }

    /// Set the drives boolean options.
    pub fn set_options(&self, opts: mtio::SetDrvBufferOptions) -> Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_BOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Add the drives boolean options.
    pub fn add_options(&self, opts: mtio::SetDrvBufferOptions) -> Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_SETBOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Clear the drives boolean options.
    pub fn clear_options(&self, opts: mtio::SetDrvBufferOptions) -> Result<i32> {
        let cmd = opts | mtio::SetDrvBufferOptions::MT_ST_CLEARBOOLEANS;
        self.set_drive_buffer(cmd)
    }

    /// Space forward over setmarks.
    pub fn fss(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTFSS, count)
    }

    /// Space backward over setmarks.
    pub fn bss(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTBSS, count)
    }

    /// Write setmarks.
    pub fn wsm(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTWSM, count)
    }

    /// Lock the drive door.
    pub fn lock(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTLOCK, 0)
    }

    /// Unlock the drive door.
    pub fn unlock(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTUNLOCK, 0)
    }

    /// Execute the SCSI load command.
    pub fn load(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTLOAD, 0)
    }

    /// Execute the SCSI unload command.
    pub fn unload(&self) -> Result<i32> {
        self.op(mtio::MTCmd::MTUNLOAD, 0)
    }

    /// Control compression with SCSI mode page 15.
    pub fn set_compression(&self, enabled: bool) -> Result<i32> {
        self.op(mtio::MTCmd::MTCOMPRESSION, if enabled { 1 } else { 0 })
    }

    /// Change the active tape partition.
    pub fn set_partition(&self, partition: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTSETPART, partition)
    }

    /// Format the tape with one or two partitions.
    pub fn make_partition(&self, part_size: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTMKPART, part_size)
    }

    /// Write an end-of-file record (mark) in immediate mode.
    pub fn weof_immediate(&self, count: i32) -> Result<i32> {
        self.op(mtio::MTCmd::MTWEOFI, count)
    }
}
