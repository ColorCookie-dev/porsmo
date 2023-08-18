use crossterm::event::{ KeyCode, KeyEvent, KeyModifiers, Event, KeyEventKind};

pub enum Command {
    Quit,
    Pause,
    Resume,
    Toggle,
    Enter,
    Skip,
    Yes,
    No,
    Invalid,
}

impl From<Event> for Command {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => Command::from(key),
            _ => Command::Invalid,
        }
    }
}

impl From<KeyEvent> for Command {
    fn from(key: KeyEvent) -> Self {
        match key {
            KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Quit,
            KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL, ..
            } => Self::Quit,
            KeyEvent {
                code: KeyCode::Char('z'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL, ..
            } => Self::Quit,
            KeyEvent {
                code: KeyCode::Char(' '),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Toggle,
            KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Enter,
            KeyEvent {
                code: KeyCode::Char('S'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::SHIFT, ..
            } => Self::Skip,
            KeyEvent {
                code: KeyCode::Char('y'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Yes,
            KeyEvent {
                code: KeyCode::Char('n'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::No,
            KeyEvent {
                code: KeyCode::Char('t'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Toggle,
            KeyEvent {
                code: KeyCode::Char('p'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Pause,
            KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE, ..
            } => Self::Resume,
            _ => Self::Invalid,
        }
    }
}
