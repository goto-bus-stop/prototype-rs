use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::PathBuf;
use esprit::script;
use esprit::error::Error as EspritError;
use estree_detect_requires::detect;
use quicli::prelude::Result; // TODO use `failure`?
use serde_json;
use sha1::{Sha1, Digest};
use graph::{Hash, SourceFile};

#[derive(Debug)]
pub struct ParseError {
    filename: PathBuf,
    inner: EspritError,
}

impl ParseError {
    fn new(filename: &PathBuf, inner: EspritError) -> ParseError {
        ParseError { filename: filename.clone(), inner }
    }

    fn into_inner(self) -> EspritError {
        self.inner
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position = match self.inner {
            EspritError::UnexpectedToken(ref token) | EspritError::FailedASI(ref token) |
            EspritError::IllegalBreak(ref token) | EspritError::IllegalContinue(ref token) |
            EspritError::DuplicateDefault(ref token) | EspritError::StrictWith(ref token) |
            EspritError::ThrowArgument(ref token) | EspritError::OrphanTry(ref token) =>
                Some(token.location),
            EspritError::TopLevelReturn(ref span) | EspritError::ForOfLetExpr(ref span) |
            EspritError::ContextualKeyword(ref span, _) | EspritError::IllegalStrictBinding(ref span, _) =>
                Some(*span),
            EspritError::InvalidLabel(ref id) | EspritError::InvalidLabelType(ref id) =>
                id.location,
            EspritError::LexError(_) => None,
            EspritError::InvalidLHS(span, _) => span,
            EspritError::UnsupportedFeature(_) => None,
            EspritError::UnexpectedDirective(span, _) => span,
            EspritError::UnexpectedModule(span) => span,
            EspritError::ImportInScript(ref _import) => None, // For now
            EspritError::ExportInScript(ref _export) => None, // For now
            EspritError::CompoundParamWithUseStrict(ref _patt) => None, // For now
        };
        write!(f, "Parse error in {}:{}\n{}", &self.filename.to_string_lossy(), match position {
            Some(span) => format!("{}:{}", span.start.line, span.start.column),
            None => "0:0".into(),
        }, self.description())
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        self.inner.description()
    }
    fn cause(&self) -> Option<&StdError> {
        Some(&self.inner)
    }
}

pub struct LoadFile {
    path: PathBuf,
}

impl LoadFile {
    pub fn new(path: PathBuf) -> Self {
        LoadFile { path }
    }

    pub fn run(self) -> Result<SourceFile> {
        self.read_file()
    }

    fn read_file(self) -> Result<SourceFile> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut source = String::new();
        reader.read_to_string(&mut source)?;

        let hash = Sha1::digest_str(&source) as Hash;

        let is_json = self.path.extension().map_or(false, |ext| ext == "json");
        if is_json {
            let value = serde_json::from_str(&source)?;
            Ok(SourceFile::JSON {
                path: self.path,
                source,
                hash,
                value,
            })
        } else {
            let ast = script(&source)
                .map_err(|e| ParseError::new(&self.path, e))?;
            let dependencies = detect(&ast);
            Ok(SourceFile::CJS {
                path: self.path,
                source,
                hash,
                ast,
                dependencies,
            })
        }
    }
}
