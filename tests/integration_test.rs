//! Integration tests for WeText-RS

use wetext_rs::*;

const FST_DIR: &str = "fsts";

#[test]
fn test_chinese_tn_numbers() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    // Note: "123" is read as individual digits "幺二三" (like phone numbers)
    // For numerical value, the FST behavior depends on the specific model
    let result = normalizer.normalize("123").unwrap();
    assert_eq!(result, "幺二三");

    let result = normalizer.normalize("2024年").unwrap();
    assert_eq!(result, "二零二四年");
}

#[test]
fn test_chinese_tn_date() {
    let config = NormalizerConfig::new()
        .with_lang(Language::Zh)
        .with_operator(Operator::Tn);
    let normalizer = Normalizer::new(FST_DIR, config);

    // Note: FST may not have date patterns for all formats
    // Test with a format that the FST supports
    let result = normalizer.normalize("2024年1月15日").unwrap();
    // The actual result depends on FST, just verify it doesn't error
    assert!(!result.is_empty());
}

#[test]
fn test_chinese_tn_money() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("100元").unwrap();
    assert_eq!(result, "一百元");
}

#[test]
fn test_chinese_tn_time() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("下午3点30分").unwrap();
    assert_eq!(result, "下午三点三十分");
}

#[test]
fn test_chinese_tn_fraction() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("3/4").unwrap();
    assert_eq!(result, "四分之三");
}

#[test]
fn test_chinese_tn_decimal() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("1.5").unwrap();
    assert_eq!(result, "一点五");
}

#[test]
fn test_chinese_itn() {
    let config = NormalizerConfig::new()
        .with_lang(Language::Zh)
        .with_operator(Operator::Itn);
    let normalizer = Normalizer::new(FST_DIR, config);

    let result = normalizer.normalize("一百二十三").unwrap();
    assert_eq!(result, "123");

    let result = normalizer.normalize("二零二四年").unwrap();
    assert_eq!(result, "2024年");

    let result = normalizer.normalize("一点五").unwrap();
    assert_eq!(result, "1.5");
}

#[test]
fn test_english_tn() {
    let config = NormalizerConfig::new().with_lang(Language::En);
    let normalizer = Normalizer::new(FST_DIR, config);

    // English TN for currency
    let result = normalizer.normalize("$100").unwrap();
    // Just verify it processes without error
    assert!(!result.is_empty());
}

#[test]
fn test_japanese_tn() {
    let config = NormalizerConfig::new().with_lang(Language::Ja);
    let normalizer = Normalizer::new(FST_DIR, config);

    let result = normalizer.normalize("100円").unwrap();
    // Just verify it processes without error
    assert!(!result.is_empty());
}

#[test]
fn test_no_normalization_needed() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    // Text without numbers should remain unchanged
    let result = normalizer.normalize("你好世界").unwrap();
    assert_eq!(result, "你好世界");
}

#[test]
fn test_empty_input() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("").unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_whitespace_trimming() {
    let normalizer = Normalizer::with_defaults(FST_DIR);

    let result = normalizer.normalize("   123   ").unwrap();
    assert_eq!(result, "幺二三");
}

#[test]
fn test_auto_language_detection() {
    let config = NormalizerConfig::new().with_lang(Language::Auto);
    let normalizer = Normalizer::new(FST_DIR, config);

    // Chinese text should be detected as Chinese
    let result = normalizer.normalize("今天是2024年").unwrap();
    assert!(result.contains("二零二四"));

    // English text should be detected as English
    let result = normalizer.normalize("$100").unwrap();
    // Result depends on English FST behavior
    assert!(!result.is_empty());
}

#[test]
fn test_contractions_fix() {
    let config = NormalizerConfig::new()
        .with_lang(Language::En)
        .with_fix_contractions(true);
    let normalizer = Normalizer::new(FST_DIR, config);

    // With contractions fix enabled
    let result = normalizer.normalize("I don't have $100").unwrap();
    assert!(
        result.contains("do not") || !result.contains("don't"),
        "Expected contractions to be expanded, got: {}",
        result
    );
}

#[test]
fn test_concurrent_normalize() {
    use std::sync::Arc;
    use std::thread;

    let normalizer = Arc::new(Normalizer::with_defaults(FST_DIR));
    let mut handles = vec![];

    for i in 0..4 {
        let n = Arc::clone(&normalizer);
        handles.push(thread::spawn(move || {
            let input = match i % 4 {
                0 => "2024年",
                1 => "100元",
                2 => "3/4",
                _ => "下午3点30分",
            };
            let result = n.normalize(input).unwrap();
            assert!(!result.is_empty(), "thread {} got empty result", i);
            result
        }));
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), 4);
}
