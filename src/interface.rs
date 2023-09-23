use defmt::trace;
use embedded_hal_async::spi::SpiBus;

use crate::mod_params::RadioError;
use crate::mod_params::RadioError::*;
use crate::mod_traits::InterfaceVariant;

pub(crate) struct SpiInterface<SPI, IV> {
    pub(crate) spi: SPI,
    pub(crate) iv: IV,
}

impl<SPI, IV> SpiInterface<SPI, IV>
where
    SPI: SpiBus<u8>,
    IV: InterfaceVariant,
{
    pub fn new(spi: SPI, iv: IV) -> Self {
        Self { spi, iv }
    }

    // Write one or more buffers to the radio.
    pub async fn write(&mut self, write_buffers: &[&[u8]], is_sleep_command: bool) -> Result<(), RadioError> {
        self.iv.set_nss_low().await?;
        for buffer in write_buffers {
            let write_err = self.spi.write(buffer).await.is_err();
            let flush_err = self.spi.flush().await.is_err();
            if write_err || flush_err {
                let _err = self.iv.set_nss_high().await;
                return Err(SPI);
            }
        }
        self.iv.set_nss_high().await?;

        if !is_sleep_command {
            self.iv.wait_on_busy().await?;
        }

        match write_buffers.len() {
            1 => trace!("write: 0x{:x}", write_buffers[0]),
            2 => trace!("write: 0x{:x} 0x{:x}", write_buffers[0], write_buffers[1]),
            3 => trace!(
                "write: 0x{:x} 0x{:x} 0x{:x}",
                write_buffers[0],
                write_buffers[1],
                write_buffers[2]
            ),
            _ => trace!("write: too many buffers"),
        }

        Ok(())
    }

    // Request a read, filling the provided buffer.
    pub async fn read(&mut self, write_buffers: &[&[u8]], read_buffer: &mut [u8]) -> Result<(), RadioError> {
        self.iv.set_nss_low().await?;
        for buffer in write_buffers {
            let write_err = self.spi.write(buffer).await.is_err();
            let flush_err = self.spi.flush().await.is_err();
            if write_err || flush_err {
                let _err = self.iv.set_nss_high().await;
                return Err(SPI);
            }
        }

        let read_result = self.spi.read(read_buffer).await.map_err(|_| RadioError::SPI);
        self.iv.set_nss_high();
        read_result?;

        self.iv.wait_on_busy().await?;

        match write_buffers.len() {
            1 => trace!("write: 0x{:x}", write_buffers[0]),
            2 => trace!("write: 0x{:x} 0x{:x}", write_buffers[0], write_buffers[1]),
            3 => trace!(
                "write: 0x{:x} 0x{:x} 0x{:x}",
                write_buffers[0],
                write_buffers[1],
                write_buffers[2]
            ),
            _ => trace!("write: too many buffers"),
        }
        trace!("read {}: 0x{:x}", read_buffer.len(), read_buffer);

        Ok(())
    }
}
