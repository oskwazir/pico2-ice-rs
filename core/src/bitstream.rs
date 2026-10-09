// SPDX-License-Identifier: MIT OR Apache-2.0

/// The bitstream is missing FF 00 magic or no sync word in the first 256 bytes
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NotABitstream;

// Constants for bitstream preamble
// https://prjicestorm.readthedocs.io/en/latest/format.html

/// First two bytes of every iCE40 bitstream
const MAGIC: [u8; 2] = [0xFF, 0x00];
/// Sync word marks the start of configuration data. MSB first
const SYNC_WORD: [u8; 4] = [0x7E, 0xAA, 0x99, 0x7E];
/// The whole sync word must lie inside this range.
const SYNC_SEARCH_WINDOW: usize = 256;

/// Validates the bitstream header only. It does not validate the rest of the file. A truncated or corrupt bitstream still passes!
///
/// <https://prjicestorm.readthedocs.io/en/latest/format.html>
pub fn check(data: &[u8]) -> Result<(), NotABitstream> {
    // if data does not start with MAGIC then return NotABitstream error
    if !data.starts_with(&MAGIC) {
        return Err(NotABitstream);
    }

    // look for SYNC_WORD in the SYNC_SEARCH_WINDOW range
    let has_sync_word: bool = data[..data.len().min(SYNC_SEARCH_WINDOW)]
        .windows(SYNC_WORD.len())
        .any(|w| w == SYNC_WORD);

    // if data does not have SYNC_WORD in the SYNC_SEARCH_WINDOW range, return NotABitstream error
    if !has_sync_word {
        return Err(NotABitstream);
    }

    // This does not check for correctness of the bitstream beyond the checks above
    // so Ok just means the header is okay
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_rejected() {
        assert_eq!(check(&[]), Err(NotABitstream))
    }

    #[test]
    fn shorter_than_header_is_rejected() {
        assert_eq!(check(b"\xFF\x00\x00\xFF\x7E"), Err(NotABitstream));
    }

    #[test]
    fn wrong_first_byte_is_rejected() {
        // First byte is incorrect - should be 0xFF from MAGIC
        assert_eq!(
            check(b"\x00\x00\x00\xFF\x7E\xAA\x99\x7E"),
            Err(NotABitstream)
        );
    }

    #[test]
    fn no_sync_word_is_rejected() {
        assert_eq!(
            check(b"\xFF\x00\x00\xFF\x00\x00\x00\x00"),
            Err(NotABitstream)
        );
    }

    #[test]
    fn sync_word_past_window_is_rejected() {
        let mut data = [0u8; SYNC_SEARCH_WINDOW + SYNC_WORD.len()];
        data[..MAGIC.len()].copy_from_slice(&MAGIC);
        data[SYNC_SEARCH_WINDOW..].copy_from_slice(&SYNC_WORD);
        assert_eq!(check(&data), Err(NotABitstream));
    }

    #[test]
    fn sync_word_at_window_edge_is_accepted() {
        let mut data = [0u8; SYNC_SEARCH_WINDOW + SYNC_WORD.len()];
        data[..MAGIC.len()].copy_from_slice(&MAGIC);
        data[SYNC_SEARCH_WINDOW - SYNC_WORD.len()..SYNC_SEARCH_WINDOW].copy_from_slice(&SYNC_WORD);
        assert_eq!(check(&data), Ok(()));
    }

    #[test]
    fn sync_word_after_comment_is_accepted() {
        // FF 00, one zero-terminated comment, 00 FF, then the sync word
        assert_eq!(
            check(b"\xFF\x00Part: iCE40UP5K-SG48\x00\x00\xFF\x7E\xAA\x99\x7E"),
            Ok(())
        );
    }

    #[test]
    fn sdk_rgb_blink_is_accepted() {
        let rgb_blink: &[u8] = include_bytes!("testdata/rgb_blink.bin");
        assert_eq!(check(rgb_blink), Ok(()));
    }
}
