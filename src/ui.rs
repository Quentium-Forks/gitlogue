use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::animation::AnimationEngine;
use crate::git::{CommitMetadata, GitRepository};
use crate::panes::{EditorPane, FileTreePane, StatusBarPane, TerminalPane};

pub struct UI<'a> {
    should_quit: bool,
    is_commit_specified: bool,
    file_tree: FileTreePane,
    editor: EditorPane,
    terminal: TerminalPane,
    status_bar: StatusBarPane,
    engine: AnimationEngine,
    metadata: Option<CommitMetadata>,
    repo: Option<&'a GitRepository>,
    next_commit_at: Option<Instant>,
}

impl<'a> UI<'a> {
    pub fn new(speed_ms: u64, is_commit_specified: bool, repo: Option<&'a GitRepository>) -> Self {
        Self {
            should_quit: false,
            is_commit_specified,
            file_tree: FileTreePane,
            editor: EditorPane,
            terminal: TerminalPane,
            status_bar: StatusBarPane,
            engine: AnimationEngine::new(speed_ms),
            metadata: None,
            repo,
            next_commit_at: None,
        }
    }

    pub fn load_commit(&mut self, metadata: CommitMetadata) {
        self.engine.load_commit(&metadata);
        self.metadata = Some(metadata);
        self.next_commit_at = None;
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Update viewport height for scroll calculation
            // Main content area height - status bar (3) - borders (2) = editor height
            let size = terminal.size()?;
            let editor_height = size
                .height
                .saturating_sub(3) // Status bar
                .saturating_sub(2); // Main content borders
            let viewport_height = (editor_height as f32 * 0.8) as usize; // 80% for editor
            let viewport_height = viewport_height.saturating_sub(2); // Editor borders
            self.engine.set_viewport_height(viewport_height);

            // Tick the animation engine
            let needs_redraw = self.engine.tick();

            if needs_redraw {
                terminal.draw(|f| self.render(f))?;
            }

            if event::poll(std::time::Duration::from_millis(16))? {
                // ~60fps polling
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }

            // Check if animation finished
            if self.engine.is_finished() {
                if self.is_commit_specified {
                    // Commit was specified - just quit
                    self.should_quit = true;
                } else if let Some(repo) = self.repo {
                    // Random commit mode - schedule next commit load after delay
                    if self.next_commit_at.is_none() {
                        // First time finishing - schedule next commit in 3 seconds
                        self.next_commit_at = Some(Instant::now() + Duration::from_secs(3));
                    } else if Instant::now() >= self.next_commit_at.unwrap() {
                        // Time to load next commit
                        match repo.random_commit() {
                            Ok(metadata) => {
                                self.load_commit(metadata);
                            }
                            Err(_) => {
                                self.should_quit = true;
                            }
                        }
                    }
                } else {
                    // No repo available - quit
                    self.should_quit = true;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn render(&self, f: &mut Frame) {
        let size = f.area();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),      // Main content area
                Constraint::Length(3),   // Status bar
            ])
            .split(size);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),  // Left side (file tree)
                Constraint::Percentage(70),  // Right side (editor + terminal)
            ])
            .split(main_layout[0]);

        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80),  // Editor
                Constraint::Percentage(20),  // Terminal
            ])
            .split(content_layout[1]);

        self.file_tree.render(
            f,
            content_layout[0],
            self.metadata.as_ref(),
            self.engine.current_file_index,
        );
        self.editor.render(f, right_layout[0], &self.engine);
        self.terminal.render(f, right_layout[1], &self.engine);
        self.status_bar
            .render(f, main_layout[1], self.metadata.as_ref());
    }
}
