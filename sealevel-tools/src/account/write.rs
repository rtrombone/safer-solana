use std::io::{Error, ErrorKind, Result, Write};

use solana_program::program_memory::sol_memcpy;

use crate::account_info::DataAccount;

/// Inspired by https://github.com/coral-xyz/anchor/blob/v0.30.1/lang/src/bpf_writer.rs and
/// https://doc.rust-lang.org/src/std/io/impls.rs.html#369-406.
pub struct AccountWriter<'a, 'b, 'c> {
    account: &'c DataAccount<'a, 'b, true>,
    position: usize,
}

impl<'a, 'b, 'c> AccountWriter<'a, 'b, 'c> {
    pub fn new(account: &'c DataAccount<'a, 'b, true>) -> Self {
        Self {
            account,
            position: 0,
        }
    }
}

impl<'a, 'b, 'c> Write for AccountWriter<'a, 'b, 'c> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let Self {
            account: DataAccount(info),
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

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
