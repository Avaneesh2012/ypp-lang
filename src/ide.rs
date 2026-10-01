// Y++ IDE v2.0 - Full Featured IDE built with egui/eframe
// Features: IntelliSense, Syntax Highlighting, Project Panel, Context Menus, Error Panel

#![windows_subsystem = "windows"]
#![allow(unused_variables, unused_mut, dead_code)]

use eframe::egui;
use egui::{
    Color32, FontId, Key, Pos2, RichText, Sense, TextEdit, Vec2,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;


// ─────────────────────────────────────────────
// AUTOCOMPLETE / INTELLISENSE DATA
// ─────────────────────────────────────────────

#[derive(Clone, Debug)]
struct CompletionItem {
    label: String,
    detail: String,
    insert_text: String,
    kind: CompletionKind,
    documentation: String,
}

#[derive(Clone, Debug, PartialEq)]
enum CompletionKind {
    Keyword,
    Builtin,
    Function,
    Variable,
    Class,
    Snippet,
}

fn get_completions() -> Vec<CompletionItem> {
    vec![
        // Keywords
        CompletionItem {
            label: "Import".into(), detail: "Import statement".into(),
            insert_text: "Import ".into(), kind: CompletionKind::Keyword,
            documentation: "Import a module or package.\nUsage: Import <module> *;".into(),
        },
        CompletionItem {
            label: "PRINT".into(), detail: "Print to console".into(),
            insert_text: "PRINT: ".into(), kind: CompletionKind::Builtin,
            documentation: "Prints a value to the console.\nUsage: PRINT: \"Hello, World!\";".into(),
        },
        CompletionItem {
            label: "NUM".into(), detail: "Declare a number variable".into(),
            insert_text: "NUM ".into(), kind: CompletionKind::Keyword,
            documentation: "Declare a numeric variable.\nUsage: NUM x = 10;".into(),
        },
        CompletionItem {
            label: "STRING".into(), detail: "Declare a string variable".into(),
            insert_text: "STRING ".into(), kind: CompletionKind::Keyword,
            documentation: "Declare a string variable.\nUsage: STRING name = \"Alice\";".into(),
        },
        CompletionItem {
            label: "if".into(), detail: "Conditional statement".into(),
            insert_text: "if ($1) {\n    $2\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Execute code conditionally.\nUsage:\nif (condition) {\n    // code\n}".into(),
        },
        CompletionItem {
            label: "else".into(), detail: "Else branch".into(),
            insert_text: "else {\n    $1\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Else branch of an if statement.".into(),
        },
        CompletionItem {
            label: "while".into(), detail: "While loop".into(),
            insert_text: "while ($1) {\n    $2\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Repeat code while condition is true.\nUsage:\nwhile (condition) {\n    // code\n}".into(),
        },
        CompletionItem {
            label: "for".into(), detail: "For loop".into(),
            insert_text: "for ($1) {\n    $2\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Iterate with a for loop.\nUsage:\nfor (i = 0; i < 10; i++) {\n    // code\n}".into(),
        },
        CompletionItem {
            label: "func".into(), detail: "Define a function".into(),
            insert_text: "func $1($2) {\n    $3\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Define a reusable function.\nUsage:\nfunc myFunc(param1, param2) {\n    // code\n}".into(),
        },
        CompletionItem {
            label: "class".into(), detail: "Define a class".into(),
            insert_text: "Public class $1 {\n    $2\n}".into(), kind: CompletionKind::Keyword,
            documentation: "Define a class with Public visibility.\nUsage:\nPublic class MyClass {\n    // fields and methods\n}".into(),
        },
        CompletionItem {
            label: "EXCEPTION".into(), detail: "Throw/handle an exception".into(),
            insert_text: "EXCEPTION ".into(), kind: CompletionKind::Builtin,
            documentation: "Throw or handle an exception in Y++.".into(),
        },
        CompletionItem {
            label: "CONCAT".into(), detail: "Concatenate strings".into(),
            insert_text: "CONCAT($1, $2)".into(), kind: CompletionKind::Builtin,
            documentation: "Concatenate two or more strings.\nUsage: CONCAT(str1, str2)".into(),
        },
        CompletionItem {
            label: "TRUE".into(), detail: "Boolean true".into(),
            insert_text: "TRUE".into(), kind: CompletionKind::Keyword,
            documentation: "Boolean literal true.".into(),
        },
        CompletionItem {
            label: "FALSE".into(), detail: "Boolean false".into(),
            insert_text: "FALSE".into(), kind: CompletionKind::Keyword,
            documentation: "Boolean literal false.".into(),
        },
        CompletionItem {
            label: "NOT".into(), detail: "Logical NOT".into(),
            insert_text: "NOT ".into(), kind: CompletionKind::Keyword,
            documentation: "Logical negation operator.".into(),
        },
        CompletionItem {
            label: "integer".into(), detail: "Integer type".into(),
            insert_text: "integer".into(), kind: CompletionKind::Keyword,
            documentation: "32-bit integer type in Y++.".into(),
        },
        CompletionItem {
            label: "double".into(), detail: "Double precision float type".into(),
            insert_text: "double".into(), kind: CompletionKind::Keyword,
            documentation: "64-bit floating point type in Y++.".into(),
        },
        CompletionItem {
            label: "smallint".into(), detail: "Small integer type".into(),
            insert_text: "smallint".into(), kind: CompletionKind::Keyword,
            documentation: "16-bit small integer type in Y++.".into(),
        },
        CompletionItem {
            label: "bool".into(), detail: "Boolean type".into(),
            insert_text: "bool".into(), kind: CompletionKind::Keyword,
            documentation: "Boolean type in Y++.".into(),
        },
        CompletionItem {
            label: "global".into(), detail: "Global scope declaration".into(),
            insert_text: "global ".into(), kind: CompletionKind::Keyword,
            documentation: "Declare a variable in global scope.".into(),
        },
        CompletionItem {
            label: "continue".into(), detail: "Continue to next iteration".into(),
            insert_text: "continue;".into(), kind: CompletionKind::Keyword,
            documentation: "Skip to the next loop iteration.".into(),
        },
        CompletionItem {
            label: "new".into(), detail: "Create a new instance".into(),
            insert_text: "new $1($2)".into(), kind: CompletionKind::Keyword,
            documentation: "Instantiate a new class object.\nUsage: new MyClass(args)".into(),
        },
        CompletionItem {
            label: "ycomponents".into(), detail: "Standard library".into(),
            insert_text: "ycomponents".into(), kind: CompletionKind::Class,
            documentation: "The Y++ standard library package.\nUsage: Import ycomponents *;".into(),
        },
        CompletionItem {
            label: "int()".into(), detail: "Cast to int".into(),
            insert_text: "int()".into(), kind: CompletionKind::Function,
            documentation: "Cast a value to integer type.\nUsage: int(value)".into(),
        },
        CompletionItem {
            label: "double()".into(), detail: "Cast to double".into(),
            insert_text: "double()".into(), kind: CompletionKind::Function,
            documentation: "Cast a value to double type.\nUsage: double(value)".into(),
        },
        CompletionItem {
            label: "string()".into(), detail: "Cast to string".into(),
            insert_text: "string()".into(), kind: CompletionKind::Function,
            documentation: "Cast a value to string type.\nUsage: string(value)".into(),
        },
    ]
}

// ─────────────────────────────────────────────
// SYNTAX HIGHLIGHTING
// ─────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum TokenKind {
    Keyword,
    Type,
    Builtin,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Normal,
}

struct HighlightedToken {
    text: String,
    kind: TokenKind,
}

fn highlight_line(line: &str) -> Vec<HighlightedToken> {
    let keywords = [
        "Import", "if", "else", "while", "for", "func", "class", "Public",
        "NOT", "TRUE", "FALSE", "global", "continue", "new", "NEW", "return",
    ];
    let type_keywords = [
        "integer", "double", "smallint", "slong", "schar", "bool", "NUM", "STRING",
    ];
    let builtins = [
        "PRINT", "EXCEPTION", "CONCAT", "int", "string", "stringint",
    ];

    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Comment: \\
        if i + 1 < chars.len() && chars[i] == '\\' && chars[i + 1] == '\\' {
            let rest: String = chars[i..].iter().collect();
            tokens.push(HighlightedToken { text: rest, kind: TokenKind::Comment });
            break;
        }

        // String literal
        if chars[i] == '"' {
            let mut s = String::from('"');
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                s.push(chars[i]);
                i += 1;
            }
            if i < chars.len() { s.push('"'); i += 1; }
            tokens.push(HighlightedToken { text: s, kind: TokenKind::String });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit() || (chars[i] == '-' && i + 1 < chars.len() && chars[i+1].is_ascii_digit()) {
            let mut n = String::new();
            n.push(chars[i]); i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                n.push(chars[i]); i += 1;
            }
            // suffix
            while i < chars.len() && chars[i].is_alphabetic() {
                n.push(chars[i]); i += 1;
            }
            tokens.push(HighlightedToken { text: n, kind: TokenKind::Number });
            continue;
        }

        // Identifier / keyword
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let mut word = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                word.push(chars[i]); i += 1;
            }
            let kind = if keywords.contains(&word.as_str()) {
                TokenKind::Keyword
            } else if type_keywords.contains(&word.as_str()) {
                TokenKind::Type
            } else if builtins.contains(&word.as_str()) {
                TokenKind::Builtin
            } else {
                TokenKind::Normal
            };
            tokens.push(HighlightedToken { text: word, kind });
            continue;
        }

        // Operators
        let op_chars = ['=', '!', '<', '>', '+', '-', '*', '/', '&', '|', '%', '^'];
        if op_chars.contains(&chars[i]) {
            let mut op = String::new();
            while i < chars.len() && op_chars.contains(&chars[i]) {
                op.push(chars[i]); i += 1;
            }
            tokens.push(HighlightedToken { text: op, kind: TokenKind::Operator });
            continue;
        }

        // Punctuation
        let punct_chars = [';', ',', '.', ':', '(', ')', '{', '}', '[', ']'];
        if punct_chars.contains(&chars[i]) {
            tokens.push(HighlightedToken { text: chars[i].to_string(), kind: TokenKind::Punctuation });
            i += 1;
            continue;
        }

        // Anything else
        tokens.push(HighlightedToken { text: chars[i].to_string(), kind: TokenKind::Normal });
        i += 1;
    }

    tokens
}

fn token_color(kind: &TokenKind, dark_mode: bool) -> Color32 {
    if dark_mode {
        match kind {
            TokenKind::Keyword    => Color32::from_rgb(86, 156, 214),   // VS Blue
            TokenKind::Type       => Color32::from_rgb(78, 201, 176),   // VS Teal
            TokenKind::Builtin    => Color32::from_rgb(220, 220, 170),  // VS Yellow
            TokenKind::String     => Color32::from_rgb(206, 145, 120),  // VS Orange-ish
            TokenKind::Number     => Color32::from_rgb(181, 206, 168),  // VS Green
            TokenKind::Comment    => Color32::from_rgb(106, 153, 85),   // VS Green comment
            TokenKind::Operator   => Color32::from_rgb(212, 212, 212),
            TokenKind::Punctuation=> Color32::from_rgb(200, 200, 200),
            TokenKind::Normal     => Color32::from_rgb(220, 220, 220),
        }
    } else {
        match kind {
            TokenKind::Keyword    => Color32::from_rgb(0, 0, 255),
            TokenKind::Type       => Color32::from_rgb(0, 128, 128),
            TokenKind::Builtin    => Color32::from_rgb(128, 0, 128),
            TokenKind::String     => Color32::from_rgb(163, 21, 21),
            TokenKind::Number     => Color32::from_rgb(9, 134, 88),
            TokenKind::Comment    => Color32::from_rgb(0, 128, 0),
            TokenKind::Operator   => Color32::from_rgb(30, 30, 30),
            TokenKind::Punctuation=> Color32::from_rgb(50, 50, 50),
            TokenKind::Normal     => Color32::from_rgb(0, 0, 0),
        }
    }
}

// ─────────────────────────────────────────────
// DIAGNOSTIC (Error / Warning)
// ─────────────────────────────────────────────

#[derive(Clone, Debug)]
enum DiagnosticKind {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
struct Diagnostic {
    line: usize,
    col: usize,
    kind: DiagnosticKind,
    message: String,
    fix_suggestion: Option<String>,
}

fn analyze_code(code: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let trimmed = line.trim();
        let line_no = line_idx + 1;

        // Check for missing semicolons on statements
        if !trimmed.is_empty()
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with('}')
            && !trimmed.starts_with("\\\\")
            && (trimmed.starts_with("PRINT")
                || trimmed.starts_with("NUM")
                || trimmed.starts_with("STRING")
                || trimmed.starts_with("EXCEPTION"))
        {
            diags.push(Diagnostic {
                line: line_no, col: trimmed.len(),
                kind: DiagnosticKind::Error,
                message: "Missing semicolon at end of statement".into(),
                fix_suggestion: Some(format!("{};", trimmed)),
            });
        }

        // Warn on unbalanced braces per line (simple heuristic)
        let open = trimmed.chars().filter(|&c| c == '{').count();
        let close = trimmed.chars().filter(|&c| c == '}').count();
        if open > 1 && close == 0 {
            diags.push(Diagnostic {
                line: line_no, col: 0,
                kind: DiagnosticKind::Warning,
                message: "Multiple opening braces on one line".into(),
                fix_suggestion: None,
            });
        }

        // Warn on TODO/FIXME comments
        if trimmed.contains("TODO") || trimmed.contains("FIXME") {
            diags.push(Diagnostic {
                line: line_no, col: 0,
                kind: DiagnosticKind::Info,
                message: "TODO/FIXME found in code".into(),
                fix_suggestion: None,
            });
        }

        // Warn about using = instead of ==
        if (trimmed.starts_with("if") || trimmed.starts_with("while"))
            && trimmed.contains("= ")
            && !trimmed.contains("==")
        {
            diags.push(Diagnostic {
                line: line_no, col: 0,
                kind: DiagnosticKind::Warning,
                message: "Possible use of assignment (=) inside condition. Did you mean == ?".into(),
                fix_suggestion: None,
            });
        }
    }

    // Check for unbalanced braces
    let open_count: usize = code.chars().filter(|&c| c == '{').count();
    let close_count: usize = code.chars().filter(|&c| c == '}').count();
    if open_count != close_count {
        diags.push(Diagnostic {
            line: 0, col: 0,
            kind: DiagnosticKind::Error,
            message: format!("Unbalanced braces: {} opening, {} closing", open_count, close_count),
            fix_suggestion: None,
        });
    }

    diags
}

// ─────────────────────────────────────────────
// PROJECT / FILE TREE
// ─────────────────────────────────────────────

#[derive(Clone, Debug)]
struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
    expanded: bool,
}

impl FileNode {
    fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        if path.is_dir() {
            let mut children = Vec::new();
            if let Ok(entries) = fs::read_dir(path) {
                let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    b_is_dir.cmp(&a_is_dir).then(a.file_name().cmp(&b.file_name()))
                });
                for entry in entries {
                    let entry_path = entry.path();
                    let entry_name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden/build directories
                    if entry_name.starts_with('.') || entry_name == "target" {
                        continue;
                    }
                    if let Some(node) = FileNode::from_path(&entry_path) {
                        children.push(node);
                    }
                }
            }
            Some(FileNode { name, path: path.to_path_buf(), is_dir: true, children, expanded: true })
        } else {
            Some(FileNode { name, path: path.to_path_buf(), is_dir: false, children: Vec::new(), expanded: false })
        }
    }
}

// ─────────────────────────────────────────────
// EDITOR TAB
// ─────────────────────────────────────────────

#[derive(Clone)]
struct EditorTab {
    title: String,
    path: Option<PathBuf>,
    content: String,
    modified: bool,
    diagnostics: Vec<Diagnostic>,
    scroll_offset: f32,
}

impl EditorTab {
    fn new_empty() -> Self {
        EditorTab {
            title: "Untitled.ypp".into(),
            path: None,
            content: "Import ycomponents *;\n\nPRINT: \"Hello, World!\";\n".into(),
            modified: false,
            diagnostics: Vec::new(),
            scroll_offset: 0.0,
        }
    }

    fn from_path(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let title = path.file_name()?.to_string_lossy().to_string();
        let mut tab = EditorTab {
            title, path: Some(path.to_path_buf()),
            content: content.clone(), modified: false,
            diagnostics: Vec::new(), scroll_offset: 0.0,
        };
        tab.diagnostics = analyze_code(&content);
        Some(tab)
    }

    fn save(&mut self) -> Result<(), String> {
        if let Some(ref p) = self.path {
            fs::write(p, &self.content).map_err(|e| e.to_string())?;
            self.modified = false;
            Ok(())
        } else {
            Err("No path set. Use Save As.".into())
        }
    }
}

// ─────────────────────────────────────────────
// CONTEXT MENU
// ─────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum ContextAction {
    GoToDefinition,
    HowToUse,
    ReformatCode,
    FixError(String),
    FixAllErrors,
    CopyLine,
    CommentOut,
    None,
}

// ─────────────────────────────────────────────
// MAIN IDE APP
// ─────────────────────────────────────────────

struct YppIde {
    // Tabs
    tabs: Vec<EditorTab>,
    active_tab: usize,

    // Project
    project_root: Option<PathBuf>,
    file_tree: Option<FileNode>,
    show_project_panel: bool,

    // Autocomplete
    completions: Vec<CompletionItem>,
    completion_word: String,
    filtered_completions: Vec<CompletionItem>,
    show_completions: bool,
    completion_selected: usize,
    completion_pos: Pos2,

    // Context menu
    context_menu_open: bool,
    context_menu_pos: Pos2,
    context_menu_line: usize,
    context_menu_word: String,

    // Output / Console
    console_output: String,
    show_console: bool,

    // Diagnostics panel
    show_diagnostics: bool,

    // UI State
    dark_mode: bool,
    font_size: f32,
    status_message: String,

    // Hover tooltip
    hover_word: String,
    hover_doc: String,
    hover_pos: Pos2,
    show_hover: bool,

    // New file/project dialogs
    show_new_project_dialog: bool,
    new_project_name: String,
    new_project_path: String,
    show_new_file_dialog: bool,
    new_file_name: String,
    
    // Find/Replace
    show_find: bool,
    find_text: String,
    replace_text: String,
    find_case_sensitive: bool,
}

impl Default for YppIde {
    fn default() -> Self {
        let mut tabs = vec![EditorTab::new_empty()];
        tabs[0].diagnostics = analyze_code(&tabs[0].content);

        // Try to detect the project root
        let project_root = std::env::current_exe()
            .ok()
            .and_then(|mut p| { p.pop(); p.pop(); Some(p) })
            .filter(|p| p.join("Cargo.toml").exists());

        let file_tree = project_root.as_ref().and_then(|r| FileNode::from_path(r));

        YppIde {
            tabs, active_tab: 0,
            project_root, file_tree,
            show_project_panel: true,
            completions: get_completions(),
            completion_word: String::new(),
            filtered_completions: Vec::new(),
            show_completions: false,
            completion_selected: 0,
            completion_pos: Pos2::ZERO,
            context_menu_open: false,
            context_menu_pos: Pos2::ZERO,
            context_menu_line: 0,
            context_menu_word: String::new(),
            console_output: String::new(),
            show_console: true,
            show_diagnostics: true,
            dark_mode: true,
            font_size: 14.0,
            status_message: "Y++ IDE v2.0 Ready".into(),
            hover_word: String::new(),
            hover_doc: String::new(),
            hover_pos: Pos2::ZERO,
            show_hover: false,
            show_new_project_dialog: false,
            new_project_name: String::new(),
            new_project_path: String::new(),
            show_new_file_dialog: false,
            new_file_name: String::new(),
            show_find: false,
            find_text: String::new(),
            replace_text: String::new(),
            find_case_sensitive: false,
        }
    }
}

impl YppIde {
    fn ypp_exe() -> PathBuf {
        if let Ok(mut p) = std::env::current_exe() {
            p.pop();
            let candidate = p.join("ypp.exe");
            if candidate.exists() { return candidate; }
        }
        PathBuf::from("ypp.exe")
    }

    fn run_current(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab) else { return; };
        let code = tab.content.clone();

        let tmp = std::env::temp_dir().join("ypp_ide2_temp.ypp");
        if let Err(e) = fs::write(&tmp, &code) {
            self.console_output = format!("Failed to write temp file: {}", e);
            return;
        }

        match Command::new(Self::ypp_exe()).arg(&tmp).output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let combined = format!("{}{}", stdout, stderr);
                self.console_output = if combined.trim().is_empty() { "(no output)".into() } else { combined };
            }
            Err(e) => {
                self.console_output = format!(
                    "Failed to run Y++ interpreter:\n{}\n\nMake sure ypp.exe is accessible.",
                    e
                );
            }
        }
        self.show_console = true;
        self.status_message = "Code executed.".into();
    }

    fn save_current(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            match tab.save() {
                Ok(_) => self.status_message = "File saved.".into(),
                Err(e) => self.status_message = format!("Save failed: {}", e),
            }
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        // Check if already open
        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.path.as_ref() == Some(&path) {
                self.active_tab = i;
                return;
            }
        }
        if let Some(tab) = EditorTab::from_path(&path) {
            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
        }
    }

    fn new_file(&mut self) {
        self.tabs.push(EditorTab::new_empty());
        self.active_tab = self.tabs.len() - 1;
    }

    fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() > 1 {
            self.tabs.remove(idx);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    fn update_completions(&mut self, word: &str) {
        if word.is_empty() {
            self.show_completions = false;
            return;
        }
        let word_lower = word.to_lowercase();
        self.filtered_completions = self.completions.iter()
            .filter(|c| c.label.to_lowercase().starts_with(&word_lower))
            .cloned()
            .collect();
        self.show_completions = !self.filtered_completions.is_empty();
        self.completion_selected = 0;
        self.completion_word = word.to_string();
    }

    fn apply_completion(&mut self, item: CompletionItem) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let word = &self.completion_word.clone();
            // Replace the current word with the completion
            if let Some(pos) = tab.content.rfind(word.as_str()) {
                // Simple replace of the last occurrence before cursor
                let insert = item.insert_text.replace("$1", "").replace("$2", "").replace("$3", "");
                tab.content.replace_range(pos..pos + word.len(), &insert);
                tab.modified = true;
                tab.diagnostics = analyze_code(&tab.content);
            }
        }
        self.show_completions = false;
    }

    fn reformat_code(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let mut result = String::new();
            let mut indent = 0usize;
            for line in tab.content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('}') && indent > 0 {
                    indent -= 1;
                }
                if !trimmed.is_empty() {
                    result.push_str(&"    ".repeat(indent));
                    result.push_str(trimmed);
                    result.push('\n');
                } else {
                    result.push('\n');
                }
                if trimmed.ends_with('{') {
                    indent += 1;
                }
            }
            tab.content = result;
            tab.modified = true;
            tab.diagnostics = analyze_code(&tab.content);
            self.status_message = "Code reformatted.".into();
        }
    }

    fn get_word_at_line(&self, line_idx: usize) -> String {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            if let Some(line) = tab.content.lines().nth(line_idx) {
                // Extract a word from the middle of the line (heuristic)
                let words: Vec<&str> = line.split_whitespace().collect();
                return words.first().unwrap_or(&"").to_string();
            }
        }
        String::new()
    }

    fn find_completion_doc(&self, word: &str) -> Option<&CompletionItem> {
        self.completions.iter().find(|c| c.label == word || c.insert_text.starts_with(word))
    }

    fn refresh_file_tree(&mut self) {
        if let Some(ref root) = self.project_root.clone() {
            self.file_tree = FileNode::from_path(root);
        }
    }

    fn open_project(&mut self, path: PathBuf) {
        self.project_root = Some(path.clone());
        self.file_tree = FileNode::from_path(&path);
        self.status_message = format!("Opened project: {}", path.display());
    }
}

impl eframe::App for YppIde {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(Key::S) && i.modifiers.ctrl {
                // Save
            }
            if i.key_pressed(Key::N) && i.modifiers.ctrl {
                // New file
            }
            if i.key_pressed(Key::F) && i.modifiers.ctrl {
                // Find
            }
            if i.key_pressed(Key::F5) {
                // Run
            }
        });

        let save_requested = ctx.input(|i| i.key_pressed(Key::S) && i.modifiers.ctrl);
        let new_requested = ctx.input(|i| i.key_pressed(Key::N) && i.modifiers.ctrl);
        let find_requested = ctx.input(|i| i.key_pressed(Key::F) && i.modifiers.ctrl);
        let run_requested = ctx.input(|i| i.key_pressed(Key::F5));

        if save_requested { self.save_current(); }
        if new_requested { self.new_file(); }
        if find_requested { self.show_find = !self.show_find; }
        if run_requested { self.run_current(); }

        // ── MENU BAR ──────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("📁 File", |ui| {
                    if ui.button("🆕 New File  (Ctrl+N)").clicked() {
                        self.new_file(); ui.close_menu();
                    }
                    if ui.button("📂 Open File...").clicked() {
                        // File picker via rfd
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Y++ Files", &["ypp"])
                            .add_filter("All Files", &["*"])
                            .pick_file()
                        {
                            self.open_file(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("📁 Open Project Folder...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.open_project(path);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("💾 Save  (Ctrl+S)").clicked() {
                        self.save_current(); ui.close_menu();
                    }
                    if ui.button("💾 Save As...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Y++ Files", &["ypp"])
                            .save_file()
                        {
                            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                tab.path = Some(path.clone());
                                tab.title = path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                let _ = tab.save();
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Close Tab").clicked() {
                        let idx = self.active_tab;
                        self.close_tab(idx);
                        ui.close_menu();
                    }
                });

                // Edit menu
                ui.menu_button("✏️ Edit", |ui| {
                    if ui.button("🔍 Find / Replace  (Ctrl+F)").clicked() {
                        self.show_find = !self.show_find; ui.close_menu();
                    }
                    if ui.button("🎨 Reformat Code").clicked() {
                        self.reformat_code(); ui.close_menu();
                    }
                });

                // View menu
                ui.menu_button("👁 View", |ui| {
                    ui.checkbox(&mut self.show_project_panel, "Project Panel");
                    ui.checkbox(&mut self.show_console, "Console Output");
                    ui.checkbox(&mut self.show_diagnostics, "Problems Panel");
                    ui.separator();
                    if self.dark_mode {
                        if ui.button("☀️ Light Mode").clicked() {
                            self.dark_mode = false; ui.close_menu();
                        }
                    } else {
                        if ui.button("🌙 Dark Mode").clicked() {
                            self.dark_mode = true; ui.close_menu();
                        }
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=28.0));
                    });
                });

                // Run menu
                ui.menu_button("▶ Run", |ui| {
                    if ui.button("▶ Run  (F5)").clicked() {
                        self.run_current(); ui.close_menu();
                    }
                    if ui.button("📋 Clear Console").clicked() {
                        self.console_output.clear(); ui.close_menu();
                    }
                });

                // Help menu
                ui.menu_button("❓ Help", |ui| {
                    if ui.button("📖 Y++ Language Reference").clicked() {
                        // Show a help dialog inline
                        self.console_output = YHELP_REFERENCE.to_string();
                        self.show_console = true;
                        ui.close_menu();
                    }
                    if ui.button("ℹ About Y++ IDE").clicked() {
                        self.console_output = "Y++ IDE v2.0\nBuilt with Rust + egui\nFeatures: IntelliSense, Syntax Highlighting, Projects, Diagnostics".to_string();
                        self.show_console = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let run_btn = ui.add(
                        egui::Button::new(RichText::new("▶ Run").color(Color32::from_rgb(78, 201, 100)).strong())
                    );
                    if run_btn.clicked() { self.run_current(); }

                    if self.dark_mode {
                        if ui.button("☀️").clicked() { self.dark_mode = false; }
                    } else {
                        if ui.button("🌙").clicked() { self.dark_mode = true; }
                    }
                });
            });
        });

        // ── STATUS BAR ────────────────────────────────────
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status_message).small());
                ui.separator();
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    let errors = tab.diagnostics.iter().filter(|d| matches!(d.kind, DiagnosticKind::Error)).count();
                    let warnings = tab.diagnostics.iter().filter(|d| matches!(d.kind, DiagnosticKind::Warning)).count();
                    if errors > 0 {
                        ui.label(RichText::new(format!("✖ {} error(s)", errors)).color(Color32::RED).small());
                    }
                    if warnings > 0 {
                        ui.label(RichText::new(format!("⚠ {} warning(s)", warnings)).color(Color32::YELLOW).small());
                    }
                    if errors == 0 && warnings == 0 {
                        ui.label(RichText::new("✔ No problems").color(Color32::GREEN).small());
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.dark_mode {
                        ui.label(RichText::new("Dark").small());
                    } else {
                        ui.label(RichText::new("Light").small());
                    }
                    ui.label(RichText::new(format!("Font: {}px", self.font_size as i32)).small());
                });
            });
        });

        // ── PROJECT PANEL (LEFT SIDEBAR) ─────────────────
        if self.show_project_panel {
            egui::SidePanel::left("project_panel")
                .min_width(180.0)
                .max_width(320.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("📁 Explorer");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🔄").on_hover_text("Refresh").clicked() {
                                self.refresh_file_tree();
                            }
                            if ui.small_button("📂").on_hover_text("Open Folder").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.open_project(path);
                                }
                            }
                            if ui.small_button("🆕").on_hover_text("New File").clicked() {
                                self.show_new_file_dialog = true;
                            }
                        });
                    });
                    ui.separator();

                    if let Some(ref root) = self.project_root.clone() {
                        ui.label(RichText::new(
                            root.file_name().unwrap_or_default().to_string_lossy().as_ref()
                        ).strong().color(Color32::from_rgb(86, 156, 214)));
                    } else {
                        ui.label(RichText::new("No project open").italics().weak());
                    }

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(ref mut tree) = self.file_tree.clone() {
                            let open_path = render_file_tree(ui, tree, 0);
                            if let Some(p) = open_path {
                                if !p.is_dir() {
                                    self.open_file(p);
                                }
                            }
                            // Update the tree (for expand/collapse state)
                            self.file_tree = Some(tree.clone());
                        }
                    });
                });
        }

        // ── BOTTOM PANELS: CONSOLE + DIAGNOSTICS ─────────
        let bottom_panel_height = if self.show_console || self.show_diagnostics { 200.0 } else { 0.0 };

        if bottom_panel_height > 0.0 {
            egui::TopBottomPanel::bottom("bottom_panel")
                .min_height(bottom_panel_height)
                .max_height(400.0)
                .show(ctx, |ui| {
                    // Tabs for console/diagnostics
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.show_console && !self.show_diagnostics, "▶ Output").clicked() {
                            self.show_console = true;
                            self.show_diagnostics = false;
                        }
                        if ui.selectable_label(self.show_diagnostics, "⚠ Problems").clicked() {
                            self.show_diagnostics = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✖").on_hover_text("Close panel").clicked() {
                                self.show_console = false;
                                self.show_diagnostics = false;
                            }
                            if ui.small_button("🗑").on_hover_text("Clear").clicked() {
                                self.console_output.clear();
                            }
                        });
                    });
                    ui.separator();

                    if self.show_diagnostics {
                        // Problems panel
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(tab) = self.tabs.get(self.active_tab) {
                                if tab.diagnostics.is_empty() {
                                    ui.label(RichText::new("✔ No problems detected").color(Color32::GREEN));
                                } else {
                                    for diag in &tab.diagnostics.clone() {
                                        ui.horizontal(|ui| {
                                            let (icon, color) = match diag.kind {
                                                DiagnosticKind::Error   => ("✖", Color32::RED),
                                                DiagnosticKind::Warning => ("⚠", Color32::YELLOW),
                                                DiagnosticKind::Info    => ("ℹ", Color32::from_rgb(86, 156, 214)),
                                            };
                                            ui.label(RichText::new(icon).color(color));
                                            if diag.line > 0 {
                                                ui.label(RichText::new(format!("Line {}: ", diag.line)).weak());
                                            }
                                            ui.label(&diag.message);
                                            if let Some(ref fix) = diag.fix_suggestion {
                                                if ui.small_button("💡 Fix").on_hover_text(fix).clicked() {
                                                    // Apply fix
                                                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                                        let line_idx = diag.line.saturating_sub(1);
                                                        let lines: Vec<&str> = tab.content.lines().collect();
                                                        if line_idx < lines.len() {
                                                            let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
                                                            new_lines[line_idx] = fix.clone();
                                                            tab.content = new_lines.join("\n");
                                                            tab.modified = true;
                                                            tab.diagnostics = analyze_code(&tab.content);
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        });
                    } else {
                        // Console output
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let output_color = if self.dark_mode {
                                    Color32::from_rgb(200, 230, 200)
                                } else {
                                    Color32::from_rgb(0, 80, 0)
                                };
                                ui.add(
                                    TextEdit::multiline(&mut self.console_output.clone())
                                        .font(FontId::monospace(self.font_size - 2.0))
                                        .text_color(output_color)
                                        .desired_width(f32::INFINITY)
                                );
                            });
                    }
                });
        }

        // ── MAIN EDITOR AREA ──────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                let tab_count = self.tabs.len();
                for i in 0..tab_count {
                    let tab = &self.tabs[i];
                    let label = if tab.modified {
                        format!("● {} ✕", tab.title)
                    } else {
                        format!("{} ✕", tab.title)
                    };
                    let is_active = i == self.active_tab;
                    let tab_btn = ui.selectable_label(is_active, &label);
                    if tab_btn.clicked() {
                        self.active_tab = i;
                    }
                    if tab_btn.middle_clicked() {
                        self.close_tab(i);
                        break;
                    }
                }
                if ui.small_button("+").on_hover_text("New Tab").clicked() {
                    self.new_file();
                }
            });
            ui.separator();

            // Find/Replace bar
            if self.show_find {
                ui.horizontal(|ui| {
                    ui.label("🔍 Find:");
                    let find_changed = ui.text_edit_singleline(&mut self.find_text).changed();
                    ui.label("Replace:");
                    ui.text_edit_singleline(&mut self.replace_text);
                    ui.checkbox(&mut self.find_case_sensitive, "Aa");
                    if ui.button("Replace All").clicked() {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            if self.find_case_sensitive {
                                tab.content = tab.content.replace(&self.find_text, &self.replace_text);
                            } else {
                                // Case insensitive replace
                                let lower_content = tab.content.to_lowercase();
                                let lower_find = self.find_text.to_lowercase();
                                let mut result = String::new();
                                let mut last = 0;
                                for (idx, _) in lower_content.match_indices(&lower_find) {
                                    result.push_str(&tab.content[last..idx]);
                                    result.push_str(&self.replace_text);
                                    last = idx + self.find_text.len();
                                }
                                result.push_str(&tab.content[last..]);
                                tab.content = result;
                            }
                            tab.modified = true;
                            tab.diagnostics = analyze_code(&tab.content);
                        }
                    }
                    if ui.button("✕").clicked() { self.show_find = false; }
                });
                ui.separator();
            }

            // Editor with line numbers and syntax highlighting
            let available = ui.available_size();

            // We'll use a custom editor approach:
            // Split the area into line numbers (left) and editor (right)
            ui.horizontal_top(|ui| {
                // Line numbers panel
                let line_no_width = 50.0;
                let content_clone = self.tabs.get(self.active_tab)
                    .map(|t| t.content.clone())
                    .unwrap_or_default();
                let line_count = content_clone.lines().count().max(1);

                // Line numbers
                let (line_no_rect, _) = ui.allocate_exact_size(
                    Vec2::new(line_no_width, available.y),
                    Sense::hover(),
                );

                let line_height = self.font_size + 4.0;
                let painter = ui.painter();
                let bg_color = if self.dark_mode {
                    Color32::from_rgb(30, 30, 30)
                } else {
                    Color32::from_rgb(240, 240, 240)
                };
                painter.rect_filled(line_no_rect, 0.0, bg_color);

                let diag_lines: std::collections::HashSet<usize> = self.tabs.get(self.active_tab)
                    .map(|t| t.diagnostics.iter().map(|d| d.line).collect())
                    .unwrap_or_default();

                for i in 0..line_count {
                    let y = line_no_rect.top() + (i as f32) * line_height + 2.0;
                    let line_no = i + 1;
                    let line_color = if diag_lines.contains(&line_no) {
                        Color32::from_rgb(220, 80, 80)
                    } else if self.dark_mode {
                        Color32::from_rgb(100, 100, 100)
                    } else {
                        Color32::from_rgb(140, 140, 140)
                    };
                    painter.text(
                        Pos2::new(line_no_rect.right() - 5.0, y),
                        egui::Align2::RIGHT_TOP,
                        format!("{}", line_no),
                        FontId::monospace(self.font_size - 2.0),
                        line_color,
                    );
                }

                // The actual text editor
                ui.vertical(|ui| {
                    let editor_area = ui.available_size();

                    egui::ScrollArea::vertical()
                        .id_source("editor_scroll")
                        .show(ui, |ui| {
                            // Get the tab content
                            let content_before = self.tabs.get(self.active_tab)
                                .map(|t| t.content.clone())
                                .unwrap_or_default();

                            let mut content = content_before.clone();

                            let text_edit = ui.add(
                                TextEdit::multiline(&mut content)
                                    .font(FontId::monospace(self.font_size))
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(50)
                                    .code_editor()
                                    .lock_focus(false)
                            );

                            // Update on change
                            if content != content_before {
                                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                    tab.modified = true;
                                    tab.diagnostics = analyze_code(&content);
                                    tab.content = content.clone();
                                }

                                // Update autocomplete
                                // Get the word being typed (last word before cursor)
                                let word = content.split(|c: char| !c.is_alphanumeric() && c != '_')
                                    .last()
                                    .unwrap_or("")
                                    .to_string();
                                if !word.is_empty() {
                                    self.completion_pos = text_edit.rect.left_bottom();
                                    self.update_completions(&word);
                                } else {
                                    self.show_completions = false;
                                }
                            }

                            // Right-click context menu
                            text_edit.context_menu(|ui| {
                                ui.set_min_width(200.0);
                                ui.label(RichText::new("Y++ Actions").strong());
                                ui.separator();

                                if ui.button("📖 How to Use (selected)").clicked() {
                                    let word = self.context_menu_word.clone();
                                    if let Some(item) = self.find_completion_doc(&word).cloned() {
                                        self.console_output = format!(
                                            "📖 {} — {}\n\n{}", item.label, item.detail, item.documentation
                                        );
                                        self.show_console = true;
                                    } else {
                                        self.console_output = format!("No documentation found for '{}'", word);
                                        self.show_console = true;
                                    }
                                    ui.close_menu();
                                }

                                if ui.button("🎨 Reformat Code").clicked() {
                                    self.reformat_code();
                                    ui.close_menu();
                                }

                                ui.separator();
                                ui.label(RichText::new("⚠ Problems on this file").weak().small());

                                let diags = self.tabs.get(self.active_tab)
                                    .map(|t| t.diagnostics.clone())
                                    .unwrap_or_default();

                                if diags.is_empty() {
                                    ui.label(RichText::new("✔ No problems").color(Color32::GREEN).small());
                                } else {
                                    for diag in &diags {
                                        let (icon, color) = match diag.kind {
                                            DiagnosticKind::Error   => ("✖", Color32::RED),
                                            DiagnosticKind::Warning => ("⚠", Color32::YELLOW),
                                            DiagnosticKind::Info    => ("ℹ", Color32::from_rgb(86, 156, 214)),
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(icon).color(color).small());
                                            ui.label(RichText::new(&diag.message).small());
                                            if let Some(ref fix) = diag.fix_suggestion.clone() {
                                                let fix_clone = fix.clone();
                                                let line_idx = diag.line.saturating_sub(1);
                                                if ui.small_button("💡 Fix").clicked() {
                                                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                                        let lines: Vec<&str> = tab.content.lines().collect();
                                                        if line_idx < lines.len() {
                                                            let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
                                                            new_lines[line_idx] = fix_clone;
                                                            tab.content = new_lines.join("\n");
                                                            tab.modified = true;
                                                            tab.diagnostics = analyze_code(&tab.content);
                                                        }
                                                    }
                                                    ui.close_menu();
                                                }
                                            }
                                        });
                                    }

                                    ui.separator();
                                    if ui.button("💡 Fix All Auto-fixable Errors").clicked() {
                                        let diags_clone = diags.clone();
                                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                            let lines: Vec<String> = tab.content.lines().map(|l| l.to_string()).collect();
                                            let mut new_lines = lines.clone();
                                            for diag in &diags_clone {
                                                if let Some(ref fix) = diag.fix_suggestion {
                                                    let line_idx = diag.line.saturating_sub(1);
                                                    if line_idx < new_lines.len() {
                                                        new_lines[line_idx] = fix.clone();
                                                    }
                                                }
                                            }
                                            tab.content = new_lines.join("\n");
                                            tab.modified = true;
                                            tab.diagnostics = analyze_code(&tab.content);
                                        }
                                        ui.close_menu();
                                    }
                                }

                                ui.separator();
                                if ui.button("📋 Copy All").clicked() {
                                    let content = self.tabs.get(self.active_tab)
                                        .map(|t| t.content.clone())
                                        .unwrap_or_default();
                                    ui.ctx().copy_text(content);
                                    ui.close_menu();
                                }
                                if ui.button("▶ Run File (F5)").clicked() {
                                    self.run_current();
                                    ui.close_menu();
                                }
                            });
                        });
                });
            });
        });

        // ── AUTOCOMPLETE POPUP ────────────────────────────
        if self.show_completions && !self.filtered_completions.is_empty() {
            let completions = self.filtered_completions.clone();
            let pos = self.completion_pos;

            egui::Area::new("autocomplete".into())
                .order(egui::Order::Foreground)
                .fixed_pos(pos)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_max_width(350.0);
                        ui.set_max_height(250.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (i, item) in completions.iter().enumerate() {
                                let is_selected = i == self.completion_selected;
                                let (icon, color) = match item.kind {
                                    CompletionKind::Keyword  => ("🔑", Color32::from_rgb(86, 156, 214)),
                                    CompletionKind::Builtin  => ("⚡", Color32::from_rgb(220, 220, 170)),
                                    CompletionKind::Function => ("🔧", Color32::from_rgb(220, 170, 110)),
                                    CompletionKind::Variable => ("📦", Color32::from_rgb(180, 180, 220)),
                                    CompletionKind::Class    => ("🏛", Color32::from_rgb(78, 201, 176)),
                                    CompletionKind::Snippet  => ("📝", Color32::from_rgb(180, 180, 180)),
                                };

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(icon).color(color).small());
                                    let row = ui.selectable_label(
                                        is_selected,
                                        RichText::new(format!("{} — {}", item.label, item.detail)).small()
                                    );
                                    if row.clicked() {
                                        // Apply completion
                                        let item_clone = item.clone();
                                        let word = self.completion_word.clone();
                                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                                            let insert = item_clone.insert_text
                                                .replace("$1", "").replace("$2", "").replace("$3", "");
                                            if let Some(pos) = tab.content.rfind(&word) {
                                                tab.content.replace_range(pos..pos + word.len(), &insert);
                                                tab.modified = true;
                                                tab.diagnostics = analyze_code(&tab.content);
                                            }
                                        }
                                        self.show_completions = false;
                                    }
                                    if row.hovered() {
                                        self.completion_selected = i;
                                        // Show doc tooltip
                                        row.show_tooltip_text(&item.documentation);
                                    }
                                });
                            }
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("↑↓ navigate  Tab/Enter: accept  Esc: dismiss").weak().small());
                        });
                    });
                });

            // Handle keyboard navigation
            let (tab_pressed, enter_pressed, esc_pressed, up_pressed, down_pressed) = ctx.input(|i| (
                i.key_pressed(Key::Tab),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::ArrowDown),
            ));

            if esc_pressed { self.show_completions = false; }
            if up_pressed && self.completion_selected > 0 { self.completion_selected -= 1; }
            if down_pressed && self.completion_selected + 1 < self.filtered_completions.len() {
                self.completion_selected += 1;
            }
            if tab_pressed || enter_pressed {
                if let Some(item) = self.filtered_completions.get(self.completion_selected).cloned() {
                    let word = self.completion_word.clone();
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        let insert = item.insert_text
                            .replace("$1", "").replace("$2", "").replace("$3", "");
                        if let Some(pos) = tab.content.rfind(&word) {
                            tab.content.replace_range(pos..pos + word.len(), &insert);
                            tab.modified = true;
                            tab.diagnostics = analyze_code(&tab.content);
                        }
                    }
                    self.show_completions = false;
                }
            }
        }

        // ── NEW FILE DIALOG ───────────────────────────────
        if self.show_new_file_dialog {
            egui::Window::new("New File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("File name:");
                    ui.text_edit_singleline(&mut self.new_file_name);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() && !self.new_file_name.is_empty() {
                            let name = self.new_file_name.clone();
                            if let Some(ref root) = self.project_root.clone() {
                                let path = root.join(&name);
                                let _ = fs::write(&path, "Import ycomponents *;\n\n");
                                self.open_file(path);
                                self.refresh_file_tree();
                            } else {
                                let mut tab = EditorTab::new_empty();
                                tab.title = name.clone();
                                self.tabs.push(tab);
                                self.active_tab = self.tabs.len() - 1;
                            }
                            self.new_file_name.clear();
                            self.show_new_file_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_file_dialog = false;
                        }
                    });
                });
        }
    }
}

// ─────────────────────────────────────────────
// FILE TREE RENDERER
// ─────────────────────────────────────────────

fn render_file_tree(ui: &mut egui::Ui, node: &mut FileNode, depth: usize) -> Option<PathBuf> {
    let indent = depth as f32 * 14.0;
    let mut open_path = None;

    ui.horizontal(|ui| {
        ui.add_space(indent);
        if node.is_dir {
            let arrow = if node.expanded { "▼" } else { "▶" };
            let folder_icon = if node.expanded { "📂" } else { "📁" };
            if ui.selectable_label(false, format!("{} {} {}", arrow, folder_icon, node.name)).clicked() {
                node.expanded = !node.expanded;
            }
        } else {
            let file_icon = if node.name.ends_with(".ypp") { "📄" }
                else if node.name.ends_with(".toml") { "⚙" }
                else if node.name.ends_with(".md") { "📝" }
                else if node.name.ends_with(".rs") { "🦀" }
                else { "📄" };

            let color = if node.name.ends_with(".ypp") {
                Color32::from_rgb(86, 156, 214)
            } else {
                Color32::GRAY
            };

            let btn = ui.selectable_label(
                false,
                RichText::new(format!("{} {}", file_icon, node.name)).color(color)
            );
            if btn.clicked() {
                open_path = Some(node.path.clone());
            }
        }
    });

    if node.is_dir && node.expanded {
        for child in node.children.iter_mut() {
            if let Some(p) = render_file_tree(ui, child, depth + 1) {
                open_path = Some(p);
            }
        }
    }

    open_path
}

// ─────────────────────────────────────────────
// LANGUAGE REFERENCE HELP TEXT
// ─────────────────────────────────────────────

const YHELP_REFERENCE: &str = r#"
╔══════════════════════════════════════════════╗
║           Y++ Language Reference             ║
╚══════════════════════════════════════════════╝

▸ IMPORT
  Import ycomponents *;

▸ VARIABLES
  NUM x = 42;
  STRING name = "Alice";
  integer count = 0;
  double pi = 3.14;
  bool flag = TRUE;

▸ PRINT
  PRINT: "Hello, World!";
  PRINT: x;

▸ CONDITIONALS
  if (x > 10) {
      PRINT: "big";
  } else {
      PRINT: "small";
  }

▸ LOOPS
  while (x > 0) {
      x--;
  }
  for (i = 0; i < 10; i++) {
      PRINT: i;
  }

▸ FUNCTIONS
  func greet(name) {
      PRINT: CONCAT("Hello, ", name);
  }
  greet("World");

▸ CLASSES
  Public class Animal {
      func speak() {
          PRINT: "...";
      }
  }
  NUM a = new Animal();

▸ TYPE CASTS
  int(value)     — cast to integer
  double(value)  — cast to double
  string(value)  — cast to string

▸ EXCEPTIONS
  EXCEPTION "Something went wrong";

▸ COMMENTS
  \\ This is a comment \\

▸ OPERATORS
  + - * /   arithmetic
  == != < > <= >=  comparison
  NOT  logical not
  ++  --  increment/decrement
"#;

// ─────────────────────────────────────────────
// ENTRY POINT
// ─────────────────────────────────────────────

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Y++ IDE v2.0")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Y++ IDE v2.0",
        native_options,
        Box::new(|_cc| Ok(Box::new(YppIde::default()))),
    ).expect("Failed to launch Y++ IDE");
}
