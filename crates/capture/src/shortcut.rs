use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    canonical: String,
    modifiers: Vec<&'static str>,
    key: String,
}

impl Shortcut {
    pub fn canonical(&self) -> &str {
        &self.canonical
    }
    pub fn display_macos(&self) -> String {
        let symbols = self
            .modifiers
            .iter()
            .map(|modifier| match *modifier {
                "Meta" => "⌘",
                "Alt" => "⌥",
                "Control" => "⌃",
                "Shift" => "⇧",
                "Space" => "Space",
                _ => "",
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{symbols} {}",
            if self.key == "Space" {
                "Space"
            } else {
                &self.key
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutError {
    #[error("shortcut requires a modifier")]
    ModifierRequired,
    #[error("shortcut is reserved")]
    Reserved,
    #[error("shortcut has an invalid key combination")]
    Invalid,
    #[error("shortcut unavailable")]
    Unavailable,
}

pub fn parse_shortcut(input: &str) -> Result<Shortcut, ShortcutError> {
    let mut modifiers = Vec::new();
    let mut key = None;
    for part in input
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let modifier = match part.to_ascii_lowercase().as_str() {
            "meta" | "command" | "cmd" => Some("Meta"),
            "alt" | "option" => Some("Alt"),
            "control" | "ctrl" => Some("Control"),
            "shift" => Some("Shift"),
            "space" => Some("Space"),
            _ => None,
        };
        if let Some(modifier) = modifier {
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        } else if key.is_none() {
            key = Some(part.to_ascii_uppercase());
        } else {
            return Err(ShortcutError::Invalid);
        }
    }
    let key = key.ok_or(ShortcutError::Invalid)?;
    if modifiers.is_empty() {
        return Err(ShortcutError::ModifierRequired);
    }
    if modifiers == ["Meta"] && matches!(key.as_str(), "Q" | "W" | "H" | "M" | ",") {
        return Err(ShortcutError::Reserved);
    }
    let canonical = modifiers
        .iter()
        .copied()
        .chain(std::iter::once(key.as_str()))
        .collect::<Vec<_>>()
        .join("+");
    Ok(Shortcut {
        canonical,
        modifiers,
        key,
    })
}

pub trait ShortcutBackend: Send + Sync {
    fn register(&self, shortcut: &str) -> Result<(), ShortcutError>;
    fn unregister(&self, shortcut: &str) -> Result<(), ShortcutError>;
}

pub struct ShortcutManager<B> {
    backend: B,
    current: Mutex<String>,
}

impl<B: ShortcutBackend> ShortcutManager<B> {
    pub fn new(backend: B, initial: &str) -> Result<Self, ShortcutError> {
        Ok(Self {
            backend,
            current: Mutex::new(parse_shortcut(initial)?.canonical),
        })
    }
    pub fn current(&self) -> String {
        self.current.lock().expect("shortcut lock poisoned").clone()
    }
    pub fn replace(&self, candidate: &str) -> Result<Shortcut, ShortcutError> {
        let candidate = parse_shortcut(candidate)?;
        let mut current = self.current.lock().expect("shortcut lock poisoned");
        if candidate.canonical == *current {
            return Ok(candidate);
        }
        self.backend.register(candidate.canonical())?;
        if self.backend.unregister(&current).is_err() {
            let _ = self.backend.unregister(candidate.canonical());
            return Err(ShortcutError::Unavailable);
        }
        *current = candidate.canonical.clone();
        Ok(candidate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutState {
    Pressed,
    Released,
}

#[derive(Default)]
pub struct PressGate {
    pressed: AtomicBool,
}
impl PressGate {
    pub fn accept(&self, state: ShortcutState) -> bool {
        match state {
            ShortcutState::Pressed => !self.pressed.swap(true, Ordering::SeqCst),
            ShortcutState::Released => {
                self.pressed.store(false, Ordering::SeqCst);
                false
            }
        }
    }
}
