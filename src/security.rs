use core::array::IntoIter;

use crate::secret::SECRET;
use max78000_hal::{
    aes::{AESIterExt, CipherType, Key, AES},
    error::{ErrorKind, Result},
    i2c::{I2CPort0, I2C},
    trng::TRNG,
};

const KEY: Key = Key::Bits128(&SECRET);

const BLOCK_SIZE: usize = 16;
const MAX_TRANSACTION_SIZE: usize = BLOCK_SIZE * 5;

const OVERALL_TRANSACTION_SIZE: usize = MAX_TRANSACTION_SIZE + BLOCK_SIZE;

pub enum TransactionKind {
    List,
    Boot,
    Attest,
    Raw([u8; MAX_TRANSACTION_SIZE]),
}

struct MasterChannel {
    trng_key: u8,
    kind: TransactionKind,
}

impl MasterChannel {
    fn into_slave(kind: TransactionKind, rand: u8) -> IntoIter<u8, OVERALL_TRANSACTION_SIZE> {
        let mut data = [0u8; OVERALL_TRANSACTION_SIZE];
        data[0] = rand;
        match kind {
            TransactionKind::List => data[1] = rand ^ b'L',
            TransactionKind::Boot => data[1] = rand ^ b'B',
            TransactionKind::Attest => data[1] = rand ^ b'A',
            TransactionKind::Raw(raw) => {
                data[1] = rand ^ b'R';
                data.iter_mut()
                    .skip(BLOCK_SIZE)
                    .zip(raw.into_iter().map(|raw| raw ^ rand))
                    .for_each(|(data, raw)| *data = raw)
            }
        }
        data[2] = rand;
        data[3] = rand;

        data.into_iter()
    }

    fn from_master<Iter>(bytes: &mut Iter) -> Option<Self>
    where
        Iter: Iterator<Item = u8>,
    {
        // for some reason, `bytes.next()? ^ trng_key` crashes rust-analyzer,
        // so here they are seperated.
        let (trng_key, kind) = (bytes.next()?, bytes.next()?);
        let kind = kind ^ trng_key;

        let kind = match kind {
            b'L' => TransactionKind::List,
            b'B' => TransactionKind::Boot,
            b'A' => TransactionKind::Attest,
            b'R' => {
                let mut data = [0u8; MAX_TRANSACTION_SIZE];
                data.iter_mut()
                    .zip(bytes.skip(BLOCK_SIZE - 2))
                    .for_each(|(data, byte)| *data = byte);
                TransactionKind::Raw(data)
            }

            _ => return None,
        };

        Some(Self { trng_key, kind })
    }
}

pub fn secure_master_transaction(
    i2c: &mut I2C<I2CPort0>,
    aes: &mut AES,
    trng: &mut TRNG,
    address: usize,
    kind: TransactionKind,
) -> Result<[u8; MAX_TRANSACTION_SIZE]> {
    let random = trng.get_trng_data() as u8;
    aes.set_key(&KEY);

    MasterChannel::into_slave(kind, random)
        .cipher(aes, CipherType::Encrypt)
        .array_chunks()
        .try_for_each(|buffer: [u8; BLOCK_SIZE]| {
            i2c.master_transaction(address, None, Some(&buffer))
        })?;

    let mut rx_buffer = [0u8; MAX_TRANSACTION_SIZE];
    i2c.master_transaction(address, Some(&mut rx_buffer), None)?;

    rx_buffer
        .clone()
        .into_iter()
        .cipher(aes, CipherType::Decrypt)
        .map(|byte| byte ^ random)
        .zip(rx_buffer.iter_mut())
        .for_each(|(cipher, plain)| *plain = cipher);

    Ok(rx_buffer)
}

pub fn secure_slave_transaction<TXFunc>(
    i2c: &mut I2C<I2CPort0>,
    aes: &mut AES,
    mon: TXFunc,
) -> Result<()>
where
    TXFunc: FnOnce(TransactionKind) -> [u8; MAX_TRANSACTION_SIZE],
{
    aes.set_key(&KEY);
    let MasterChannel { trng_key, kind } = MasterChannel::from_master(
        &mut loop {
            match i2c.slave_manual_pulling(&mut [0].into_iter().cycle()) {
                Ok(rx_iter) => break Ok(rx_iter),
                Err(ErrorKind::NoResponse) => (),
                Err(err) => break Err(err),
            }
        }?
        .into_iter()
        .cipher(aes, CipherType::Decrypt),
    )
    .ok_or(ErrorKind::Abort)?;

    let mut resp_iter = mon(kind)
        .into_iter()
        .map(|byte| byte ^ trng_key)
        .cipher(aes, CipherType::Encrypt);

    loop {
        match i2c.slave_manual_pulling(&mut resp_iter) {
            Ok(_) => break Ok(()),
            Err(ErrorKind::NoResponse) => (),
            Err(err) => break Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    extern crate std;
    use std::vec::Vec;

    #[test]
    fn test_making_master_channel_list() {
        for trng_key in 0..=255 {
            let host_channel: Vec<u8> = MasterChannel::into_slave(TransactionKind::List, trng_key)
                .take(16)
                .collect();

            assert_eq!(
                host_channel,
                &[
                    trng_key,
                    b'L' ^ trng_key,
                    trng_key,
                    trng_key,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            );
        }
    }

    #[test]
    fn test_making_master_channel_boot() {
        for trng_key in 0..=255 {
            let host_channel: Vec<u8> = MasterChannel::into_slave(TransactionKind::Boot, trng_key)
                .take(16)
                .collect();

            assert_eq!(
                host_channel,
                &[
                    trng_key,
                    b'B' ^ trng_key,
                    trng_key,
                    trng_key,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            );
        }
    }

    #[test]
    fn test_making_master_channel_attest() {
        for trng_key in 0..=255 {
            let host_channel: Vec<u8> =
                MasterChannel::into_slave(TransactionKind::Attest, trng_key)
                    .take(16)
                    .collect();

            assert_eq!(
                host_channel,
                &[
                    trng_key,
                    b'A' ^ trng_key,
                    trng_key,
                    trng_key,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            );
        }
    }

    #[test]
    fn test_making_master_channel_raw() {
        for trng_key in 0..=255 {
            let host_channel: Vec<u8> = MasterChannel::into_slave(
                TransactionKind::Raw([trng_key; MAX_TRANSACTION_SIZE]),
                trng_key,
            )
            // one extra byte to test trng_key ^ trng_key for the raw bytes
            .take(17)
            .collect();

            assert_eq!(
                host_channel,
                &[
                    trng_key,
                    b'R' ^ trng_key,
                    trng_key,
                    trng_key,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0
                ]
            );
        }
    }
}
