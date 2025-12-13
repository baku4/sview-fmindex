use std::fs;
use std::path::PathBuf;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Generate text only
pub fn generate_text(
    data_dir: PathBuf,
    text_length: usize,
    seed: u64,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    println!("Generating text...");
    println!("Output directory: {}", data_dir.display());
    println!("Text length: {}", text_length);
    println!("Seed: {}", seed);
    println!("Overwrite: {}", overwrite);
    println!();

    // 출력 디렉토리 생성
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    let text_path = data_dir.join("text.txt");

    // 파일이 이미 존재하고 overwrite가 false면 스킵
    if text_path.exists() && !overwrite {
        println!("Text file already exists: {}", text_path.display());
        println!("Use --overwrite to overwrite.");
        return Ok(());
    }

    // RNG 초기화
    let mut rng = StdRng::seed_from_u64(seed);

    // ACGT 문자 생성
    let nucleotides = [b'A', b'C', b'G', b'T'];

    // text.txt 파일 생성 (개행 없음)
    let text: Vec<u8> = (0..text_length)
        .map(|_| nucleotides[rng.gen_range(0..4)])
        .collect();
    fs::write(&text_path, &text)?;
    println!("Text file created: {}", text_path.display());

    let total_time = start_time.elapsed().as_nanos();
    println!("Text generation completed successfully!");
    println!("Total time: {} ns", total_time);

    Ok(())
}

/// Generate patterns with cold/warm ratio
pub fn generate_pattern(
    data_dir: PathBuf,
    pattern_length: usize,
    pattern_count: usize,
    cold_ratio: f64,
    seed: u64,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    println!("Generating patterns...");
    println!("Output directory: {}", data_dir.display());
    println!("Pattern length: {}", pattern_length);
    println!("Pattern count: {}", pattern_count);
    println!("Cold ratio: {:.0}%", cold_ratio * 100.0);
    println!("Seed: {}", seed);
    println!("Overwrite: {}", overwrite);
    println!();

    // 텍스트 파일 읽기
    let text_path = data_dir.join("text.txt");
    if !text_path.exists() {
        return Err(format!("Text file not found: {}. Run generate-text first.", text_path.display()).into());
    }
    let text = fs::read(&text_path)?;
    let text_length = text.len();
    println!("Loaded text: {} bytes", text_length);

    let pattern_path = data_dir.join("pattern.txt");

    // 파일이 이미 존재하고 overwrite가 false면 스킵
    if pattern_path.exists() && !overwrite {
        println!("Pattern file already exists: {}", pattern_path.display());
        println!("Use --overwrite to overwrite.");
        return Ok(());
    }

    // RNG 초기화
    let mut rng = StdRng::seed_from_u64(seed);

    // Cold/Warm 패턴 개수 계산
    let cold_count = (cold_ratio * pattern_count as f64).ceil() as usize;
    let cold_count = cold_count.min(pattern_count); // 최대값 제한
    let warm_count = pattern_count - cold_count;

    println!("Cold patterns: {} (new)", cold_count);
    println!("Warm patterns: {} (repeated)", warm_count);

    let max_start_index = text_length.saturating_sub(pattern_length);

    // Cold 패턴: 랜덤 위치에서 추출 (새로운 패턴들)
    let cold_patterns: Vec<Vec<u8>> = (0..cold_count)
        .map(|_| {
            let start_index = rng.gen_range(0..=max_start_index);
            text[start_index..start_index + pattern_length].to_vec()
        })
        .collect();

    // Warm 패턴: cold 패턴들을 반복해서 사용
    let warm_patterns: Vec<Vec<u8>> = if cold_count > 0 {
        (0..warm_count)
            .map(|i| cold_patterns[i % cold_count].clone())
            .collect()
    } else {
        Vec::new()
    };

    // cold 먼저, 그 다음 warm
    let all_patterns: Vec<Vec<u8>> = cold_patterns
        .into_iter()
        .chain(warm_patterns.into_iter())
        .collect();

    // 패턴 파일 저장
    let pattern_content = all_patterns
        .iter()
        .map(|p| String::from_utf8_lossy(p))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&pattern_path, pattern_content)?;
    println!("Pattern file created: {}", pattern_path.display());

    let total_time = start_time.elapsed().as_nanos();
    println!("Pattern generation completed successfully!");
    println!("Total time: {} ns", total_time);

    Ok(())
}

/// Legacy function for backward compatibility
pub fn generate_data(
    data_dir: PathBuf,
    text_length: usize,
    pattern_length: usize,
    pattern_count: usize,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    generate_text(data_dir.clone(), text_length, seed, true)?;
    generate_pattern(data_dir, pattern_length, pattern_count, 1.0, seed, true)?;
    Ok(())
} 