use vocab_platform::{MonitorWorkArea, ScreenPoint, ScreenRect, ScreenSize};

const GAP: f64 = 12.0;
const MARGIN: f64 = 8.0;

pub fn place_floating_window(
    anchor: ScreenRect,
    pointer: ScreenPoint,
    monitors: &[MonitorWorkArea],
    card: ScreenSize,
) -> ScreenPoint {
    let monitor = monitor_for(anchor, pointer, monitors)
        .copied()
        .unwrap_or(MonitorWorkArea::new(
            pointer.x,
            pointer.y,
            card.width,
            card.height,
            1.0,
        ));
    let effective_anchor = if anchor.is_available() {
        anchor
    } else {
        ScreenRect::new(pointer.x, pointer.y, 0.0, 0.0)
    };
    let x = effective_anchor.x + (effective_anchor.width - card.width) / 2.0;
    let above = effective_anchor.y - card.height - GAP;
    let y = if above >= monitor.y + MARGIN {
        above
    } else {
        effective_anchor.y + effective_anchor.height + GAP
    };

    ScreenPoint::new(
        clamp(
            x,
            monitor.x + MARGIN,
            monitor.x + monitor.width - card.width - MARGIN,
        ),
        clamp(
            y,
            monitor.y + MARGIN,
            monitor.y + monitor.height - card.height - MARGIN,
        ),
    )
}

fn monitor_for(
    anchor: ScreenRect,
    pointer: ScreenPoint,
    monitors: &[MonitorWorkArea],
) -> Option<&MonitorWorkArea> {
    let target = if anchor.is_available() {
        anchor.center()
    } else {
        pointer
    };
    monitors
        .iter()
        .find(|monitor| monitor.contains(target))
        .or_else(|| monitors.iter().find(|monitor| monitor.contains(pointer)))
        .or_else(|| monitors.first())
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if max < min {
        min
    } else {
        value.clamp(min, max)
    }
}
