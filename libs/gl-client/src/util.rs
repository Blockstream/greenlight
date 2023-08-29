




pub fn is_feature_bit_enabled(bitmap : &[u8], index : usize) -> bool {
    let n_bytes = bitmap.len();
    let (byte_index, bit_index ) = (index / 8, index % 8);

    // The index doesn't fit in the byte-array
    if byte_index >= n_bytes {
        return false
    }

    let selected_byte = bitmap[n_bytes - 1 - byte_index];
    let bit_mask = 1u8 << (bit_index);


    return (selected_byte & bit_mask) != 0
}



#[cfg(test)]
mod test {
    use super::*;

    fn to_bitmap(feature_hex_string : &str) -> Result<Vec<u8>, String> {
        hex::decode(&feature_hex_string)
            .map_err(|x| x.to_string())
    }

    #[test]
    fn test_parse_bitmap() {
        // Check the lowest bits in the bitmap
        let feature_bitmap_00 = to_bitmap(&"01").unwrap();
        let feature_bitmap_01 = to_bitmap(&"02").unwrap();
        let feature_bitmap_02 = to_bitmap(&"04").unwrap();
        let feature_bitmap_03 = to_bitmap(&"08").unwrap();
        let feature_bitmap_04 = to_bitmap(&"10").unwrap();
        let feature_bitmap_05 = to_bitmap(&"20").unwrap();
        let feature_bitmap_06 = to_bitmap(&"40").unwrap();
        let feature_bitmap_07 = to_bitmap(&"80").unwrap();

        // Check that the expected bit is enabled
        assert!(is_feature_bit_enabled(&feature_bitmap_00, 0));
        assert!(is_feature_bit_enabled(&feature_bitmap_01, 1));
        assert!(is_feature_bit_enabled(&feature_bitmap_02, 2));
        assert!(is_feature_bit_enabled(&feature_bitmap_03, 3));
        assert!(is_feature_bit_enabled(&feature_bitmap_04, 4));
        assert!(is_feature_bit_enabled(&feature_bitmap_05, 5));
        assert!(is_feature_bit_enabled(&feature_bitmap_06, 6));
        assert!(is_feature_bit_enabled(&feature_bitmap_07, 7));

        // Check that other bits are disabled
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 0));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 2));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 3));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 4));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 5));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 6));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 7));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 8));
        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 9));

        assert!(! is_feature_bit_enabled(&feature_bitmap_01, 1000));
    }

    #[test]
    fn test_lsps_option_enabled_bitmap()  {
        // Copied from LSPS0
        // This set bit number 729
        let data = "0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let bitmap = to_bitmap(&data).unwrap();

        // Check that the expected bit is enabled
        assert!(is_feature_bit_enabled(&bitmap, 729));

        // Check that the expected bit is disabled
        assert!(! is_feature_bit_enabled(&bitmap, 728));
        assert!(! is_feature_bit_enabled(&bitmap, 730));
    }
}