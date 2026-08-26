pub mod coordinator;
pub mod placement;
pub mod shortcut;

pub use coordinator::*;
pub use placement::place_floating_window;
pub use shortcut::*;
pub use vocab_platform_api::{CaptureOrigin, MonitorWorkArea, ScreenPoint, ScreenRect, ScreenSize};
