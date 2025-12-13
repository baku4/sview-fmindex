use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod generate;
mod build;
mod locate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate benchmark test data (legacy, generates both text and patterns)
    Generate {
        /// Data directory for test data
        #[arg(short, long, default_value = "test_data")]
        data_dir: PathBuf,

        /// Text length
        #[arg(short, long, default_value_t = 100000)]
        text_length: usize,

        /// Pattern length
        #[arg(short, long, default_value_t = 20)]
        pattern_length: usize,

        /// Number of patterns
        #[arg(short = 'n', long, default_value_t = 100)]
        pattern_count: usize,

        /// Random seed (default: 0)
        #[arg(short, long, default_value_t = 0)]
        seed: u64,
    },

    /// Generate text only
    GenerateText {
        /// Data directory for test data
        #[arg(short, long, default_value = "test_data")]
        data_dir: PathBuf,

        /// Text length
        #[arg(short, long, default_value_t = 100000)]
        text_length: usize,

        /// Random seed (default: 0)
        #[arg(short, long, default_value_t = 0)]
        seed: u64,

        /// Overwrite existing file
        #[arg(long)]
        overwrite: bool,
    },

    /// Generate patterns from existing text
    GeneratePattern {
        /// Data directory for test data
        #[arg(short, long, default_value = "test_data")]
        data_dir: PathBuf,

        /// Pattern length
        #[arg(short, long, default_value_t = 20)]
        pattern_length: usize,

        /// Number of patterns
        #[arg(short = 'n', long, default_value_t = 100)]
        pattern_count: usize,

        /// Cold pattern ratio (0.0 ~ 1.0). Cold patterns are new, warm patterns are repeated.
        #[arg(short, long, default_value_t = 1.0)]
        cold_ratio: f64,

        /// Random seed (default: 0)
        #[arg(short, long, default_value_t = 0)]
        seed: u64,

        /// Overwrite existing file
        #[arg(long)]
        overwrite: bool,
    },
    
    /// Build and save FM-index
    Build {
        /// Data directory
        #[arg(short, long, default_value = "test_data")]
        data_dir: PathBuf,
        
        /// Algorithm to use (lt-fm-index, sview-memory, sview-mmap)
        #[arg(short, long, default_value = "sview-memory")]
        algorithm: String,
        
        /// Suffix array sampling ratio
        #[arg(short, long, default_value_t = 2)]
        sasr: usize,
        
        /// Kmer lookup table size
        #[arg(short, long, default_value_t = 3)]
        klts: usize,
        
        /// Treat T as wildcard (only index ACG)
        #[arg(short, long)]
        treat_t_as_wildcard: bool,
    },
    
    /// Locate patterns using saved indices
    Locate {
        /// Data directory
        #[arg(short, long, default_value = "test_data")]
        data_dir: PathBuf,

        /// Algorithm to use (lt-fm-index, sview-memory, sview-mmap, or all)
        #[arg(short, long, default_value = "all")]
        algorithm: String,

        /// Treat T as wildcard (only index ACG)
        #[arg(short, long)]
        treat_t_as_wildcard: bool,

        /// Drop page caches before loading blob (requires sudo)
        #[arg(long)]
        drop_caches: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Generate { data_dir, text_length, pattern_length, pattern_count, seed } => {
            generate::generate_data(data_dir, text_length, pattern_length, pattern_count, seed)?;
        }
        Commands::GenerateText { data_dir, text_length, seed, overwrite } => {
            generate::generate_text(data_dir, text_length, seed, overwrite)?;
        }
        Commands::GeneratePattern { data_dir, pattern_length, pattern_count, cold_ratio, seed, overwrite } => {
            generate::generate_pattern(data_dir, pattern_length, pattern_count, cold_ratio, seed, overwrite)?;
        }
        Commands::Build { data_dir, algorithm, sasr, klts, treat_t_as_wildcard } => {
            let algorithm = algorithm.parse()?;
            build::build_indices(algorithm, data_dir, sasr, klts, treat_t_as_wildcard)?;
        }
        Commands::Locate { data_dir, algorithm, treat_t_as_wildcard, drop_caches } => {
            let algorithm = algorithm.parse()?;
            locate::locate_patterns(algorithm, data_dir, treat_t_as_wildcard, drop_caches)?;
        }
    }

    Ok(())
}
