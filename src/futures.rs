use futures::{io::AsyncReadExt, io::AsyncWriteExt, TryFutureExt};
use std::io::{Error, ErrorKind, Result};

use crate::{AsyncMinitelRead, AsyncMinitelWrite};

impl<T> AsyncMinitelRead for T
where
    T: futures::io::AsyncRead + Unpin,
{
    async fn read(&mut self, data: &mut [u8]) -> Result<()> {
        self.read_exact(data)
            .map_err(|e| Error::new(ErrorKind::Other, e))
            .await?;
        Ok(())
    }
}

impl<T> AsyncMinitelWrite for T
where
    T: futures::io::AsyncWrite + Unpin,
{
    async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.write_all(data)
            .map_err(|e| Error::new(ErrorKind::Other, e))
            .await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        futures::AsyncWriteExt::flush(self)
            .map_err(|e| Error::new(ErrorKind::Other, e))
            .await?;
        Ok(())
    }
}
