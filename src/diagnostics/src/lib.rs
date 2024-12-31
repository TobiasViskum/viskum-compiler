use fxhash::FxHashMap;
use ir::{ Delimeter, ItemErrorKind, ModId, ResKind, Symbol, Ty };
use op::BinaryOp;
use span::Span;
use std::{ fmt::Write, path::PathBuf, sync::{ LazyLock, Mutex } };

static DIAGNOSTICS: LazyLock<Mutex<ProgramDiagnostics>> = LazyLock::new(||
    Mutex::new(ProgramDiagnostics::new())
);

pub fn report_diagnostics(diagnostics: Vec<Diagnostic>) {
    DIAGNOSTICS.lock().unwrap().add_diagnostics(diagnostics);
}

pub fn has_error() -> bool {
    DIAGNOSTICS.lock().unwrap().highest_severity.is_some()
}

pub fn print_diagnostics() {
    let diagnostics = DIAGNOSTICS.lock().unwrap();
    let diagnostics = diagnostics.get_diagnostics();

    let errors_vec = diagnostics
        .iter()
        .filter(|x| {
            match x {
                Diagnostic { kind: DiagnosticKind::Error(_), .. } => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    let warnings_vec = diagnostics
        .iter()
        .filter(|x| {
            match x {
                Diagnostic { kind: DiagnosticKind::Warning(_), .. } => true,
                _ => false,
            }
        })
        .collect::<Vec<_>>();

    if !errors_vec.is_empty() {
        println!("\n\x1b[91mErrors:\x1b[0m");
        let mut buffer = String::new();
        for diagnostic in errors_vec {
            diagnostic.write_msg(&mut buffer);
        }
        println!("{}\n", buffer);
    }

    if !warnings_vec.is_empty() {
        println!("\n\x1b[93mWarnings:\x1b[0m");
        let mut buffer = String::new();
        for diagnostic in warnings_vec {
            diagnostic.write_msg(&mut buffer);
        }
        println!("{}\n", buffer);
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

    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    range: Span,
}

impl Diagnostic {
    pub fn new_error(kind: ErrorKind, range: Span) -> Self {
        Self {
            kind: DiagnosticKind::Error(kind),
            range,
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

    pub fn write_msg(&self, buffer: &mut String) {
        match self.kind {
            DiagnosticKind::Error(kind) => kind.write_msg(buffer, &self.range),
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
    MissingCommaBetweenFnArgs {
        arg_symbol_after_missing_comma: Symbol,
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
            Self::MissingCommaBetweenFnArgs { .. } => Severity::Severe,

            Self::FnWithoutBody { .. } => Severity::NoImpact,
            Self::ReturnOutsideFn => Severity::NoImpact,
            Self::MissingReturn => Severity::NoImpact,
            Self::ExpectedBoolExprInCond { .. } => Severity::NoImpact,
            Self::AssignmentToImmutable { .. } => Severity::NoImpact,
            Self::BreakOutsideLoop => Severity::NoImpact,
            Self::ContinueOutsideLoop => Severity::NoImpact,
        }
    }

    /// Used with the LSP to make each error unique
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
            Self::MissingCommaBetweenFnArgs { .. } => "E016",
        }
    }

    pub fn write_msg(&self, buffer: &mut String, span: &Span) {
        let write_error = match self {
            Self::MissingCommaBetweenFnArgs { arg_symbol_after_missing_comma } => {
                write!(
                    buffer,
                    "Missing comma between function arguments at line {}. Try adding a comma before argument `{}`",
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
