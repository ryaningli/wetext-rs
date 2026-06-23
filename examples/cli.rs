use clap::Parser;
use wetext_rs::{Language, Normalizer, NormalizerConfig, Operator};

/// WeText-RS 文本归一化命令行工具
#[derive(Parser)]
#[command(name = "wetext-cli", version, about = "Text Normalization / Inverse Text Normalization CLI")]
struct Cli {
    /// FST 文件目录路径
    #[arg(short, long)]
    fst_dir: String,

    /// 操作模式: tn (文本归一化) 或 itn (逆文本归一化)
    #[arg(short, long, value_enum, default_value = "itn")]
    mode: Mode,

    /// 输入文本
    #[arg(short, long)]
    text: String,

    /// 语言: auto, zh, en, ja
    #[arg(short, long, default_value = "auto")]
    lang: String,
}

#[derive(clap::ValueEnum, Clone)]
enum Mode {
    Tn,
    Itn,
}

fn main() {
    let cli = Cli::parse();

    let operator = match cli.mode {
        Mode::Tn => Operator::Tn,
        Mode::Itn => Operator::Itn,
    };

    let lang = match cli.lang.as_str() {
        "zh" => Language::Zh,
        "en" => Language::En,
        "ja" => Language::Ja,
        _ => Language::Auto,
    };

    let config = NormalizerConfig::new()
        .with_operator(operator)
        .with_lang(lang);

    let normalizer = Normalizer::new(&cli.fst_dir, config);
    match normalizer.normalize(&cli.text) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("错误: {}", e),
    }
}
