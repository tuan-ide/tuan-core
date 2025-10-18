use convert_case::{Case, Casing};
use oxc_parser::lexer as oxc_lexer;
use oxc_span::SourceType;
use std::path::Path;

pub fn tokenize_file(file_path: &Path) -> Result<Vec<String>, std::io::Error> {
    let source_code = std::fs::read_to_string(&file_path)?;
    let allocator = oxc_allocator::Allocator::default();
    let source_type = match file_path.extension().and_then(|s| s.to_str()) {
        Some("tsx") => SourceType::tsx(),
        Some("ts") => SourceType::ts(),
        Some("jsx") => SourceType::jsx(),
        Some("js") => SourceType::unambiguous().with_jsx(true).with_module(true),
        Some("mjs") => SourceType::mjs(),
        Some("cjs") => SourceType::cjs(),
        _ => SourceType::unambiguous(),
    };

    let mut tokens = vec![];

    let mut lexer = oxc_lexer::Lexer::new_for_benchmarks(&allocator, &source_code, source_type);
    loop {
        let token = lexer.next_token();
        let kind = token.kind();
        if kind.is_eof() {
            break;
        }
        if kind.is_identifier() && !kind.is_any_keyword() {
            let span = token.span();
            let ident = &source_code[span.start as usize..span.end as usize];
            if ident == "undefined" {
                continue;
            }

            ident.to_case(Case::Lower).split_whitespace().for_each(|t| {
                if t.len() >= 3 {
                    tokens.push(t.to_string());
                }
            });
        }
    }

    Ok(tokens)
}
