use std::{fmt, sync::mpsc, time::Duration};

use async_trait::async_trait;
use vocab_platform_api::{OcrCandidate, OcrProvider, PlatformError, ScreenPoint, ScreenRect};
use windows::{
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem},
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
        Imaging::SoftwareBitmap,
    },
    Media::Ocr::OcrEngine,
    Win32::{
        Foundation::{HMODULE, POINT, RECT, RPC_E_CHANGED_MODE},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11_BIND_SHADER_RESOURCE, D3D11_BOX, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11CreateDevice,
                ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
            },
            Dxgi::{IDXGIDevice, IDXGISurface},
        },
        System::{
            Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
            WinRT::Direct3D11::{
                CreateDirect3D11DeviceFromDXGIDevice, CreateDirect3D11SurfaceFromDXGISurface,
                IDirect3DDxgiInterfaceAccess,
            },
            WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
        },
        UI::{
            HiDpi::{GetDpiForWindow, PhysicalToLogicalPointForPerMonitorDPI},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowRect},
        },
    },
    core::{Interface, factory},
};

const REGION_WIDTH: f64 = 640.0;
const REGION_HEIGHT: f64 = 360.0;

#[derive(Clone, Debug)]
struct RecognizedWord {
    text: String,
    bounds: ScreenRect,
}

pub(crate) struct WindowsOcrProvider;

#[async_trait]
impl OcrProvider for WindowsOcrProvider {
    async fn recognize_near(
        &self,
        pointer: ScreenPoint,
    ) -> Result<Vec<OcrCandidate>, PlatformError> {
        tokio::task::spawn_blocking(move || capture_and_recognize(pointer))
            .await
            .map_err(|_| operation("Windows OCR worker failed"))?
    }
}

fn capture_and_recognize(pointer: ScreenPoint) -> Result<Vec<OcrCandidate>, PlatformError> {
    let _apartment = ComApartment::enter()?;
    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return Err(operation("OCR source window is unavailable"));
    }
    let item_bounds = logical_window_bounds(window)?;
    let region = bounded_capture_region(pointer, item_bounds);
    let scale = f64::from(unsafe { GetDpiForWindow(window) }) / 96.0;
    let item = capture_item_for_window(window)?;
    let size = item
        .Size()
        .map_err(|_| operation("OCR capture size is unavailable"))?;
    let (device, context, winrt_device) = direct3d_device()?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &winrt_device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        size,
    )
    .map_err(|_| operation("OCR frame pool creation failed"))?;
    let pool_guard = CloseFramePool(pool.clone());
    let session = pool
        .CreateCaptureSession(&item)
        .map_err(|_| operation("OCR capture session creation failed"))?;
    let session_guard = CloseSession(session.clone());
    let (sender, receiver) = mpsc::sync_channel(1);
    let token = pool
        .FrameArrived(&TypedEventHandler::new(
            move |sender_pool: windows::core::Ref<'_, Direct3D11CaptureFramePool>, _| {
                if let Some(sender_pool) = sender_pool.as_ref()
                    && let Ok(frame) = sender_pool.TryGetNextFrame()
                {
                    let _ = sender.try_send(frame);
                }
                Ok(())
            },
        ))
        .map_err(|_| operation("OCR frame callback registration failed"))?;
    let subscription_guard = FrameSubscription {
        pool: pool.clone(),
        token,
    };
    session
        .StartCapture()
        .map_err(|_| operation("OCR capture could not start"))?;
    let frame = receiver
        .recv_timeout(Duration::from_secs(3))
        .map_err(|_| operation("OCR capture timed out"))?;
    let frame_guard = CloseFrame(frame.clone());

    let local_x = ((region.x - item_bounds.x) * scale).round().max(0.0) as u32;
    let local_y = ((region.y - item_bounds.y) * scale).round().max(0.0) as u32;
    let width = (region.width * scale).round().max(1.0) as u32;
    let height = (region.height * scale).round().max(1.0) as u32;
    let surface = cropped_surface(&device, &context, &frame, local_x, local_y, width, height)?;
    let bitmap = SoftwareBitmap::CreateCopyFromSurfaceAsync(&surface)
        .and_then(|operation| operation.get())
        .map_err(|_| operation("OCR bitmap copy failed"))?;
    let bitmap_guard = CloseBitmap(bitmap.clone());
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|_| operation("Windows OCR language is unavailable"))?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .and_then(|operation| operation.get())
        .map_err(|_| operation("Windows OCR recognition failed"))?;
    let mut words = Vec::new();
    for line in result
        .Lines()
        .map_err(|_| operation("OCR lines are unavailable"))?
    {
        for word in line
            .Words()
            .map_err(|_| operation("OCR words are unavailable"))?
        {
            let rect = word
                .BoundingRect()
                .map_err(|_| operation("OCR word bounds are unavailable"))?;
            words.push(RecognizedWord {
                text: word
                    .Text()
                    .map_err(|_| operation("OCR word text is unavailable"))?
                    .to_string(),
                bounds: ScreenRect::new(
                    f64::from(rect.X),
                    f64::from(rect.Y),
                    f64::from(rect.Width),
                    f64::from(rect.Height),
                ),
            });
        }
    }
    let candidates = normalize_words(&words, ScreenPoint::new(region.x, region.y), scale);
    eprintln!(
        "{}",
        OcrDiagnostics {
            region_width: width,
            region_height: height,
            candidate_count: candidates.len(),
        }
    );
    drop(subscription_guard);
    drop(bitmap_guard);
    drop(frame_guard);
    drop(session_guard);
    drop(pool_guard);
    Ok(candidates)
}

fn logical_window_bounds(
    window: windows::Win32::Foundation::HWND,
) -> Result<ScreenRect, PlatformError> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(window, &mut rect) }
        .map_err(|_| operation("OCR source bounds are unavailable"))?;
    let mut top_left = POINT {
        x: rect.left,
        y: rect.top,
    };
    let mut bottom_right = POINT {
        x: rect.right,
        y: rect.bottom,
    };
    if !unsafe { PhysicalToLogicalPointForPerMonitorDPI(Some(window), &mut top_left) }.as_bool()
        || !unsafe { PhysicalToLogicalPointForPerMonitorDPI(Some(window), &mut bottom_right) }
            .as_bool()
    {
        return Err(operation("OCR source coordinates are unavailable"));
    }
    Ok(ScreenRect::new(
        f64::from(top_left.x),
        f64::from(top_left.y),
        f64::from(bottom_right.x - top_left.x),
        f64::from(bottom_right.y - top_left.y),
    ))
}

fn capture_item_for_window(
    window: windows::Win32::Foundation::HWND,
) -> Result<GraphicsCaptureItem, PlatformError> {
    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(|_| operation("Windows Graphics Capture is unavailable"))?;
    unsafe { interop.CreateForWindow(window) }
        .map_err(|_| operation("OCR source window cannot be captured"))
}

fn direct3d_device() -> Result<(ID3D11Device, ID3D11DeviceContext, IDirect3DDevice), PlatformError>
{
    let mut device = None;
    let mut context = None;
    unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&raw mut device),
            None,
            Some(&raw mut context),
        )
    }
    .map_err(|_| operation("Direct3D device creation failed"))?;
    let device = device.ok_or_else(|| operation("Direct3D device is unavailable"))?;
    let context = context.ok_or_else(|| operation("Direct3D context is unavailable"))?;
    let dxgi: IDXGIDevice = device
        .cast()
        .map_err(|_| operation("DXGI device conversion failed"))?;
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
        .map_err(|_| operation("WinRT Direct3D device conversion failed"))?;
    let winrt_device = inspectable
        .cast()
        .map_err(|_| operation("WinRT Direct3D device is unavailable"))?;
    Ok((device, context, winrt_device))
}

fn cropped_surface(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    frame: &Direct3D11CaptureFrame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<windows::Graphics::DirectX::Direct3D11::IDirect3DSurface, PlatformError> {
    let source_surface = frame
        .Surface()
        .map_err(|_| operation("OCR frame surface is unavailable"))?;
    let access: IDirect3DDxgiInterfaceAccess = source_surface
        .cast()
        .map_err(|_| operation("OCR frame texture access failed"))?;
    let source: ID3D11Texture2D = unsafe { access.GetInterface() }
        .map_err(|_| operation("OCR frame texture is unavailable"))?;
    let mut source_desc = D3D11_TEXTURE2D_DESC::default();
    unsafe { source.GetDesc(&mut source_desc) };
    let x = x.min(source_desc.Width.saturating_sub(1));
    let y = y.min(source_desc.Height.saturating_sub(1));
    let width = width.min(source_desc.Width.saturating_sub(x)).max(1);
    let height = height.min(source_desc.Height.saturating_sub(y)).max(1);
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: source_desc.Format,
        SampleDesc: source_desc.SampleDesc,
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };
    let mut cropped = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&raw mut cropped)) }
        .map_err(|_| operation("OCR crop texture creation failed"))?;
    let cropped = cropped.ok_or_else(|| operation("OCR crop texture is unavailable"))?;
    let source_box = D3D11_BOX {
        left: x,
        top: y,
        front: 0,
        right: x + width,
        bottom: y + height,
        back: 1,
    };
    unsafe { context.CopySubresourceRegion(&cropped, 0, 0, 0, 0, &source, 0, Some(&source_box)) };
    let dxgi_surface: IDXGISurface = cropped
        .cast()
        .map_err(|_| operation("OCR crop surface conversion failed"))?;
    let inspectable = unsafe { CreateDirect3D11SurfaceFromDXGISurface(&dxgi_surface) }
        .map_err(|_| operation("OCR WinRT surface conversion failed"))?;
    inspectable
        .cast()
        .map_err(|_| operation("OCR WinRT surface is unavailable"))
}

struct CloseFramePool(Direct3D11CaptureFramePool);
impl Drop for CloseFramePool {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}
struct CloseSession(windows::Graphics::Capture::GraphicsCaptureSession);
impl Drop for CloseSession {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}
struct CloseFrame(Direct3D11CaptureFrame);
impl Drop for CloseFrame {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}
struct CloseBitmap(SoftwareBitmap);
impl Drop for CloseBitmap {
    fn drop(&mut self) {
        let _ = self.0.Close();
    }
}

struct FrameSubscription {
    pool: Direct3D11CaptureFramePool,
    token: i64,
}

impl Drop for FrameSubscription {
    fn drop(&mut self) {
        let _ = self.pool.RemoveFrameArrived(self.token);
    }
}

struct OcrDiagnostics {
    region_width: u32,
    region_height: u32,
    candidate_count: usize,
}

impl fmt::Display for OcrDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "windows_ocr region_width={} region_height={} candidate_count={}",
            self.region_width, self.region_height, self.candidate_count
        )
    }
}

fn operation(message: &str) -> PlatformError {
    PlatformError::Operation(message.into())
}

struct ComApartment {
    uninitialize: bool,
}

impl ComApartment {
    fn enter() -> Result<Self, PlatformError> {
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_ok() {
            Ok(Self { uninitialize: true })
        } else if result == RPC_E_CHANGED_MODE {
            Ok(Self {
                uninitialize: false,
            })
        } else {
            Err(operation("Windows OCR COM initialization failed"))
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.uninitialize {
            unsafe { CoUninitialize() };
        }
    }
}

fn bounded_capture_region(pointer: ScreenPoint, item: ScreenRect) -> ScreenRect {
    let width = REGION_WIDTH.min(item.width);
    let height = REGION_HEIGHT.min(item.height);
    let x = (pointer.x - width / 2.0).clamp(item.x, item.x + item.width - width);
    let y = (pointer.y - height / 2.0).clamp(item.y, item.y + item.height - height);
    ScreenRect::new(x, y, width, height)
}

fn normalize_words(
    words: &[RecognizedWord],
    region_origin: ScreenPoint,
    scale_factor: f64,
) -> Vec<OcrCandidate> {
    words
        .iter()
        .filter(|word| !word.text.trim().is_empty() && word.bounds.is_available())
        .map(|word| OcrCandidate {
            text: word.text.clone(),
            bounds: ScreenRect::new(
                region_origin.x + word.bounds.x / scale_factor,
                region_origin.y + word.bounds.y / scale_factor,
                word.bounds.width / scale_factor,
                word.bounds.height / scale_factor,
            ),
            confidence: 1.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use vocab_platform_api::{ScreenPoint, ScreenRect};

    use super::{OcrDiagnostics, RecognizedWord, bounded_capture_region, normalize_words};

    #[test]
    fn bounded_region_centers_on_pointer_and_clamps_to_the_capture_item() {
        assert_eq!(
            bounded_capture_region(
                ScreenPoint::new(1_900.0, 1_050.0),
                ScreenRect::new(0.0, 0.0, 1_920.0, 1_080.0),
            ),
            ScreenRect::new(1_280.0, 720.0, 640.0, 360.0),
        );
        assert_eq!(
            bounded_capture_region(
                ScreenPoint::new(-900.0, 100.0),
                ScreenRect::new(-1_280.0, -200.0, 1_280.0, 800.0),
            ),
            ScreenRect::new(-1_220.0, -80.0, 640.0, 360.0),
        );
    }

    #[test]
    fn recognized_words_become_portable_candidates_without_content_diagnostics() {
        let candidates = normalize_words(
            &[RecognizedWord {
                text: "Straße".into(),
                bounds: ScreenRect::new(12.0, 18.0, 70.0, 24.0),
            }],
            ScreenPoint::new(-1_220.0, -80.0),
            1.25,
        );

        assert_eq!(candidates[0].text, "Straße");
        assert_eq!(
            candidates[0].bounds,
            ScreenRect::new(-1210.4, -65.6, 56.0, 19.2)
        );
        assert_eq!(candidates[0].confidence, 1.0);
        let diagnostics = OcrDiagnostics {
            region_width: 640,
            region_height: 360,
            candidate_count: candidates.len(),
        }
        .to_string();
        assert!(!diagnostics.contains("Straße"));
    }
}
