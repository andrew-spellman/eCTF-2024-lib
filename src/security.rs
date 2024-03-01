use crate::secret::SECRET;
use max78000_hal::{
    aes::{AESIterExt, CipherType, Key, AES},
    error::{ErrorKind, Result},
    i2c::{I2CPort0, I2C},
    trng::TRNG,
};

const KEY: Key = Key::Bits128(&SECRET);

pub fn _secure_master_transaction(
    i2c: &mut I2C<I2CPort0>,
    aes: &mut AES,
    trng: &mut TRNG,
    address: usize,
    rx_len: usize,
    tx: &[u8],
) -> Result<Option<[u8; 80]>> {
    if rx_len % 16 != 0 {
        return Err(ErrorKind::BadParam);
    }

    let random = trng.get_trng_data();

    aes.set_key(&KEY);

    let mut cipher_iter = tx
        .iter()
        .copied()
        .map(|byte| byte ^ random as u8)
        .cipher(aes, CipherType::Encrypt)
        .peekable();

    while cipher_iter.peek().is_some() {
        let mut buffer = [0; 16];
        buffer
            .iter_mut()
            .zip(&mut cipher_iter)
            .for_each(|(buf_item, iter_item)| *buf_item = iter_item);

        i2c.master_transaction(address, None, Some(&buffer))?
    }

    if rx_len > 0 {
        let mut rx_buffer: [u8; 80] = [0; 80];
        i2c.master_transaction(address, Some(&mut rx_buffer[0..rx_len]), None)?;
        rx_buffer
            .clone()
            .iter()
            .copied()
            .cipher(aes, CipherType::Decrypt)
            .map(|byte| byte ^ random as u8)
            .zip(rx_buffer.iter_mut())
            .for_each(|(cipher, plain)| *plain = cipher)
    }
    Ok(None)
}

pub fn _secure_slave_transaction<Iter>(
    i2c: &mut I2C<I2CPort0>,
    aes: &mut AES,
    address: usize,
    mut tx: Iter,
    random: u32,
) -> Result<impl IntoIterator<Item = u8>>
where
    Iter: Iterator<Item = u8> + core::marker::Copy,
{
    let rx_iter = loop {
        match i2c.slave_manual_pulling([0u8; 0].into_iter()) {
            Ok(rx_iter) => break rx_iter,
            Err(ErrorKind::NoResponse) => (),
            Err(err) => return Err(err),
        }
        
    }

    loop {
        let crypt = tx.cipher(aes, CipherType::Encrypt)
        match i2c.slave_manual_pulling(tx) {
            Ok(rx_iter) => return Ok(rx_iter),
            Err(ErrorKind::NoResponse) => (),
            Err(err) => return Err(err),
        }
    }
}
