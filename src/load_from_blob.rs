use crate::{
    FmIndex,
    // traits
    Position, Block,
    components::{
        Header, View,
        // headers
        MagicNumber, EncodingTable, CountArrayHeader, SuffixArrayHeader, BwmHeader,
        // views
        CountArrayView, SuffixArrayView, BwmView,
    },
};

/// Error type for loading fm-index from blob
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// The provided data does not appear to be a valid FM-index blob, often due to a magic number mismatch or corruption.
    #[error("Invalid FM-index format. The data does not appear to be a valid FM-index blob.")]
    InvalidFormat,

    /// The size of the provided blob does not match the expected size calculated from its headers.
    #[error("Mismatched blob size: headers indicate a total size of {0} bytes, but the provided blob is {1} bytes.")]
    MismatchedBlobSize(usize, usize),
}


impl<'a, P: Position, B: Block> FmIndex<'a, P, B> {
    /// Load fm-index from blob
    pub fn load(blob: &'a [u8]) -> Result<Self, LoadError> {
        // Load headers
        let (magic_number, remaining_bytes) = MagicNumber::read_from_blob::<B>(blob);
        if !(magic_number.is_valid() && magic_number.is_supported_version()) {
            return Err(LoadError::InvalidFormat);
        }
        let (encoding_table, remaining_bytes) = EncodingTable::read_from_blob::<B>(remaining_bytes);
        let (count_array_header, remaining_bytes) = CountArrayHeader::read_from_blob::<B>(remaining_bytes);
        let (suffix_array_header, remaining_bytes) = SuffixArrayHeader::read_from_blob::<B>(remaining_bytes);
        let (bwm_header, body_blob) = BwmHeader::read_from_blob::<B>(remaining_bytes);

        // check body size
        let actual_body_size = body_blob.len();
        let expected_body_size = {
            CountArrayView::<P>::aligned_body_size::<B>(&count_array_header)
            + SuffixArrayView::<P>::aligned_body_size::<B>(&suffix_array_header)
            + BwmView::<P, B>::aligned_body_size::<B>(&bwm_header)
        };
        if actual_body_size != expected_body_size {
            let header_size = {
                magic_number.aligned_size::<B>()
                + encoding_table.aligned_size::<B>()
                + count_array_header.aligned_size::<B>()
                + suffix_array_header.aligned_size::<B>()
                + bwm_header.aligned_size::<B>()
            };
            return Err(LoadError::MismatchedBlobSize(
                header_size + expected_body_size,
                header_size + actual_body_size,
            ));
        }

        // Get views
        //  - Count array
        let mut body_start_index = 0;
        let mut body_end_index = CountArrayView::<P>::aligned_body_size::<B>(&count_array_header);
        let count_array_view = CountArrayView::<P>::load_from_body::<B>(&count_array_header, &body_blob[body_start_index..body_end_index]);
        //  - Suffix array
        body_start_index = body_end_index;
        body_end_index += SuffixArrayView::<P>::aligned_body_size::<B>(&suffix_array_header);
        let suffix_array_view = SuffixArrayView::<P>::load_from_body::<B>(&suffix_array_header, &body_blob[body_start_index..body_end_index]);
        //  - BWM
        body_start_index = body_end_index;
        body_end_index += BwmView::<P, B>::aligned_body_size::<B>(&bwm_header);
        let bwm_view = BwmView::<P, B>::load_from_body::<B>(&bwm_header, &body_blob[body_start_index..body_end_index]);

        Ok(Self {
            magic_number,
            encoding_table,
            count_array_header,
            suffix_array_header,
            bwm_header,
            count_array_view,
            suffix_array_view,
            bwm_view,
        })
    }
}