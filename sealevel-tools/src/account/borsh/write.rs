use borsh::io::{Error, ErrorKind, Result, Write};
use solana_program::program_memory::sol_memcpy;

use crate::account_info::Account;

/// Struct that implements [Write] for use with writable [Account].
///
/// Inspired by <https://github.com/coral-xyz/anchor/blob/v0.30.1/lang/src/bpf_writer.rs>.
pub struct BorshAccountWriter<'a, 'b> {
    account: &'b Account<'a, true>,
    position: usize,
}

impl<'a, 'b> BorshAccountWriter<'a, 'b> {
    /// Instantiate a new writer using a reference to writable [Account].
    ///
    /// Position defaults to zero.
    #[inline(always)]
    pub fn new(account: &'b Account<'a, true>) -> Self {
        Self {
            account,
            position: 0,
        }
    }
}

impl<'a, 'b> Write for BorshAccountWriter<'a, 'b> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let Self {
            account: Account(info),
            position,
        } = self;

        // Write only the amount the account can hold.
        let n = info.data_len().saturating_sub(*position).min(buf.len());

        // Try borrowing account info data to memcpy into it.
        let data: &mut [u8] = &mut info
            .try_borrow_mut_data()
            .map_err(|_| Error::new(ErrorKind::NotFound, "Cannot borrow account info data"))?;

        sol_memcpy(&mut data[*position..], buf, n);

        *position = position.saturating_add(n);

        Ok(n)
    }

    #[inline(always)]
    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        if self.write(data)? == data.len() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::WriteZero,
                "Failed to write all bytes",
            ))
        }
    }

    #[inline(always)]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}