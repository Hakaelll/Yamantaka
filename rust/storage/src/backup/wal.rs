//! Conservatively validate a complete WAL before SQLite can ignore damaged frames.
use crate::database::storage;
use std::{fs::File, io::Read, path::Path};
use yamantaka_domain::{Cancellation, Failure, Result};
fn invalid() -> Failure {
    Failure::Invalid("The WAL needs separate recovery; the complete library was preserved".into())
}
fn be(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().expect("Fixed WAL field"))
}
fn checksum(bytes: &[u8], little: bool, state: &mut [u32; 2]) {
    for pair in bytes.chunks_exact(8) {
        let word = |slice: &[u8]| {
            if little {
                u32::from_le_bytes(slice.try_into().unwrap())
            } else {
                be(slice)
            }
        };
        state[0] = state[0]
            .wrapping_add(word(&pair[..4]))
            .wrapping_add(state[1]);
        state[1] = state[1]
            .wrapping_add(word(&pair[4..]))
            .wrapping_add(state[0]);
    }
}
pub(super) fn validate(path: &Path, database: &Path, cancel: &Cancellation) -> Result<()> {
    cancel.check()?;
    let mut file = File::open(path).map_err(storage)?;
    let size = file.metadata().map_err(storage)?.len();
    if size == 0 {
        return Ok(());
    }
    if size < 32 {
        return Err(invalid());
    }
    let mut header = [0; 32];
    file.read_exact(&mut header).map_err(storage)?;
    let little = match be(&header[..4]) {
        0x377f0682 => true,
        0x377f0683 => false,
        _ => return Err(invalid()),
    };
    let page = be(&header[8..12]);
    let mut database_header = [0; 18];
    File::open(database)
        .and_then(|mut file| file.read_exact(&mut database_header))
        .map_err(storage)?;
    let database_page = u16::from_be_bytes(database_header[16..18].try_into().unwrap());
    let database_page = if database_page == 1 {
        65536
    } else {
        u32::from(database_page)
    };
    if be(&header[4..8]) != 3007000
        || &database_header[..16] != b"SQLite format 3\0"
        || database_page != page
        || !(512..=65536).contains(&page)
        || !page.is_power_of_two()
        || !(size - 32).is_multiple_of(u64::from(page) + 24)
    {
        return Err(invalid());
    }
    let mut state = [0; 2];
    checksum(&header[..24], little, &mut state);
    if state != [be(&header[24..28]), be(&header[28..32])] {
        return Err(invalid());
    }
    let mut frame = vec![0; page as usize + 24];
    for _ in 0..(size - 32) / (u64::from(page) + 24) {
        cancel.check()?;
        file.read_exact(&mut frame).map_err(storage)?;
        if be(&frame[..4]) == 0 || frame[8..16] != header[16..24] {
            return Err(invalid());
        }
        checksum(&frame[..8], little, &mut state);
        checksum(&frame[24..], little, &mut state);
        if state != [be(&frame[16..20]), be(&frame[20..24])] {
            return Err(invalid());
        }
    }
    Ok(())
}
