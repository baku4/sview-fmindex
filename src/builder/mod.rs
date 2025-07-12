use std::marker::PhantomData;

use crate::{
    // traits
    Position, Block,
    components::{
        Header, View,
        // headers
        MagicNumber, ChrEncodingTable, CountArrayHeader, SuffixArrayHeader, BwmHeader,
        // views
        CountArrayView, SuffixArrayView, BwmView,
    },
};

pub mod build_config;

pub struct FmIndexBuilder<P: Position, B: Block> {
    // Unchangeable after init
    text_len: usize,
    chr_count: u32,
    magic_number: MagicNumber,
    chr_encoding_table: ChrEncodingTable,
    // Configs
    suffix_array_config: build_config::SuffixArrayConfig,
    lookup_table_config: build_config::LookupTableConfig,
    // Changeable after init
    count_array_header: CountArrayHeader,
    suffix_array_header: SuffixArrayHeader,
    bwm_header: BwmHeader,
    // Phantom data
    _phantom: PhantomData<(P, B)>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// The number of distinct characters exceeds the capacity of the chosen block type.
    #[error("The character count ({1}) exceeds the maximum for the chosen block type ({0}). Try using a larger block type or reducing the number of characters.")]
    ChrCountOver(u32, u32),

    /// The length of the provided text does not match the length declared during builder initialization.
    #[error("Mismatched text length: expected {0} bytes, but got {1} bytes.")]
    UnmatchedTextLength(usize, usize),

    /// The provided blob slice has an incorrect size.
    #[error("Incorrect blob size: expected {0} bytes, but got {1} bytes.")]
    InvalidBlobSize(usize, usize),
    
    /// The provided blob slice is not properly aligned.
    #[error("Improper blob alignment: required alignment is {0} bytes, but the blob has an offset of {1} bytes.")]
    NotAlignedBlob(usize, usize),

    /// An invalid build configuration was provided.
    #[error("Invalid build configuration: {0}")]
    InvalidConfig(String),
}

impl<P: Position, B: Block> FmIndexBuilder<P, B> {
    // ================================================
    // Set up builder
    // ================================================
    pub fn init<T: AsRef<[u8]>>(
        text_len: usize,
        characters_by_index: &[T],
    ) -> Result<Self, BuildError> {
        let suffix_array_config = build_config::SuffixArrayConfig::default();
        let lookup_table_config = build_config::LookupTableConfig::default();

        let chr_count = characters_by_index.len() as u32;
        let chr_encoding_table = ChrEncodingTable::new(
            characters_by_index,
        );
        if chr_count > B::MAX_CHR {
            return Err(BuildError::ChrCountOver(B::MAX_CHR, chr_count));
        }

        // Generate headers
        let (count_array_header, suffix_array_header, bwm_header) = Self::generate_headers(
            text_len,
            chr_count,
            &suffix_array_config,
            &lookup_table_config,
        )?;

        Ok(Self {
            // Unchangeable after init
            text_len,
            chr_count,
            magic_number: MagicNumber::new(),
            chr_encoding_table,
            // Configs
            lookup_table_config,
            suffix_array_config,
            // Changeable after init
            count_array_header,
            suffix_array_header,
            bwm_header,
            // Phantom data
            _phantom: PhantomData,
        })
    }
    fn generate_headers(
        text_len: usize,
        chr_count: u32,
        suffix_array_config: &build_config::SuffixArrayConfig,
        lookup_table_config: &build_config::LookupTableConfig,
    ) -> Result<(CountArrayHeader, SuffixArrayHeader, BwmHeader), BuildError> {
        let lookup_table_kmer_size = lookup_table_config.kmer_size::<P>(chr_count)?;
        let suffix_array_sampling_ratio = suffix_array_config.sampling_ratio()?;

        let count_array_header = CountArrayHeader::new(
            chr_count,
            lookup_table_kmer_size,
        );
        let suffix_array_header = SuffixArrayHeader::new(
            text_len as u64,
            suffix_array_sampling_ratio,
        );
        let bwm_header = BwmHeader::new::<P, B>(
            text_len as u64,
            chr_count + 1,
        );

        Ok((
            count_array_header,
            suffix_array_header,
            bwm_header,
        ))
    }
    pub fn set_lookup_table_config(self, config: build_config::LookupTableConfig) -> Result<Self, BuildError> {
        let (count_array_header, suffix_array_header, bwm_header) = Self::generate_headers(
            self.text_len,
            self.chr_count,
            &self.suffix_array_config,
            &config,
        )?;

        Ok(Self {
            lookup_table_config: config,
            count_array_header,
            suffix_array_header,
            bwm_header,
            ..self
        })
    }
    pub fn set_suffix_array_config(self, config: build_config::SuffixArrayConfig) -> Result<Self, BuildError> {
        let (count_array_header, suffix_array_header, bwm_header) = Self::generate_headers(
            self.text_len,
            self.chr_count,
            &config,
            &self.lookup_table_config,
        )?;

        Ok(Self {
            suffix_array_config: config,
            count_array_header,
            suffix_array_header,
            bwm_header,
            ..self
        })
    }

    // ================================================
    // Blob size calculation
    // ================================================
    /// Calculate the total size of the blob in bytes
    pub fn blob_size(&self) -> usize {
        self.header_size() + self.body_size()
    }
    // Header size in bytes
    fn header_size(&self) -> usize {
        self.magic_number.aligned_size::<B>()
        + self.chr_encoding_table.aligned_size::<B>()
        + self.count_array_header.aligned_size::<B>()
        + self.suffix_array_header.aligned_size::<B>()
        + self.bwm_header.aligned_size::<B>()
    }
    // Body size in bytes
    fn body_size(&self) -> usize {
        CountArrayView::<P>::aligned_body_size::<B>(&self.count_array_header)
        + SuffixArrayView::<P>::aligned_body_size::<B>(&self.suffix_array_header) 
        + BwmView::<P, B>::aligned_body_size::<B>(&self.bwm_header)
    }

    // ================================================
    // Build
    // ================================================
    /// Build the FM-index and write to the provided blob slice
    pub fn build<'a>(
        &self,
        mut text: Vec<u8>,
        blob: &'a mut [u8],
    ) -> Result<(), BuildError> {
        // Check text length
        if text.len() != self.text_len {
            return Err(BuildError::UnmatchedTextLength(self.text_len, text.len()));
        }

        // Check alignment
        let required_alignment = B::ALIGN_SIZE;
        let offset = blob.as_ptr() as usize % required_alignment;
        if offset != 0 {
            return Err(BuildError::NotAlignedBlob(required_alignment, offset));
        }

        // Check blob size
        let blob_size = self.blob_size();
        let blob_size_actual = blob.len();
        if blob_size != blob_size_actual {
            return Err(BuildError::InvalidBlobSize(blob_size, blob_size_actual));
        }

        // 1) Write headers
        let mut header_start_index = 0;
        // Magic number
        let mut header_end_index = self.magic_number.aligned_size::<B>();
        self.magic_number.write_to_blob(&mut blob[header_start_index..header_end_index]);
        // Chr encoding table
        header_start_index = header_end_index;
        header_end_index += self.chr_encoding_table.aligned_size::<B>();
        self.chr_encoding_table.write_to_blob(&mut blob[header_start_index..header_end_index]);
        // Count array header
        header_start_index = header_end_index;
        header_end_index += self.count_array_header.aligned_size::<B>();
        self.count_array_header.write_to_blob(&mut blob[header_start_index..header_end_index]);
        // Suffix array header
        header_start_index = header_end_index;
        header_end_index += self.suffix_array_header.aligned_size::<B>();
        self.suffix_array_header.write_to_blob(&mut blob[header_start_index..header_end_index]);
        // BWM header
        header_start_index = header_end_index;
        header_end_index += self.bwm_header.aligned_size::<B>();
        self.bwm_header.write_to_blob(&mut blob[header_start_index..header_end_index]);

        // 2) Build & write bodies
        let mut body_start_index = header_end_index;
        let mut body_end_index = body_start_index + CountArrayView::<P>::aligned_body_size::<B>(&self.count_array_header);
        // Count array
        //  - encode text with encoding table
        //  - during encoding, count the number of each character & kmer
        self.count_array_header.count_and_encode_text::<P, B>(
            &mut text,
            &self.chr_encoding_table,
            &mut blob[body_start_index..body_end_index],
        );
        // Suffix array
        //  - burrow-wheeler transform
        //  - get sentinel character index
        body_start_index = body_end_index;
        body_end_index = body_start_index + SuffixArrayView::<P>::aligned_body_size::<B>(&self.suffix_array_header);

        let sentinel_chr_index = self.suffix_array_header.write_to_blob_and_get_sentinel_chr_index::<P>(
            &mut text,
            &mut blob[body_start_index..body_end_index],
        );
        // BWM
        body_start_index = body_end_index;
        body_end_index = body_start_index + BwmView::<P, B>::aligned_body_size::<B>(&self.bwm_header);
        self.bwm_header.encode_bwm_body::<P, B>(
            text,
            sentinel_chr_index, 
            &mut blob[body_start_index..body_end_index],
        );

        Ok(())
    }
}
