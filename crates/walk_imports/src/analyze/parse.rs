use std::path::Path;

use anyhow::{anyhow, Context, Error};
use swc_core::{
    common::{
        comments::SingleThreadedComments,
        errors::{ColorConfig, Handler},
        sync::Lrc,
        SourceMap,
    },
    ecma::{
        ast::Module,
        parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax},
    },
};
pub fn parse(
    code_map: &Lrc<SourceMap>,
    file: &Path,
) -> Result<(Module, SingleThreadedComments), Error> {
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(code_map.clone()));

    let source_file = code_map
        .load_file(file)
        .with_context(|| format!("Failed to load file {:?}", &file))?;

    let comments = SingleThreadedComments::default();
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*source_file),
        Some(&comments),
    );

    let capturing = Capturing::new(lexer);

    let mut parser = Parser::new_from(capturing);

    for error in parser.take_errors() {
        error.into_diagnostic(&handler).emit();
    }

    let module_result = parser
        .parse_module()
        .map_err(|error| error.into_diagnostic(&handler).emit());

    match module_result {
        Ok(module) => Ok((module, comments)),
        Err(_) => Err(anyhow!("Failed to parse module {:?}", file)),
    }
}
