use vocab_capture::{
    CaptureOrigin, MonitorWorkArea, ScreenPoint, ScreenRect, ScreenSize, place_floating_window,
};

fn point(x: f64, y: f64) -> ScreenPoint {
    ScreenPoint::new(x, y)
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> ScreenRect {
    ScreenRect::new(x, y, width, height)
}

#[test]
fn places_above_selection_and_flips_below_at_top_edge() {
    let monitor = MonitorWorkArea::new(-1440.0, 0.0, 1440.0, 900.0, 2.0);
    let card = ScreenSize::new(380.0, 220.0);

    assert_eq!(
        place_floating_window(
            rect(-900.0, 500.0, 120.0, 24.0),
            point(-840.0, 512.0),
            &[monitor],
            card,
        ),
        point(-1030.0, 268.0)
    );
    assert_eq!(
        place_floating_window(
            rect(-900.0, 8.0, 120.0, 24.0),
            point(-840.0, 20.0),
            &[monitor],
            card,
        ),
        point(-1030.0, 44.0)
    );
}

#[test]
fn clamps_card_to_the_selected_monitors_work_area() {
    let monitors = [
        MonitorWorkArea::new(-1440.0, 0.0, 1440.0, 900.0, 2.0),
        MonitorWorkArea::new(0.0, 0.0, 1728.0, 1080.0, 2.0),
    ];

    assert_eq!(
        place_floating_window(
            rect(-1438.0, 400.0, 10.0, 20.0),
            point(-1433.0, 410.0),
            &monitors,
            ScreenSize::new(380.0, 220.0),
        ),
        point(-1432.0, 168.0)
    );
}

#[test]
fn uses_pointer_monitor_when_selection_bounds_are_unavailable() {
    let monitors = [
        MonitorWorkArea::new(-1440.0, 0.0, 1440.0, 900.0, 2.0),
        MonitorWorkArea::new(0.0, 0.0, 1728.0, 1080.0, 2.0),
    ];

    assert_eq!(
        place_floating_window(
            ScreenRect::default(),
            point(1000.0, 500.0),
            &monitors,
            ScreenSize::new(380.0, 220.0),
        ),
        point(810.0, 268.0)
    );
}

#[test]
fn capture_origin_round_trips_through_json() {
    let value = serde_json::to_string(&CaptureOrigin::Ocr).unwrap();
    assert_eq!(value, "\"ocr\"");
    assert_eq!(
        serde_json::from_str::<CaptureOrigin>(&value).unwrap(),
        CaptureOrigin::Ocr
    );
}
