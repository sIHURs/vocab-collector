use async_trait::async_trait;
use vocab_platform_api::{
    CaptureCandidate, CaptureOrigin, PlatformError, ScreenRect, SelectionProvider,
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HWND, POINT, RPC_E_CHANGED_MODE},
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, SAFEARRAY,
            },
            Ole::{SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound},
            Threading::{
                OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
            },
        },
        UI::{
            Accessibility::{
                CUIAutomation, IUIAutomation, IUIAutomationTextPattern, IUIAutomationTextRange,
                TextUnit, TextUnit_Line, TextUnit_Paragraph, UIA_TextPatternId,
            },
            HiDpi::PhysicalToLogicalPointForPerMonitorDPI,
            WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW},
        },
    },
    core::PWSTR,
};

pub(crate) struct WindowsSelectionProvider;

#[derive(Debug)]
struct SelectionSnapshot {
    selected_text: String,
    document_text: Option<String>,
    source_app: Option<String>,
    source_title: Option<String>,
    bounds: Vec<ScreenRect>,
}

#[async_trait]
impl SelectionProvider for WindowsSelectionProvider {
    async fn capture_selection(&self) -> Result<CaptureCandidate, PlatformError> {
        tokio::task::spawn_blocking(|| capture_focused_selection().and_then(normalize_snapshot))
            .await
            .map_err(|_| operation("UI Automation worker failed"))?
    }
}

fn capture_focused_selection() -> Result<SelectionSnapshot, PlatformError> {
    let _apartment = ComApartment::enter()?;
    // SAFETY: COM is initialized for this thread by `ComApartment`; every interface remains
    // local to this call and is dropped before the apartment guard uninitializes COM.
    unsafe { capture_focused_selection_in_apartment() }
}

unsafe fn capture_focused_selection_in_apartment() -> Result<SelectionSnapshot, PlatformError> {
    // SAFETY: the CLSID and requested interface are the documented UI Automation client object.
    let automation: IUIAutomation = unsafe {
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            .map_err(|_| operation("UI Automation initialization failed"))?
    };
    // SAFETY: `automation` is a live interface in the current COM apartment.
    let focused = unsafe { automation.GetFocusedElement() }
        .map_err(|_| operation("UI Automation could not read the focused element"))?;
    // SAFETY: the typed query requests only the standard TextPattern interface.
    let pattern =
        unsafe { focused.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }
            .map_err(|_| PlatformError::UnsupportedElement)?;
    // SAFETY: the range array is owned by the returned COM interface and is accessed in bounds.
    let ranges = unsafe { pattern.GetSelection() }
        .map_err(|_| operation("UI Automation could not read the selection"))?;
    // SAFETY: `ranges` is a live UI Automation array.
    if unsafe { ranges.Length() }.unwrap_or_default() <= 0 {
        return Err(PlatformError::EmptySelection);
    }
    // SAFETY: index zero is valid after the positive length check.
    let range =
        unsafe { ranges.GetElement(0) }.map_err(|_| PlatformError::InvalidSelectionRange)?;
    // SAFETY: `range` is a live selected text range; `-1` requests all text in that range.
    let selected_text = unsafe { range.GetText(-1) }
        .map_err(|_| PlatformError::InvalidSelectionRange)?
        .to_string();
    // SAFETY: optional metadata calls use live UIA interfaces and failures are non-fatal.
    let document_text = unsafe { enclosing_context(&range) };
    // SAFETY: optional range geometry is copied immediately and the SAFEARRAY is destroyed.
    // SAFETY: the foreground HWND is borrowed and used only during this synchronous call.
    let foreground = unsafe { GetForegroundWindow() };
    let physical_bounds = unsafe { selection_bounds(&range) }.unwrap_or_default();
    let bounds = normalize_physical_bounds(&physical_bounds, |x, y| {
        physical_to_logical(foreground, x, y)
    });
    // SAFETY: property access is on the live focused element.
    let process_id = unsafe { focused.CurrentProcessId() }.ok();

    Ok(SelectionSnapshot {
        selected_text,
        document_text,
        source_app: process_id.and_then(process_name),
        source_title: foreground_window_title(),
        bounds,
    })
}

unsafe fn enclosing_context(range: &IUIAutomationTextRange) -> Option<String> {
    for unit in [TextUnit_Paragraph, TextUnit_Line] {
        // SAFETY: the clone remains in the same COM apartment as the live source range.
        let Ok(context) = (unsafe { range.Clone() }) else {
            continue;
        };
        // SAFETY: expanding the cloned range does not mutate the active selection.
        if unsafe { context.ExpandToEnclosingUnit(TextUnit(unit.0)) }.is_err() {
            continue;
        }
        // SAFETY: `context` is a live UIA text range; the copied BSTR owns the returned text.
        if let Ok(text) = unsafe { context.GetText(-1) } {
            return Some(text.to_string());
        }
    }
    None
}

struct ComApartment {
    uninitialize: bool,
}

impl ComApartment {
    fn enter() -> Result<Self, PlatformError> {
        // SAFETY: null reserved pointer and MTA are valid. A successful call is balanced in Drop.
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_ok() {
            Ok(Self { uninitialize: true })
        } else if result == RPC_E_CHANGED_MODE {
            Ok(Self {
                uninitialize: false,
            })
        } else {
            Err(operation("COM initialization failed"))
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.uninitialize {
            // SAFETY: this balances the successful CoInitializeEx call on the same thread.
            unsafe { CoUninitialize() };
        }
    }
}

unsafe fn selection_bounds(
    range: &IUIAutomationTextRange,
) -> windows::core::Result<Vec<ScreenRect>> {
    // SAFETY: `range` is a live UIA range and returns a caller-owned SAFEARRAY.
    let array = unsafe { range.GetBoundingRectangles()? };
    if array.is_null() {
        return Ok(Vec::new());
    }
    let array = SafeArrayGuard(array);
    // SAFETY: UIA returns a one-dimensional SAFEARRAY of f64 rectangle components.
    let lower = unsafe { SafeArrayGetLBound(array.0, 1)? };
    // SAFETY: same valid SAFEARRAY and dimension as above.
    let upper = unsafe { SafeArrayGetUBound(array.0, 1)? };
    let mut values = Vec::with_capacity((upper - lower + 1).max(0) as usize);
    for index in lower..=upper {
        let mut value = 0.0_f64;
        // SAFETY: the index is within the bounds read from the SAFEARRAY; destination is f64.
        unsafe {
            SafeArrayGetElement(
                array.0,
                &index,
                (&raw mut value).cast::<core::ffi::c_void>(),
            )?
        };
        values.push(value);
    }
    Ok(values
        .as_chunks::<4>()
        .0
        .iter()
        .map(|parts| ScreenRect::new(parts[0], parts[1], parts[2], parts[3]))
        .filter(|rect| rect.is_available())
        .collect())
}

struct SafeArrayGuard(*mut SAFEARRAY);

impl Drop for SafeArrayGuard {
    fn drop(&mut self) {
        // SAFETY: the pointer is the caller-owned SAFEARRAY returned by UI Automation.
        let _ = unsafe { SafeArrayDestroy(self.0) };
    }
}

fn foreground_window_title() -> Option<String> {
    // SAFETY: GetForegroundWindow returns a borrowed HWND; GetWindowTextW writes within the buffer.
    unsafe {
        let window = GetForegroundWindow();
        if window.0.is_null() {
            return None;
        }
        let mut buffer = vec![0_u16; 512];
        let length = GetWindowTextW(window, &mut buffer);
        (length > 0).then(|| String::from_utf16_lossy(&buffer[..length as usize]))
    }
}

fn process_name(process_id: i32) -> Option<String> {
    let process_id = u32::try_from(process_id).ok()?;
    // SAFETY: the handle is requested read-only for the UIA-reported process ID.
    let process =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }.ok()?;
    let mut buffer = vec![0_u16; 1024];
    let mut length = buffer.len() as u32;
    // SAFETY: buffer and length describe writable storage; the process handle is valid.
    let result = unsafe {
        QueryFullProcessImageNameW(
            process,
            Default::default(),
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    };
    // SAFETY: this closes exactly the handle returned by OpenProcess.
    let _ = unsafe { CloseHandle(process) };
    result.ok()?;
    std::path::Path::new(&String::from_utf16_lossy(&buffer[..length as usize]))
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

fn operation(message: &'static str) -> PlatformError {
    PlatformError::Operation(message.into())
}

fn normalize_snapshot(snapshot: SelectionSnapshot) -> Result<CaptureCandidate, PlatformError> {
    if snapshot.selected_text.trim().is_empty() {
        return Err(PlatformError::EmptySelection);
    }
    let sentence = snapshot
        .document_text
        .as_deref()
        .and_then(|document| sentence_containing(document, &snapshot.selected_text))
        .unwrap_or_else(|| snapshot.selected_text.clone());
    let selection_bounds = combined_bounds(&snapshot.bounds);

    Ok(CaptureCandidate {
        selected_text: snapshot.selected_text,
        sentence,
        source_app: snapshot.source_app,
        source_title: snapshot.source_title,
        source_url: None,
        selection_bounds,
        origin: CaptureOrigin::Accessibility,
    })
}

fn sentence_containing(document: &str, selection: &str) -> Option<String> {
    if document.match_indices(selection).take(2).count() != 1 {
        let context = document.trim();
        return (!context.is_empty()).then(|| context.to_string());
    }
    let selected_start = document.find(selection)?;
    let selected_end = selected_start + selection.len();
    let start = document[..selected_start]
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            is_sentence_end(character).then_some(index + character.len_utf8())
        })
        .unwrap_or(0);
    let end = document[selected_end..]
        .char_indices()
        .find_map(|(offset, character)| {
            is_sentence_end(character).then_some(selected_end + offset + character.len_utf8())
        })
        .unwrap_or(document.len());
    let sentence = document[start..end].trim();
    (!sentence.is_empty()).then(|| sentence.to_string())
}

fn normalize_physical_bounds(
    bounds: &[ScreenRect],
    mut convert: impl FnMut(f64, f64) -> Option<(f64, f64)>,
) -> Vec<ScreenRect> {
    bounds
        .iter()
        .filter_map(|rect| {
            let (left, top) = convert(rect.x, rect.y)?;
            let (right, bottom) = convert(rect.x + rect.width, rect.y + rect.height)?;
            let logical = ScreenRect::new(left, top, right - left, bottom - top);
            logical.is_available().then_some(logical)
        })
        .collect()
}

fn physical_to_logical(window: HWND, x: f64, y: f64) -> Option<(f64, f64)> {
    if window.0.is_null() {
        return None;
    }
    let mut point = POINT {
        x: x.round() as i32,
        y: y.round() as i32,
    };
    // SAFETY: `window` is the current foreground window and `point` is writable storage.
    unsafe { PhysicalToLogicalPointForPerMonitorDPI(Some(window), &mut point) }
        .as_bool()
        .then(|| (f64::from(point.x), f64::from(point.y)))
}

fn is_sentence_end(character: char) -> bool {
    matches!(
        character,
        '.' | '!' | '?' | '\n' | '\r' | '。' | '！' | '？'
    )
}

fn combined_bounds(bounds: &[ScreenRect]) -> Option<ScreenRect> {
    let mut available = bounds.iter().copied().filter(|rect| rect.is_available());
    let first = available.next()?;
    let (left, top, right, bottom) = available.fold(
        (
            first.x,
            first.y,
            first.x + first.width,
            first.y + first.height,
        ),
        |(left, top, right, bottom), rect| {
            (
                left.min(rect.x),
                top.min(rect.y),
                right.max(rect.x + rect.width),
                bottom.max(rect.y + rect.height),
            )
        },
    );
    Some(ScreenRect::new(left, top, right - left, bottom - top))
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{CaptureOrigin, PlatformError, ScreenRect};

    use super::{SelectionSnapshot, normalize_snapshot};

    #[test]
    fn normalizes_exact_unicode_context_metadata_and_combined_bounds() {
        let candidate = normalize_snapshot(SelectionSnapshot {
            selected_text: "naïve—CAFÉ 👩🏽‍💻".into(),
            document_text: Some("Before. naïve—CAFÉ 👩🏽‍💻 remains exact. After.".into()),
            source_app: Some("notepad.exe".into()),
            source_title: Some("Unicode fixture - Notepad".into()),
            bounds: vec![
                ScreenRect::new(100.0, 200.0, 80.0, 20.0),
                ScreenRect::new(100.0, 220.0, 50.0, 20.0),
            ],
        })
        .unwrap();

        assert_eq!(candidate.selected_text, "naïve—CAFÉ 👩🏽‍💻");
        assert_eq!(candidate.sentence, "naïve—CAFÉ 👩🏽‍💻 remains exact.");
        assert_eq!(candidate.source_app.as_deref(), Some("notepad.exe"));
        assert_eq!(
            candidate.source_title.as_deref(),
            Some("Unicode fixture - Notepad")
        );
        assert_eq!(candidate.source_url, None);
        assert_eq!(
            candidate.selection_bounds,
            Some(ScreenRect::new(100.0, 200.0, 80.0, 40.0))
        );
        assert_eq!(candidate.origin, CaptureOrigin::Accessibility);
    }

    #[test]
    fn rejects_whitespace_only_selection_with_a_typed_error() {
        assert_eq!(
            normalize_snapshot(SelectionSnapshot {
                selected_text: " \r\n\t".into(),
                document_text: None,
                source_app: None,
                source_title: None,
                bounds: Vec::new(),
            }),
            Err(PlatformError::EmptySelection)
        );
    }

    #[test]
    fn repeated_selection_keeps_the_enclosing_context_instead_of_guessing_an_occurrence() {
        let candidate = normalize_snapshot(SelectionSnapshot {
            selected_text: "repeat".into(),
            document_text: Some("First repeat. The selected repeat is here.".into()),
            source_app: None,
            source_title: None,
            bounds: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            candidate.sentence,
            "First repeat. The selected repeat is here."
        );
    }

    #[test]
    fn normalizes_physical_rectangles_with_the_supplied_window_coordinate_conversion() {
        let logical = super::normalize_physical_bounds(
            &[ScreenRect::new(150.0, 300.0, 300.0, 60.0)],
            |x, y| Some((x / 1.5, y / 1.5)),
        );

        assert_eq!(logical, vec![ScreenRect::new(100.0, 200.0, 200.0, 40.0)]);
    }
}
