use fxhash::FxHashMap;
use ir::{
    Delimeter,
    ExpectedSymbolKind,
    ItemErrorKind,
    MissingCommaPlace,
    ModId,
    ResKind,
    Symbol,
    Ty,
};
use op::BinaryOp;
use span::Span;
use token::TokenKind;
use std::{ fmt::Write, path::PathBuf, sync::{ LazyLock, Mutex } };

static DIAGNOSTICS: LazyLock<Mutex<ProgramDiagnostics>> = LazyLock::new(||
    Mutex::new(ProgramDiagnostics::new())
);

pub fn set_mode_id_to_file_path(mod_id: ModId, file_path: PathBuf) {
    DIAGNOSTICS.lock().unwrap().mod_name_to_file_path.insert(mod_id, file_path);
}

pub fn report_diagnostics(diagnostics: Vec<Diagnostic>) {
    DIAGNOSTICS.lock().unwrap().add_diagnostics(diagnostics);
}

pub fn has_error() -> bool {
    DIAGNOSTICS.lock().unwrap().highest_severity.is_some()
}

pub fn print_diagnostics() {
    let mut diagnostics_lock = DIAGNOSTICS.lock().unwrap();
    diagnostics_lock.sort_by_mod_id();
    let diagnostics = diagnostics_lock.get_sorted_diagnostics();

    let mut error_buffer = String::new();
    let mut warning_buffer = String::new();

    let mut mod_id = None;
    let mut file_content = String::new();

    for diagnostic in diagnostics {
        if let Some(mod_id) = mod_id {
            if mod_id != diagnostic.mod_id {
                file_content = diagnostics_lock.load_file_content(mod_id);
            }
        } else {
            mod_id = Some(diagnostic.mod_id);
            file_content = diagnostics_lock.load_file_content(diagnostic.mod_id);
        }

        if let DiagnosticKind::Error(_) = diagnostic.kind {
            diagnostic.write_msg(&mut error_buffer, &file_content);
            writeln!(error_buffer).unwrap();
        } else if let DiagnosticKind::Warning(_) = diagnostic.kind {
            diagnostic.write_msg(&mut warning_buffer, &file_content);
            writeln!(warning_buffer).unwrap();
        }
    }

    if !error_buffer.is_empty() {
        println!("\n\x1b[91mErrors:\x1b[0m\n{}", error_buffer);
    }

    if !warning_buffer.is_empty() {
        println!("\n\x1b[93mWarnings:\x1b[0m\n{}", warning_buffer);
    }
}

pub struct ProgramDiagnostics {
    diagnostics: Vec<Diagnostic>,
    highest_severity: Option<Severity>,
    mod_name_to_file_path: FxHashMap<ModId, PathBuf>,
}

impl ProgramDiagnostics {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            highest_severity: None,
            mod_name_to_file_path: FxHashMap::default(),
        }
    }

    pub fn load_file_content(&self, mod_id: ModId) -> String {
        let file_path = self.mod_name_to_file_path.get(&mod_id).unwrap();
        let file_content = std::fs::read_to_string(file_path).unwrap();
        file_content
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn add_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.extend(
            diagnostics.into_iter().map(|x| {
                match (x.get_severity(), self.highest_severity) {
                    (Some(severity), Some(highest_severity)) if
                        (severity as u8) > (highest_severity as u8)
                    => {
                        self.highest_severity = Some(severity);
                    }
                    (Some(severity), None) => {
                        self.highest_severity = Some(severity);
                    }
                    _ => {}
                }

                x
            })
        );
    }

    pub fn sort_by_mod_id(&mut self) {
        self.diagnostics.sort_by(|a, b| { a.mod_id.0.cmp(&b.mod_id.0) });
    }

    pub fn get_sorted_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    range: Span,
    mod_id: ModId,
}

impl Diagnostic {
    pub fn new_error(kind: ErrorKind, range: Span, mod_id: ModId) -> Self {
        Self {
            kind: DiagnosticKind::Error(kind),
            range,
            mod_id,
        }
    }

    pub fn get_severity_code(&self) -> u8 {
        match self.kind {
            DiagnosticKind::Error(_) => 1,
            DiagnosticKind::Warning(_) => 2,
            DiagnosticKind::Info(_) => 3,
            DiagnosticKind::Hint(_) => 4,
        }
    }

    pub fn get_severity(&self) -> Option<Severity> {
        match self.kind {
            DiagnosticKind::Error(kind) => Some(kind.get_severity()),
            DiagnosticKind::Warning(_) => None,
            DiagnosticKind::Info(_) => None,
            DiagnosticKind::Hint(_) => None,
        }
    }

    pub fn write_msg(&self, buffer: &mut String, file_content: &str) {
        match self.kind {
            DiagnosticKind::Error(kind) => kind.write_msg(buffer, &self.range, file_content),
            DiagnosticKind::Warning(_) => todo!(),
            DiagnosticKind::Info(_) => todo!(),
            DiagnosticKind::Hint(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticKind {
    Error(ErrorKind),
    Warning(WarningKind),
    Info(InfoKind),
    Hint(HintKind),
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    UndefinedLookup {
        symbol: Symbol,
        res_kind: ResKind,
    },
    MismatchedFieldTypes {
        struct_symbol: Symbol,
        field_name: Symbol,
        expected_ty: Ty,
        provided_ty: Ty,
    },
    MissingStructField {
        struct_symbol: Symbol,
        missing_field: Symbol,
    },
    MismatchedReturnTypes {
        expected_ty: Ty,
        provided_ty: Ty,
    },
    ReturnOutsideFn,
    MissingReturn,
    UndefinedStructField {
        struct_symbol: Symbol,
        field_name: Symbol,
    },
    ExpectedBoolExprInCond {
        found_ty: Ty,
    },
    AssignmentToImmutable {
        symbol: Symbol,
    },
    BreakOutsideLoop,
    ContinueOutsideLoop,
    MismatchedTypesInBinaryExpr {
        op: BinaryOp,
        lhs_ty: Ty,
        rhs_ty: Ty,
    },
    ExpectedDelimeterAfter {
        expected_delim: Delimeter,
        kind: ItemErrorKind,
    },
    ExpectedDelimeterBefore {
        expected_delim: Delimeter,
        kind: ItemErrorKind,
    },
    FnWithoutBody {
        symbol: Symbol,
    },
    MissingComma {
        missing_comma_place: MissingCommaPlace,
        arg_symbol_after_missing_comma: Symbol,
    },
    UnexpectedTokens {
        expected_str: &'static str,
    },
    ExpectedIdent {
        additional_info: Option<&'static str>,
        found: Symbol,
    },
    ExpectedToken {
        additional_info: Option<&'static str>,
        expected: TokenKind,
        found: Symbol,
    },
    ExpectedExprOrItem {
        found: Symbol,
    },
}

impl ErrorKind {
    pub fn get_severity(&self) -> Severity {
        match self {
            Self::UndefinedLookup { .. } => Severity::Severe,
            Self::MismatchedFieldTypes { .. } => Severity::Severe,
            Self::MissingStructField { .. } => Severity::Severe,
            Self::MismatchedReturnTypes { .. } => Severity::Severe,
            Self::UndefinedStructField { .. } => Severity::Severe,
            Self::MismatchedTypesInBinaryExpr { .. } => Severity::Severe,
            Self::ExpectedDelimeterAfter { .. } => Severity::Severe,
            Self::ExpectedDelimeterBefore { .. } => Severity::Severe,
            Self::MissingComma { .. } => Severity::Severe,
            Self::UnexpectedTokens { .. } => Severity::Severe,
            Self::ExpectedIdent { .. } => Severity::Severe,
            Self::ExpectedToken { .. } => Severity::Severe,
            Self::ExpectedExprOrItem { .. } => Severity::Severe,

            Self::FnWithoutBody { .. } => Severity::NoImpact,
            Self::ReturnOutsideFn => Severity::NoImpact,
            Self::MissingReturn => Severity::NoImpact,
            Self::ExpectedBoolExprInCond { .. } => Severity::NoImpact,
            Self::AssignmentToImmutable { .. } => Severity::NoImpact,
            Self::BreakOutsideLoop => Severity::NoImpact,
            Self::ContinueOutsideLoop => Severity::NoImpact,
        }
    }

    /// Used with the LSP (in the future) to make each error unique
    pub fn get_issue_code(&self) -> &str {
        match self {
            Self::UndefinedLookup { .. } => "E001",
            Self::MismatchedFieldTypes { .. } => "E002",
            Self::MissingStructField { .. } => "E003",
            Self::MismatchedReturnTypes { .. } => "E004",
            Self::ReturnOutsideFn => "E005",
            Self::MissingReturn => "E006",
            Self::UndefinedStructField { .. } => "E007",
            Self::ExpectedBoolExprInCond { .. } => "E008",
            Self::AssignmentToImmutable { .. } => "E009",
            Self::BreakOutsideLoop => "E010",
            Self::MismatchedTypesInBinaryExpr { .. } => "E011",
            Self::ExpectedDelimeterAfter { .. } => "E012",
            Self::ExpectedDelimeterBefore { .. } => "E013",
            Self::FnWithoutBody { .. } => "E014",
            Self::ContinueOutsideLoop => "E015",
            Self::MissingComma { .. } => "E016",
            Self::UnexpectedTokens { .. } => "E017",
            Self::ExpectedIdent { .. } => "E018",
            Self::ExpectedToken { .. } => "E019",
            Self::ExpectedExprOrItem { .. } => "E020",
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span, file_content: &str) {
        let write_error = match self {
            Self::ExpectedExprOrItem { found } => {
                write!(
                    buffer,
                    "Expected expression or item at line {}. Found `{}`",
                    span.get_line(),
                    found.get()
                )
            }
            Self::ExpectedToken { additional_info, expected, found } => {
                write!(
                    buffer,
                    "Expected `{}`{}at line {}. Found `{}`",
                    expected,
                    match additional_info {
                        Some(additional_info) => format!(" {} ", additional_info),
                        None => "".to_string(),
                    },
                    span.get_line(),
                    found.get()
                )
            }
            Self::ExpectedIdent { additional_info, found } => {
                write!(
                    buffer,
                    "Expected identifier{}at line {}. Found `{}`",
                    match additional_info {
                        Some(additional_info) => format!(" {} ", additional_info),
                        None => " ".to_string(),
                    },
                    span.get_line(),
                    found.get()
                )
            }
            Self::UnexpectedTokens { expected_str } => {
                write!(
                    buffer,
                    "Unexpected tokens at line {}. {}. Found tokens '{}'",
                    span.get_line(),
                    expected_str,
                    &file_content[span.get_byte_range()]
                )
            }
            Self::MissingComma { missing_comma_place, arg_symbol_after_missing_comma } => {
                write!(
                    buffer,
                    "Missing comma between {} at line {}. Try adding a comma before identifier `{}`",
                    missing_comma_place,
                    span.get_line(),
                    arg_symbol_after_missing_comma.get()
                )
            }
            Self::ContinueOutsideLoop => {
                write!(buffer, "Cannot use `continue` outside loop at line {}", span.get_line())
            }
            Self::FnWithoutBody { symbol } => {
                write!(
                    buffer,
                    "Function `{}` is missing a body at line {}",
                    symbol.get(),
                    span.get_line()
                )
            }
            Self::ExpectedDelimeterBefore { expected_delim, kind } => {
                write!(
                    buffer,
                    "Expected `{}` before {} at line {}",
                    expected_delim,
                    kind,
                    span.get_line()
                )
            }
            Self::ExpectedDelimeterAfter { expected_delim, kind } => {
                write!(
                    buffer,
                    "Expected `{}` after {} at line {}",
                    expected_delim,
                    kind,
                    span.get_line()
                )
            }
            Self::MismatchedTypesInBinaryExpr { op, lhs_ty, rhs_ty } => {
                write!(
                    buffer,
                    "Mismatched types in binary expression at line {}. Expected types `{}` and `{}` for operator `{}`, but got types `{}` and `{}`",
                    span.get_line(),
                    lhs_ty,
                    rhs_ty,
                    op,
                    lhs_ty,
                    rhs_ty
                )
            }
            Self::BreakOutsideLoop => {
                write!(buffer, "Cannot use `break` outside loop at line {}", span.get_line())
            }
            Self::AssignmentToImmutable { symbol } => {
                write!(
                    buffer,
                    "Cannot assign to immutable variable `{}` at line {}",
                    symbol.get(),
                    span.get_line()
                )
            }
            Self::ExpectedBoolExprInCond { found_ty } => {
                write!(
                    buffer,
                    "Expected boolean expression in condition at line {}. Found type `{}`",
                    span.get_line(),
                    found_ty
                )
            }
            Self::UndefinedStructField { struct_symbol, field_name } => {
                write!(
                    buffer,
                    "Undefined field `{}` for struct `{}` at line {}",
                    field_name.get(),
                    struct_symbol.get(),
                    span.get_line()
                )
            }
            Self::MissingReturn => {
                write!(buffer, "Missing return statement at line {}", span.get_line())
            }
            Self::ReturnOutsideFn => {
                write!(buffer, "Return outside function at line {}", span.get_line())
            }
            Self::UndefinedLookup { symbol, res_kind } => {
                write!(
                    buffer,
                    "Undefined {} `{}` at line {}",
                    match res_kind {
                        ResKind::Adt => "type",
                        ResKind::Variable => "variable",
                        ResKind::Fn => "function",
                        ResKind::ConstVariable => "constant",
                        ResKind::ConstStr => unreachable!(),
                    },
                    symbol.get(),
                    span.get_line()
                )
            }
            Self::MismatchedFieldTypes { struct_symbol, field_name, expected_ty, provided_ty } => {
                write!(
                    buffer,
                    "Mismatched field types for struct `{}` at line {}. Expected field `{}` to have type `{}`, but got type `{}`",
                    struct_symbol.get(),
                    span.get_line(),
                    field_name.get(),
                    expected_ty,
                    provided_ty
                )
            }
            Self::MissingStructField { struct_symbol, missing_field } => {
                write!(
                    buffer,
                    "Missing field `{}` for struct `{}` at line {}",
                    missing_field.get(),
                    struct_symbol.get(),
                    span.get_line()
                )
            }
            Self::MismatchedReturnTypes { expected_ty, provided_ty } => {
                write!(
                    buffer,
                    "Mismatched return types at line {}. Expected type `{}`, but got type `{}`",
                    span.get_line(),
                    expected_ty,
                    provided_ty
                )
            }
        };

        write_error.expect("Unexpected write error");
    }
}

/// Describes how serious an error is, but is not used in the LSP
#[derive(PartialEq, Clone, Copy)]
pub enum Severity {
    /// Have little or no impact on further analysis, but cannot codegen
    NoImpact = 0,

    /// Can continue the AST validation, but program will not compile into the ICFG
    Severe = 1,

    /// Cannot continue and stops the program immediately
    Fatal = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum WarningKind {}

#[derive(Debug, Clone, Copy)]
pub enum InfoKind {}

#[derive(Debug, Clone, Copy)]
pub enum HintKind {}
