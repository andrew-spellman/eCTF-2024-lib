use max78000_hal::{
    error::Result,
    i2c::{I2CPort0, I2C},
};

pub fn _secure_master_transaction(
    i2c: &mut I2C<I2CPort0>,
    address: usize,
    rx: Option<&mut [u8]>,
    tx: Option<&[u8]>,
    random: u32,
) {
    _ = (i2c, address, rx, tx, random);
}

pub fn _secure_slave_transaction<RXFun, TXFun>(
    i2c: &mut I2C<I2CPort0>,
    address: usize,
    rx: RXFun,
    tx: TXFun,
    random: u32,
) -> Result<()>
where
    RXFun: FnMut(u8) -> Result<()>,
    TXFun: FnMut() -> Result<u8>,
{
    _ = (address, rx, tx, random);
    i2c.slave_transaction(
        |byte| {
            _ = byte;
            Ok(())
        },
        || {
            _ = 0;
            Ok(0)
        },
    )
}
