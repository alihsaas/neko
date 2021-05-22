use crate::editor_helper::EditorHelper;
use rustyline::{
    completion::FilenameCompleter, config::OutputStreamType, highlight::MatchingBracketHighlighter,
    hint::HistoryHinter, validate::MatchingBracketValidator, Cmd, CompletionType, Config, Editor,
    EventHandler, KeyEvent,
};

pub struct Repl {
    pub editor_enabled: bool,
    pub editor: Editor<EditorHelper>,
    pub session_history: Vec<String>,
}

impl Repl {
    pub fn new() -> Self {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let helper = EditorHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
        };

        let mut editor = Editor::with_config(config);
        editor.set_helper(Some(helper));
        Self {
            editor_enabled: false,
            editor,
            session_history: vec![],
        }
    }

    pub fn add_history(&mut self, line: &str) {
        self.session_history.push(line.to_string());
    }

    pub fn bind(&mut self) {
        if self.editor_enabled {
            self.editor
                .bind_sequence(KeyEvent::from('\r'), EventHandler::Simple(Cmd::Newline));
            self.editor
                .bind_sequence(KeyEvent::ctrl('d'), EventHandler::Simple(Cmd::AcceptLine));
        } else {
            self.editor.unbind_sequence(KeyEvent::from('\r'));
            self.editor.unbind_sequence(KeyEvent::ctrl('d'));
        }
    }

    pub fn enable_editor(&mut self) {
        self.editor_enabled = true;
    }
    pub fn disable_editor(&mut self) {
        self.editor_enabled = false;
    }
    pub fn toggle_editor(&mut self) {
        self.editor_enabled = !self.editor_enabled;
        self.bind()
    }
}
