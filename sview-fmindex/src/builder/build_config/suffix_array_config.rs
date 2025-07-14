use super::BuildError;

/// Configuration for the suffix array
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuffixArrayConfig {
    /// Not compressed
    Uncompressed,
    /// Compressed with given sampling ratio
    Compressed(u32),
}

impl Default for SuffixArrayConfig {
    fn default() -> Self {
        Self::Uncompressed
    }
}

impl SuffixArrayConfig {
    /// Get the sampling ratio
    pub fn sampling_ratio(&self) -> Result<u32, BuildError> {
        match self {
            Self::Uncompressed => Ok(1),
            Self::Compressed(ratio) => {
                if *ratio < 2 {
                    Err(BuildError::InvalidConfig(
                        "Sampling ratio for compressed suffix array must be at least 2".to_string()
                    ))
                } else {
                    Ok(*ratio)
                }
            },
        }
    }
}
