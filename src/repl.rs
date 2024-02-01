//! Read-eval-print loop for Lisper

// Copied from https://github.com/rosenbergm/arpar/blob/main/src/repl.rs

use std::borrow::Cow;
use std::collections::HashSet;

use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, Editor, Event, EventContext, EventHandler, KeyEvent, RepeatCount};
use rustyline::{ConditionalEventHandler, Context};
use rustyline_derive::{Completer, Helper, Validator};

use crate::eval::evaluate;
use crate::lexer::lex;
use crate::parser::parse;
use crate::scope::Scope;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Completer, Helper, Validator)]
pub struct CommandHinter {
    pub hints: HashSet<CommandHint>,
}

impl Highlighter for CommandHinter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned("\x1b[2m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    // fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
    //     false
    // }
}

impl Hint for CommandHint {
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

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for CommandHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                // expect hint after word complete, like redis cli, add condition:
                // line.ends_with(" ")
                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

pub fn command_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();

    set.insert(CommandHint::new("exit", "exit"));

    set
}

pub struct TabEventHandler;
impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, _n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));

        if ctx.has_hint() {
            Some(Cmd::CompleteHint)
        } else {
            None
        }
    }
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct CommandHint {
    display: String,
    complete_up_to: usize,
}

pub fn run_repl() -> rustyline::Result<()> {
    println!(
        "
==========  Lisper v{}  ==========

A simple LISP-like interpreter written in Rust as a part of the Programming 1
course at MFF CUNI.

To exit the REPL, type `exit`.
    ",
        VERSION
    );

    let helper = CommandHinter {
        hints: command_hints(),
    };

    let mut editor: Editor<CommandHinter, DefaultHistory> = Editor::new()?;

    editor.set_helper(Some(helper));

    editor.bind_sequence(
        KeyEvent::from('\t'),
        EventHandler::Conditional(Box::new(TabEventHandler)),
    );

    let mut env = Scope::new().wrap();

    loop {
        let line = editor.readline("> ")?.trim().to_string();

        editor.add_history_entry(line.clone())?;

        if line == "exit" {
            return Ok(());
        }

        let tokens = lex(&line);

        match parse(&mut tokens.into_iter().peekable()) {
            Err(parser_error) => {
                println!("PARSER ERROR: {parser_error}");
            }
            Ok(parsed) => {
                let evaluated = evaluate(&parsed, &mut env);

                match evaluated {
                    Ok(result) => println!("{result}"),
                    Err(err) => println!("EVAL ERROR: {err}"),
                }
            }
        }
    }
}
