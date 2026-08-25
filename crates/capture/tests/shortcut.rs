use std::sync::{Arc, Mutex};
use vocab_capture::{
    PressGate, ShortcutBackend, ShortcutError, ShortcutManager, ShortcutState, parse_shortcut,
};

#[derive(Clone, Default)]
struct RecordingBackend(Arc<Mutex<Vec<String>>>);

impl ShortcutBackend for RecordingBackend {
    fn register(&self, shortcut: &str) -> Result<(), ShortcutError> {
        self.0.lock().unwrap().push(format!("register:{shortcut}"));
        Ok(())
    }
    fn unregister(&self, shortcut: &str) -> Result<(), ShortcutError> {
        self.0
            .lock()
            .unwrap()
            .push(format!("unregister:{shortcut}"));
        Ok(())
    }
}

#[test]
fn validates_default_and_rejects_unmodified_or_reserved_shortcuts() {
    assert_eq!(
        parse_shortcut("Alt+Space+V").unwrap().display_macos(),
        "⌥ Space V"
    );
    assert_eq!(
        parse_shortcut("V").unwrap_err(),
        ShortcutError::ModifierRequired
    );
    assert_eq!(
        parse_shortcut("Meta+Q").unwrap_err(),
        ShortcutError::Reserved
    );
}

#[test]
fn replacement_registers_candidate_before_removing_current() {
    let backend = RecordingBackend::default();
    let manager = ShortcutManager::new(backend.clone(), "Alt+Space+V").unwrap();
    manager.replace("Control+Shift+W").unwrap();
    assert_eq!(
        *backend.0.lock().unwrap(),
        ["register:Control+Shift+W", "unregister:Alt+Space+V"]
    );
}

#[test]
fn replacement_failure_keeps_the_current_shortcut() {
    #[derive(Clone)]
    struct Rejecting;
    impl ShortcutBackend for Rejecting {
        fn register(&self, _: &str) -> Result<(), ShortcutError> {
            Err(ShortcutError::Unavailable)
        }
        fn unregister(&self, _: &str) -> Result<(), ShortcutError> {
            panic!("old shortcut removed")
        }
    }
    let manager = ShortcutManager::new(Rejecting, "Alt+Space+V").unwrap();
    assert_eq!(
        manager.replace("Control+Shift+W"),
        Err(ShortcutError::Unavailable)
    );
    assert_eq!(manager.current(), "Alt+Space+V");
}

#[test]
fn press_gate_ignores_key_repeat_until_release() {
    let gate = PressGate::default();
    assert!(gate.accept(ShortcutState::Pressed));
    assert!(!gate.accept(ShortcutState::Pressed));
    assert!(!gate.accept(ShortcutState::Released));
    assert!(gate.accept(ShortcutState::Pressed));
}
