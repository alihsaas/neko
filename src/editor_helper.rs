use ansi_term::Colour;
use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hint, Hinter, HistoryHinter},
    validate::{self, MatchingBracketValidator, Validator},
    Context,
};
use rustyline_derive::Helper;
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    cell::RefCell,
};

use crate::{
    interpreter::Interpreter,
    interpreter_option::InterpreterOptions,
};

pub struct OutputHint {
    pub display: String,
    pub complete_up_to: usize,
}

impl Hint for OutputHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl OutputHint {
    fn new(text: &str, complete_up_to: &str) -> Self {
        assert!(text.starts_with(complete_up_to));
        Self {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }
}

#[derive(Helper)]
pub struct EditorHelper {
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub validator: MatchingBracketValidator,
    pub hinter: HistoryHinter,
    pub interpreter: RefCell<Interpreter>,
}

impl Completer for EditorHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for EditorHelper {
    type Hint = OutputHint;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<OutputHint> {
        if line.trim().is_empty() {
            let hint = self.hinter.hint(line, pos, ctx).unwrap_or_default();
            Some(OutputHint::new(&hint, &hint))
        } else {
            match self
                .interpreter
                .borrow_mut()
                .interpret_with_option(line, &InterpreterOptions::all())
            {
                Ok(value) => {
                    let hint = self.hinter.hint(line, pos, ctx).unwrap_or_default();
                    Some(OutputHint::new(
                        &format!("{}\n{}", &hint, &value.stringify()),
                        &hint,
                    ))
                }
                Err(_) => {
                    let hint = self.hinter.hint(line, pos, ctx).unwrap_or_default();
                    Some(OutputHint::new(&hint, &hint))
                }
            }
        }
    }
}

impl Highlighter for EditorHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Owned(format!("{}{}", prompt, Colour::White.suffix().to_string()))
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(Colour::RGB(128, 127, 113).paint(hint).to_string())
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for EditorHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}
