use crate::Position;
use super::BuildError;

/// Configuration for the lookup table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupTableConfig {
    // Not use lookup table
    None,
    // Lookup table size is limited by max memory size
    MaxMemory(usize),
    // Use explicit k-mer size
    KmerSize(u32),
}

impl Default for LookupTableConfig {
    fn default() -> Self {
        Self::None
    }
}

impl LookupTableConfig {
    /// Get the k-mer size
    pub fn kmer_size<P: Position>(
        &self,
        symbol_count: u32,
    ) -> Result<u32, BuildError> {
        match self {
            Self::None => Ok(1),
            Self::MaxMemory(size) => Ok(Self::largest_kmer_size_below_max_memory_size::<P>(symbol_count, *size)),
            Self::KmerSize(size) => {
                if *size < 2 {
                    Err(BuildError::InvalidConfig("K-mer size must be at least 2".to_string()))
                } else {
                    Ok(*size)
                }
            }
        }
    }
    fn largest_kmer_size_below_max_memory_size<P: Position>(
        symbol_count: u32,
        max_memory_size: usize,
    ) -> u32 {
        let symbol_with_sentinel_count = symbol_count + 1; // +1 for sentinel
        let mut kmer_size = 2; // 1 is minimum k-mer size

        let size_cal_fn = |kmer_size: u32| (symbol_with_sentinel_count as usize).pow(kmer_size) * std::mem::size_of::<P>();

        while size_cal_fn(kmer_size) <= max_memory_size {
            kmer_size += 1;
        }
        kmer_size - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_kmer_sise_is_one() {
        // None
        let config = LookupTableConfig::None;
        assert_eq!(config.kmer_size::<u32>(1).unwrap(), 1);
        assert_eq!(config.kmer_size::<u32>(2).unwrap(), 1);

        // KmerSize
        let config = LookupTableConfig::KmerSize(1);
        assert!(config.kmer_size::<u32>(1).is_err());
        let config = LookupTableConfig::KmerSize(2);
        assert_eq!(config.kmer_size::<u32>(2).unwrap(), 2);
        
        // MaxMemory
        let config = LookupTableConfig::MaxMemory(0);
        assert_eq!(config.kmer_size::<u32>(1).unwrap(), 1);
        assert_eq!(config.kmer_size::<u32>(2).unwrap(), 1);
    }
}