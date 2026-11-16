use std::collections::HashMap;
use std::mem::ManuallyDrop;

use mere_base::{Cmd, Color, DrawList, Gfx, ImageId, Mat4, Material, MeshId, Rect, Vertex, Weight, observe};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::core::{Error, HRESULT, Interface, PCSTR, Result, s, w};

// Direct2D draws only into B8G8R8A8 surfaces, so the picture is in that order; textures uploaded by the
// application stay RGBA, which a shader reads the same way.
const PICTURE: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM;
const DEPTH: DXGI_FORMAT = DXGI_FORMAT_D32_FLOAT;

// What the web view gets from WebGL's antialias:true. 4 is the count every Direct3D 11 card must support
// for the formats used here; it is still asked, because WARP or a remote adapter may differ.
const WANTED_SAMPLES: u32 = 4;

// Direct2D's "make the target again" (D2DERR_RECREATE_TARGET), which windows-rs does not export.
const D2DERR_RECREATE_TARGET: HRESULT = HRESULT(0x8899000C_u32 as i32);
// Present's success code for "the window is not visible" (DXGI_STATUS_OCCLUDED), likewise not exported.
const DXGI_STATUS_OCCLUDED: HRESULT = HRESULT(0x087A0001);

const SHADERS: &str = r#"
cbuffer Frame : register(b0) { float4x4 view_proj; float4 tint; };
struct VsIn { float3 pos : POSITION; float2 uv : TEXCOORD0; };
struct VsOut { float4 pos : SV_Position; float2 uv : TEXCOORD0; };
VsOut vs(VsIn i) { VsOut o; o.pos = mul(view_proj, float4(i.pos, 1)); o.uv = i.uv; return o; }
Texture2D image : register(t0);
SamplerState pick : register(s0);
float4 ps_image(VsOut i) : SV_Target { return image.Sample(pick, i.uv); }
// Premultiplied, as the blend state expects.
float4 ps_flat(VsOut i) : SV_Target { return float4(tint.rgb * tint.a, tint.a); }
"#;

#[repr(C)]
struct FrameConstants {
    view_proj: [f32; 16],
    tint: [f32; 4],
}

enum Target {
    Window(IDXGISwapChain1),
    Offscreen(ID3D11Texture2D),
}

struct Mesh {
    vertices: ID3D11Buffer,
    indices: ID3D11Buffer,
    count: u32,
}

/// Everything that lives as long as the picture's size: dropped and made again on a resize.
struct Views {
    // The 3D pass draws into this: the multisampled image when samples > 1, else the picture itself.
    rtv: ID3D11RenderTargetView,
    dsv: ID3D11DepthStencilView,
    // The single-sample image Direct2D draws on and the screen shows.
    picture: ID3D11Texture2D,
    // Only with multisampling: resolved into picture before the interface is drawn.
    multisampled: Option<ID3D11Texture2D>,
    // Kept so Direct2D draws into the same texture as Direct3D.
    _bitmap: ID2D1Bitmap1,
}

pub struct Renderer {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    d2d: ID2D1DeviceContext,
    dwrite: IDWriteFactory,
    brush: ID2D1SolidColorBrush,
    // For the polylines' path geometries, and the one stroke style they share.
    factory: ID2D1Factory1,
    stroke: ID2D1StrokeStyle,
    target: Target,
    views: Option<Views>,
    width: u32,
    height: u32,
    scale: f32,
    samples: u32,
    // True from `begin` until the 3D pass is resolved into the picture (a no-op without multisampling).
    unresolved: bool,
    lost: bool,
    // Present said the window is covered or minimised: drawing it would only waste the processor.
    occluded: bool,
    vs: ID3D11VertexShader,
    ps_image: ID3D11PixelShader,
    ps_flat: ID3D11PixelShader,
    layout: ID3D11InputLayout,
    constants: ID3D11Buffer,
    nearest: ID3D11SamplerState,
    linear: ID3D11SamplerState,
    blend_over: ID3D11BlendState,
    depth_write: ID3D11DepthStencilState,
    depth_test: ID3D11DepthStencilState,
    raster: ID3D11RasterizerState,
    meshes: HashMap<MeshId, Mesh>,
    images: HashMap<ImageId, ID3D11ShaderResourceView>,
    // The same pictures as Direct2D bitmaps, for `Cmd::Image`; the texture above is what a mesh drapes.
    bitmaps: HashMap<ImageId, ID2D1Bitmap1>,
    // One text format a size and weight: DirectWrite formats are immutable, and sizes are few.
    formats: HashMap<(u32, Weight), IDWriteTextFormat>,
    // None when IBM Plex Mono could not be loaded: text is then drawn in Consolas from the system.
    fonts: Option<crate::fonts::Fonts>,
}

impl Renderer {
    pub fn for_window(hwnd: HWND, width: u32, height: u32, scale: f32) -> Result<Renderer> {
        let (device, ctx) = create_device()?;
        let factory: IDXGIFactory2 = unsafe { device.cast::<IDXGIDevice>()?.GetAdapter()?.GetParent()? };
        let desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width.max(1),
            Height: height.max(1),
            Format: PICTURE,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            // The flip model hands the image to the compositor without a copy; "discard" lets the driver
            // forget the previous image, which a frame redrawn whole never needs.
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            ..Default::default()
        };
        let chain = unsafe { factory.CreateSwapChainForHwnd(&device, hwnd, &desc, None, None)? };
        Renderer::new(device, ctx, Target::Window(chain), width, height, scale)
    }

    pub fn offscreen(width: u32, height: u32, scale: f32) -> Result<Renderer> {
        let (device, ctx) = create_device()?;
        let texture = picture_texture(&device, width, height)?;
        Renderer::new(device, ctx, Target::Offscreen(texture), width, height, scale)
    }

    fn new(device: ID3D11Device, ctx: ID3D11DeviceContext, target: Target, width: u32, height: u32, scale: f32) -> Result<Renderer> {
        unsafe {
            let factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d_device = factory.CreateDevice(&device.cast::<IDXGIDevice>()?)?;
            let d2d = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let fonts = match dwrite.cast::<IDWriteFactory5>().and_then(|f| crate::fonts::load(&f)) {
                Ok(f) => Some(f),
                Err(e) => {
                    observe!(Warn, "IBM Plex Mono not loaded ({e}); falling back to {}", crate::fonts::FALLBACK_FAMILY);
                    None
                }
            };
            let brush = d2d.CreateSolidColorBrush(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, None)?;
            // SVG's stroke-linejoin: round with the default butt caps and miter limit.
            let stroke: ID2D1StrokeStyle = factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES1 {
                    startCap: D2D1_CAP_STYLE_FLAT,
                    endCap: D2D1_CAP_STYLE_FLAT,
                    dashCap: D2D1_CAP_STYLE_FLAT,
                    lineJoin: D2D1_LINE_JOIN_ROUND,
                    miterLimit: 10.0,
                    dashStyle: D2D1_DASH_STYLE_SOLID,
                    dashOffset: 0.0,
                    transformType: D2D1_STROKE_TRANSFORM_TYPE_NORMAL,
                },
                None,
            )?.into();

            let vs_code = compile(s!("vs"), s!("vs_5_0"))?;
            let mut vs = None;
            device.CreateVertexShader(&vs_code, None, Some(&mut vs))?;
            let mut ps_image = None;
            device.CreatePixelShader(&compile(s!("ps_image"), s!("ps_5_0"))?, None, Some(&mut ps_image))?;
            let mut ps_flat = None;
            device.CreatePixelShader(&compile(s!("ps_flat"), s!("ps_5_0"))?, None, Some(&mut ps_flat))?;

            let elements = [
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: s!("POSITION"),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32B32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 0,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: s!("TEXCOORD"),
                    SemanticIndex: 0,
                    Format: DXGI_FORMAT_R32G32_FLOAT,
                    InputSlot: 0,
                    AlignedByteOffset: 12,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                },
            ];
            let mut layout = None;
            device.CreateInputLayout(&elements, &vs_code, Some(&mut layout))?;

            let mut constants = None;
            device.CreateBuffer(
                &D3D11_BUFFER_DESC {
                    ByteWidth: size_of::<FrameConstants>() as u32,
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
                    ..Default::default()
                },
                None,
                Some(&mut constants),
            )?;

            let sampler = |filter| -> Result<ID3D11SamplerState> {
                let mut s = None;
                device.CreateSamplerState(
                    &D3D11_SAMPLER_DESC {
                        Filter: filter,
                        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                        MaxLOD: f32::MAX,
                        ..Default::default()
                    },
                    Some(&mut s),
                )?;
                Ok(s.unwrap())
            };
            let nearest = sampler(D3D11_FILTER_MIN_MAG_MIP_POINT)?;
            let linear = sampler(D3D11_FILTER_MIN_MAG_MIP_LINEAR)?;

            let mut blend = D3D11_BLEND_DESC::default();
            blend.RenderTarget[0] = D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: true.into(),
                SrcBlend: D3D11_BLEND_ONE,
                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOp: D3D11_BLEND_OP_ADD,
                SrcBlendAlpha: D3D11_BLEND_ONE,
                DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
            };
            let mut blend_over = None;
            device.CreateBlendState(&blend, Some(&mut blend_over))?;

            // Reversed depth: cleared to 0 (infinitely far), nearer is greater.
            let depth = |write| -> Result<ID3D11DepthStencilState> {
                let mut d = None;
                device.CreateDepthStencilState(
                    &D3D11_DEPTH_STENCIL_DESC {
                        DepthEnable: true.into(),
                        DepthWriteMask: if write { D3D11_DEPTH_WRITE_MASK_ALL } else { D3D11_DEPTH_WRITE_MASK_ZERO },
                        DepthFunc: D3D11_COMPARISON_GREATER,
                        ..Default::default()
                    },
                    Some(&mut d),
                )?;
                Ok(d.unwrap())
            };
            let depth_write = depth(true)?;
            let depth_test = depth(false)?;

            let mut raster = None;
            device.CreateRasterizerState(
                &D3D11_RASTERIZER_DESC {
                    FillMode: D3D11_FILL_SOLID,
                    // A terrain is seen from both sides when the camera dips under a cliff.
                    CullMode: D3D11_CULL_NONE,
                    DepthClipEnable: true.into(),
                    MultisampleEnable: true.into(),
                    ..Default::default()
                },
                Some(&mut raster),
            )?;

            let samples = pick_samples(&device);
            let mut r = Renderer {
                device,
                ctx,
                d2d,
                dwrite,
                brush,
                factory,
                stroke,
                target,
                views: None,
                width,
                height,
                scale,
                samples,
                unresolved: false,
                lost: false,
                occluded: false,
                vs: vs.unwrap(),
                ps_image: ps_image.unwrap(),
                ps_flat: ps_flat.unwrap(),
                layout: layout.unwrap(),
                constants: constants.unwrap(),
                nearest,
                linear,
                blend_over: blend_over.unwrap(),
                depth_write,
                depth_test,
                raster: raster.unwrap(),
                meshes: HashMap::new(),
                images: HashMap::new(),
                bitmaps: HashMap::new(),
                formats: HashMap::new(),
                fonts,
            };
            r.make_views()?;
            Ok(r)
        }
    }

    fn picture(&self) -> Result<ID3D11Texture2D> {
        match &self.target {
            Target::Window(chain) => unsafe { chain.GetBuffer(0) },
            Target::Offscreen(t) => Ok(t.clone()),
        }
    }

    fn make_views(&mut self) -> Result<()> {
        unsafe {
            let picture = self.picture()?;
            let (w, h) = (self.width.max(1), self.height.max(1));
            let ms = self.samples > 1;

            // With multisampling the 3D pass cannot draw on the picture (one sample a pixel): it gets an
            // image of its own, with a depth buffer of the same count, and is resolved into the picture.
            let multisampled = if ms {
                let mut t = None;
                self.device.CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: w,
                        Height: h,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: PICTURE,
                        SampleDesc: DXGI_SAMPLE_DESC { Count: self.samples, Quality: 0 },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
                        ..Default::default()
                    },
                    None,
                    Some(&mut t),
                )?;
                t
            } else {
                None
            };
            let mut rtv = None;
            self.device.CreateRenderTargetView(multisampled.as_ref().unwrap_or(&picture), None, Some(&mut rtv))?;

            let mut depth = None;
            self.device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: w,
                    Height: h,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DEPTH,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: self.samples, Quality: 0 },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: D3D11_BIND_DEPTH_STENCIL.0 as u32,
                    ..Default::default()
                },
                None,
                Some(&mut depth),
            )?;
            let mut dsv = None;
            self.device.CreateDepthStencilView(&depth.unwrap(), None, Some(&mut dsv))?;

            let dpi = 96.0 * self.scale;
            let bitmap = self.d2d.CreateBitmapFromDxgiSurface(
                &picture.cast::<IDXGISurface>()?,
                Some(&D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT { format: PICTURE, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                    dpiX: dpi,
                    dpiY: dpi,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                    colorContext: ManuallyDrop::new(None),
                }),
            )?;
            self.d2d.SetTarget(&bitmap);
            self.d2d.SetDpi(dpi, dpi);
            self.views = Some(Views { rtv: rtv.unwrap(), dsv: dsv.unwrap(), picture, multisampled, _bitmap: bitmap });
        }
        Ok(())
    }

    /// True once the graphics device is gone (driver reset, card removed, Direct2D asking for a new
    /// target). From then on every call draws nothing and logs nothing more: the window loop must drop
    /// this `Renderer`, build a new one with `for_window` and upload its meshes and images again, since
    /// they lived on the lost device. Set by `end` (Present), `resize` (ResizeBuffers) and `draw_list`
    /// (EndDraw).
    /// The family text is drawn in: IBM Plex Mono, or Consolas when the font files could not be loaded.
    pub fn text_family(&self) -> &'static str {
        if self.fonts.is_some() { crate::fonts::FAMILY } else { crate::fonts::FALLBACK_FAMILY }
    }

    pub fn lost(&self) -> bool {
        self.lost
    }

    /// True when the last Present said the window is not visible.
    pub fn occluded(&self) -> bool {
        self.occluded
    }

    /// Asks the system whether the window shows again, without drawing: a test present. Clears the flag then.
    pub fn still_occluded(&mut self) -> bool {
        if let Target::Window(chain) = &self.target {
            let hr = unsafe { chain.Present(0, DXGI_PRESENT_TEST) };
            self.occluded = hr == DXGI_STATUS_OCCLUDED;
        } else {
            self.occluded = false;
        }
        self.occluded
    }

    /// Has the system post `msg` to the window when it becomes visible or covered.
    pub fn watch_occlusion(&self, hwnd: HWND, msg: u32) {
        let factory: Result<IDXGIFactory2> = unsafe { self.device.cast::<IDXGIDevice>().and_then(|d| d.GetAdapter()?.GetParent()) };
        match factory.and_then(|f| unsafe { f.RegisterOcclusionStatusWindow(hwnd, msg) }) {
            Ok(_) => {}
            Err(e) => observe!(Warn, "occlusion status not watched: {e}"),
        }
    }

    /// For tests: flags the device as lost, as a failed Present would.
    #[doc(hidden)]
    pub fn mark_lost(&mut self) {
        self.lost = true;
    }

    /// Records a failure, and flags the device as lost when the error says so.
    fn failed(&mut self, what: &str, e: &Error) {
        let code = e.code();
        let gone = code == DXGI_ERROR_DEVICE_REMOVED
            || code == DXGI_ERROR_DEVICE_RESET
            || code == D2DERR_RECREATE_TARGET
            // Some drivers report a vanished device through other errors; the device knows for sure.
            || unsafe { self.device.GetDeviceRemovedReason() }.is_err();
        if gone {
            if !self.lost {
                observe!(Error, "{what}: device lost ({e}), the renderer must be rebuilt");
            }
            self.lost = true;
        } else {
            observe!(Error, "{what}: {e}");
        }
    }

    /// Copies the multisampled 3D pass into the picture. Whatever Direct2D drew there before is
    /// replaced, so the interface must come after the last mesh of a frame.
    fn resolve(&mut self) {
        if !self.unresolved {
            return;
        }
        self.unresolved = false;
        if let Some(Views { picture, multisampled: Some(ms), .. }) = &self.views {
            unsafe { self.ctx.ResolveSubresource(picture, 0, ms, 0, PICTURE) };
        }
    }

    /// A new size or scale for a window's picture. Every view on the old images must be released first, or
    /// the swap chain refuses to resize.
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) -> Result<()> {
        if width == 0 || height == 0 || self.lost {
            return Ok(());
        }
        // Without views the last resize failed halfway: the same size must be tried again, not skipped.
        if (width, height, scale) == (self.width, self.height, self.scale) && self.views.is_some() {
            return Ok(());
        }
        unsafe {
            self.d2d.SetTarget(None);
            self.ctx.OMSetRenderTargets(None, None);
            self.views = None;
            self.ctx.Flush();
            let resized = if let Target::Window(chain) = &self.target {
                chain.ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))
            } else {
                picture_texture(&self.device, width, height).map(|t| self.target = Target::Offscreen(t))
            };
            if let Err(e) = resized {
                // The buffers kept their old size, so the stored size stays the old one too.
                self.failed("resize buffers", &e);
                return Err(e);
            }
        }
        // From here the buffers have the new size, whatever happens to the views.
        (self.width, self.height, self.scale) = (width, height, scale);
        let made = self.make_views();
        if let Err(e) = &made {
            self.failed("make views", e);
        }
        made
    }
    /// The offscreen picture as RGBA rows, top to bottom.
    pub fn read_rgba(&self) -> Result<Vec<u8>> {
        let Target::Offscreen(texture) = &self.target else {
            panic!("read_rgba is for offscreen renderers");
        };
        unsafe {
            // The GPU's own texture cannot be read by the CPU: copy it into one made for reading.
            let mut staging = None;
            self.device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: self.width,
                    Height: self.height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: PICTURE,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                    Usage: D3D11_USAGE_STAGING,
                    CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                    ..Default::default()
                },
                None,
                Some(&mut staging),
            )?;
            let staging = staging.unwrap();
            self.ctx.CopyResource(&staging, texture);
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            self.ctx.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;
            let (w, h) = (self.width as usize, self.height as usize);
            let mut out = vec![0u8; w * h * 4];
            for y in 0..h {
                // Rows may be padded: RowPitch is the real distance between two rows.
                let row = std::slice::from_raw_parts((mapped.pData as *const u8).add(y * mapped.RowPitch as usize), w * 4);
                for x in 0..w {
                    let (b, g, r, a) = (row[x * 4], row[x * 4 + 1], row[x * 4 + 2], row[x * 4 + 3]);
                    out[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[r, g, b, a]);
                }
            }
            self.ctx.Unmap(&staging, 0);
            Ok(out)
        }
    }

    fn format(&mut self, size: f32, weight: Weight) -> Option<IDWriteTextFormat> {
        let key = ((size * 64.0) as u32, weight);
        let dw_weight = match weight {
            Weight::Regular => DWRITE_FONT_WEIGHT_NORMAL,
            Weight::SemiBold => DWRITE_FONT_WEIGHT_SEMI_BOLD,
        };
        if let Some(f) = self.formats.get(&key) {
            return Some(f.clone());
        }
        let made = unsafe {
            match &self.fonts {
                Some(fonts) => self
                    .dwrite
                    .CreateTextFormat(
                        w!("IBM Plex Mono"),
                        &fonts.collection,
                        dw_weight,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        size,
                        w!("en-us"),
                    ),
                None => self.dwrite.CreateTextFormat(
                    w!("Consolas"),
                    None,
                    dw_weight,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    size,
                    w!("en-us"),
                ),
            }
        };
        match made {
            Ok(f) => {
                self.formats.insert(key, f.clone());
                Some(f)
            }
            Err(e) => {
                observe!(Error, "text format at {size}: {e}");
                None
            }
        }
    }

    fn try_draw_mesh(&mut self, viewport: Rect, id: MeshId, view_proj: &Mat4, material: Material) -> Result<()> {
        let Some(mesh) = self.meshes.get(&id) else {
            observe!(Warn, "mesh {:?} drawn before it was uploaded", id);
            return Ok(());
        };
        let Some(views) = &self.views else { return Ok(()) };
        let s = self.scale;
        unsafe {
            let c = &self.ctx;
            c.OMSetRenderTargets(Some(&[Some(views.rtv.clone())]), &views.dsv);
            c.RSSetViewports(Some(&[D3D11_VIEWPORT {
                TopLeftX: viewport.x * s,
                TopLeftY: viewport.y * s,
                Width: viewport.w * s,
                Height: viewport.h * s,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            }]));
            c.RSSetState(&self.raster);
            c.IASetInputLayout(&self.layout);
            c.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            let stride = size_of::<Vertex>() as u32;
            let offset = 0u32;
            c.IASetVertexBuffers(0, 1, Some(&Some(mesh.vertices.clone())), Some(&stride), Some(&offset));
            c.IASetIndexBuffer(&mesh.indices, DXGI_FORMAT_R32_UINT, 0);
            let tint = match material {
                Material::Flat { color } => [color.r, color.g, color.b, color.a],
                Material::Image { .. } => [1.0; 4],
            };
            let constants = FrameConstants { view_proj: view_proj.m, tint };
            c.UpdateSubresource(&self.constants, 0, None, &constants as *const _ as *const _, 0, 0);
            c.VSSetShader(&self.vs, None);
            c.VSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            c.PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            match material {
                Material::Image { image, nearest } => {
                    let Some(srv) = self.images.get(&image) else {
                        observe!(Warn, "image {:?} drawn before it was uploaded", image);
                        return Ok(());
                    };
                    c.PSSetShader(&self.ps_image, None);
                    c.PSSetShaderResources(0, Some(&[Some(srv.clone())]));
                    let sampler = if nearest { &self.nearest } else { &self.linear };
                    c.PSSetSamplers(0, Some(&[Some(sampler.clone())]));
                    c.OMSetBlendState(None, None, 0xFFFF_FFFF);
                    c.OMSetDepthStencilState(&self.depth_write, 0);
                }
                Material::Flat { .. } => {
                    c.PSSetShader(&self.ps_flat, None);
                    c.OMSetBlendState(&self.blend_over, None, 0xFFFF_FFFF);
                    // Tested against the land, but leaves no depth: what is drawn after it still shows.
                    c.OMSetDepthStencilState(&self.depth_test, 0);
                }
            }
            c.DrawIndexed(mesh.count, 0, 0);
        }
        Ok(())
    }

    fn try_draw_list(&mut self, list: &DrawList) -> Result<()> {
        unsafe {
            self.d2d.BeginDraw();
            let mut clips = 0usize;
            for cmd in &list.cmds {
                match cmd {
                    Cmd::PushClip { rect } => {
                        // Aliased: the page's clip edges are whole pixels, and a blurred edge would show as a seam.
                        self.d2d.PushAxisAlignedClip(&d2d_rect(*rect), D2D1_ANTIALIAS_MODE_ALIASED);
                        clips += 1;
                    }
                    Cmd::PopClip => {
                        if clips > 0 {
                            self.d2d.PopAxisAlignedClip();
                            clips -= 1;
                        } else {
                            observe!(Warn, "draw list: PopClip without PushClip, ignored");
                        }
                    }
                    Cmd::Fill { rect, color } => {
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.FillRectangle(&d2d_rect(snap_fill(*rect, self.scale)), &self.brush);
                    }
                    Cmd::Shape { rect, grid, figures, color, crisp } => {
                        if figures.is_empty() || *grid <= 0.0 {
                            continue;
                        }
                        let s = self.scale;
                        let geometry = self.factory.CreatePathGeometry()?;
                        let sink = geometry.Open()?;
                        sink.SetFillMode(D2D1_FILL_MODE_WINDING);
                        // The point type belongs to a crate this one may not name, so it is taken from an ellipse's centre.
                        // Crisp icons: the origin lands on a device pixel first, then each vertex's offset from it is
                        // rounded by one rule, so two edges of the same width in the path are the same width on screen
                        // (rounding the absolute position made a 1.33 px stroke 1 or 2 px by where the icon sat).
                        let (ox, oy) = if *crisp { ((rect.x * s).round() / s, (rect.y * s).round() / s) } else { (rect.x, rect.y) };
                        let pt = |p: &mere_base::Vec2| {
                            let mut dx = p.x * rect.w / grid;
                            let mut dy = p.y * rect.h / grid;
                            if *crisp {
                                dx = (dx * s).round() / s;
                                dy = (dy * s).round() / s;
                            }
                            let (x, y) = (ox + dx, oy + dy);
                            let mut v = D2D1_ELLIPSE::default().point;
                            v.X = x;
                            v.Y = y;
                            v
                        };
                        for figure in figures.iter().filter(|f| f.len() >= 2) {
                            sink.BeginFigure(pt(&figure[0]), D2D1_FIGURE_BEGIN_FILLED);
                            let rest: Vec<_> = figure[1..].iter().map(pt).collect();
                            sink.AddLines(&rest);
                            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        }
                        sink.Close()?;
                        self.brush.SetColor(&d2d_color(*color));
                        // Pixel icons are fills of whole device pixels: coverage would grey their edges.
                        if *crisp {
                            self.d2d.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);
                        }
                        self.d2d.FillGeometry(&geometry, &self.brush, None);
                        if *crisp {
                            self.d2d.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
                        }
                    }
                    Cmd::Frame { rect, width_px, color } => {
                        // `width_px` counts logical pixels, so a hairline is one of them: `scale` device
                        // pixels, rounded to whole ones (at least 1) because a fractional stroke would be
                        // blurred by coverage. A line of n device pixels centred on a pixel edge would
                        // straddle two pixels; centring it half a line inside the rectangle, whose edges
                        // are snapped to device pixels, lands it on whole pixels.
                        let device_px = (width_px * self.scale).round().max(1.0);
                        let wd = device_px / self.scale;
                        let half = wd / 2.0;
                        let snapped = snap(*rect, self.scale);
                        let r = D2D_RECT_F {
                            left: snapped.x + half,
                            top: snapped.y + half,
                            right: snapped.x + snapped.w - half,
                            bottom: snapped.y + snapped.h - half,
                        };
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.DrawRectangle(&r, &self.brush, wd, None);
                    }
                    Cmd::Image { image, rect } => {
                        let Some(bitmap) = self.bitmaps.get(image) else {
                            observe!(Warn, "draw list: image {:?} drawn before it was uploaded", image);
                            continue;
                        };
                        // Nearest: the page draws its map with image-rendering: pixelated.
                        self.d2d.DrawBitmap(bitmap, Some(&d2d_rect(*rect)), 1.0, D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR, None, None);
                    }
                    Cmd::Disc { center, radius, color } => {
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.FillEllipse(&ellipse(*center, *radius), &self.brush);
                    }
                    Cmd::Ring { center, radius, width, color } => {
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.DrawEllipse(&ellipse(*center, *radius), &self.brush, *width, None);
                    }
                    Cmd::Polyline { points, width, color } => {
                        if points.len() < 2 {
                            continue;
                        }
                        let geometry = self.factory.CreatePathGeometry()?;
                        let sink = geometry.Open()?;
                        // The point type belongs to a crate this one may not name, so it is taken from an ellipse's centre.
                        let pt = |p: &mere_base::Vec2| {
                            let mut v = D2D1_ELLIPSE::default().point;
                            v.X = p.x;
                            v.Y = p.y;
                            v
                        };
                        sink.BeginFigure(pt(&points[0]), D2D1_FIGURE_BEGIN_HOLLOW);
                        let rest: Vec<_> = points[1..].iter().map(pt).collect();
                        sink.AddLines(&rest);
                        sink.EndFigure(D2D1_FIGURE_END_OPEN);
                        sink.Close()?;
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.DrawGeometry(&geometry, &self.brush, *width, &self.stroke);
                    }
                    Cmd::Text { at, text, size, color, weight } => {
                        let Some(format) = self.format(*size, *weight) else { continue };
                        let wide: Vec<u16> = text.encode_utf16().collect();
                        let r = D2D_RECT_F { left: at.x, top: at.y, right: at.x + 100_000.0, bottom: at.y + size * 2.0 };
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.DrawText(
                            &wide,
                            &format,
                            &r,
                            &self.brush,
                            D2D1_DRAW_TEXT_OPTIONS_NONE,
                            DWRITE_MEASURING_MODE_NATURAL,
                        );
                    }
                }
            }
            // Direct2D refuses EndDraw with clips left open.
            if clips > 0 {
                observe!(Warn, "draw list: {clips} clip(s) never popped, closed at the end");
            }
            for _ in 0..clips {
                self.d2d.PopAxisAlignedClip();
            }
            self.d2d.EndDraw(None, None)
        }
    }
}

impl Gfx for Renderer {
    fn size_px(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn scale(&self) -> f32 {
        self.scale
    }

    fn upload_image(&mut self, id: ImageId, width: u32, height: u32, rgba: &[u8]) {
        // The driver reads width*height*4 bytes from the pointer: a short slice would be read past its end.
        let expected = (width as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4));
        if width == 0 || height == 0 || expected != Some(rgba.len()) {
            observe!(Error, "image {:?} of {width}x{height} refused: {} bytes given, {:?} expected", id, rgba.len(), expected);
            return;
        }
        // Direct2D wants premultiplied B8G8R8A8; the texture keeps the application's RGBA.
        let mut bgra = Vec::with_capacity(rgba.len());
        for p in rgba.chunks_exact(4) {
            let a = p[3] as u32;
            let m = |c: u8| if a == 255 { c } else { ((c as u32 * a + 127) / 255) as u8 };
            bgra.extend_from_slice(&[m(p[2]), m(p[1]), m(p[0]), p[3]]);
        }
        let bitmap = unsafe {
            self.d2d.CreateBitmap(
                D2D_SIZE_U { width, height },
                Some(bgra.as_ptr() as *const _),
                width * 4,
                &D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT { format: PICTURE, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: ManuallyDrop::new(None),
                },
            )
        };
        match bitmap {
            Ok(b) => {
                self.bitmaps.insert(id, b);
            }
            Err(e) => observe!(Error, "bitmap {:?} of {width}x{height}: {e}", id),
        }
        let made = unsafe {
            let mut texture = None;
            let init = D3D11_SUBRESOURCE_DATA { pSysMem: rgba.as_ptr() as *const _, SysMemPitch: width * 4, SysMemSlicePitch: 0 };
            self.device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: width,
                        Height: height,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                        ..Default::default()
                    },
                    Some(&init),
                    Some(&mut texture),
                )
                .and_then(|_| {
                    let mut srv = None;
                    self.device.CreateShaderResourceView(&texture.unwrap(), None, Some(&mut srv)).map(|_| srv.unwrap())
                })
        };
        match made {
            Ok(srv) => {
                self.images.insert(id, srv);
            }
            Err(e) => observe!(Error, "image {:?} of {width}x{height}: {e}", id),
        }
    }

    fn upload_mesh(&mut self, id: MeshId, vertices: &[Vertex], indices: &[u32]) {
        let buffer = |bytes: &[u8], bind: D3D11_BIND_FLAG| -> Result<ID3D11Buffer> {
            let mut b = None;
            unsafe {
                self.device.CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: bytes.len() as u32,
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: bind.0 as u32,
                        ..Default::default()
                    },
                    Some(&D3D11_SUBRESOURCE_DATA { pSysMem: bytes.as_ptr() as *const _, ..Default::default() }),
                    Some(&mut b),
                )?;
            }
            Ok(b.unwrap())
        };
        let as_bytes = |p: *const u8, n: usize| unsafe { std::slice::from_raw_parts(p, n) };
        let v = buffer(as_bytes(vertices.as_ptr() as _, size_of_val(vertices)), D3D11_BIND_VERTEX_BUFFER);
        let i = buffer(as_bytes(indices.as_ptr() as _, size_of_val(indices)), D3D11_BIND_INDEX_BUFFER);
        match (v, i) {
            (Ok(vertices), Ok(idx)) => {
                self.meshes.insert(id, Mesh { vertices, indices: idx, count: indices.len() as u32 });
            }
            (Err(e), _) | (_, Err(e)) => observe!(Error, "mesh {:?}: {e}", id),
        }
    }

    fn begin(&mut self, ground: Color) {
        if self.lost {
            return;
        }
        let Some(views) = &self.views else { return };
        self.unresolved = true;
        unsafe {
            self.ctx.ClearRenderTargetView(&views.rtv, &[ground.r, ground.g, ground.b, ground.a]);
            self.ctx.ClearDepthStencilView(&views.dsv, D3D11_CLEAR_DEPTH.0 as u32, 0.0, 0);
        }
    }

    fn draw_mesh(&mut self, viewport: Rect, mesh: MeshId, view_proj: &Mat4, material: Material) {
        if self.lost {
            return;
        }
        // A mesh drawn after a list is resolved over that list again: lists belong after the meshes.
        self.unresolved = true;
        if let Err(e) = self.try_draw_mesh(viewport, mesh, view_proj, material) {
            self.failed(&format!("draw mesh {mesh:?}"), &e);
        }
    }

    /// Meshes first, lists after: the multisampled 3D pass is resolved into the picture on the first
    /// list of a frame, and the interface is drawn on that.
    fn draw_list(&mut self, list: &DrawList) {
        if self.lost {
            return;
        }
        self.resolve();
        if let Err(e) = self.try_draw_list(list) {
            self.failed("draw list", &e);
        }
    }

    /// Shows the frame. After it, `Renderer::lost` says whether the device went away and the renderer has
    /// to be rebuilt.
    fn end(&mut self) {
        if self.lost {
            return;
        }
        self.resolve();
        let presented = match &self.target {
            // Waits for the next refresh of the screen: one frame a refresh at most, never torn. A success
            // code says the window is covered (`.ok()` would hide it).
            Target::Window(chain) => {
                let hr = unsafe { chain.Present(1, DXGI_PRESENT(0)) };
                self.occluded = hr == DXGI_STATUS_OCCLUDED;
                hr.ok()
            }
            Target::Offscreen(_) => {
                unsafe { self.ctx.Flush() };
                Ok(())
            }
        };
        if let Err(e) = presented {
            self.failed("present", &e);
        }
    }
}

fn create_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    let try_kind = |kind| {
        let (mut device, mut ctx) = (None, None);
        unsafe {
            D3D11CreateDevice(
                None,
                kind,
                HMODULE::default(),
                // Direct2D can draw on a Direct3D texture only when the device takes B8G8R8A8.
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut ctx),
            )
            .map(|_| (device.unwrap(), ctx.unwrap()))
        }
    };
    // WARP is Windows' software renderer: slower, but there when no graphics card answers (a remote
    // session, a test machine), so a picture can always be taken.
    try_kind(D3D_DRIVER_TYPE_HARDWARE).or_else(|e| {
        observe!(Warn, "no hardware device ({e}), using WARP");
        try_kind(D3D_DRIVER_TYPE_WARP)
    })
}

/// The largest sample count up to `WANTED_SAMPLES` that both the picture and the depth format support;
/// 1 (no antialiasing) when none does.
fn pick_samples(device: &ID3D11Device) -> u32 {
    let supported = |format, count| unsafe { device.CheckMultisampleQualityLevels(format, count) }.is_ok_and(|levels| levels > 0);
    let mut count = WANTED_SAMPLES;
    while count > 1 {
        if supported(PICTURE, count) && supported(DEPTH, count) {
            return count;
        }
        count /= 2;
    }
    observe!(Warn, "no multisampling up to {WANTED_SAMPLES}x, drawing without antialiasing");
    1
}

fn picture_texture(device: &ID3D11Device, width: u32, height: u32) -> Result<ID3D11Texture2D> {
    let mut t = None;
    unsafe {
        device.CreateTexture2D(
            &D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: PICTURE,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                ..Default::default()
            },
            None,
            Some(&mut t),
        )?;
    }
    Ok(t.unwrap())
}

/// Shaders are compiled when the renderer starts, by the compiler Windows ships (d3dcompiler_47.dll): no
/// build step, and the source sits above in plain text. It takes a few milliseconds.
fn compile(entry: PCSTR, profile: PCSTR) -> Result<Vec<u8>> {
    let (mut code, mut errors) = (None, None);
    let done = unsafe {
        D3DCompile(
            SHADERS.as_ptr() as *const _,
            SHADERS.len(),
            s!("mere"),
            None,
            None,
            entry,
            profile,
            0,
            0,
            &mut code,
            Some(&mut errors),
        )
    };
    if let Err(e) = done {
        let why = errors
            .map(|b| unsafe { String::from_utf8_lossy(std::slice::from_raw_parts(b.GetBufferPointer() as *const u8, b.GetBufferSize())).into_owned() })
            .unwrap_or_default();
        observe!(Error, "shader {}: {why}", unsafe { entry.display() });
        return Err(e);
    }
    let blob = code.unwrap();
    Ok(unsafe { std::slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()).to_vec() })
}

fn d2d_color(c: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r: c.r, g: c.g, b: c.b, a: c.a }
}

fn ellipse(center: mere_base::Vec2, radius: f32) -> D2D1_ELLIPSE {
    // The point type comes from a crate this one may not name, so it is built by Default and filled in.
    let mut e = D2D1_ELLIPSE { radiusX: radius, radiusY: radius, ..Default::default() };
    e.point.X = center.x;
    e.point.Y = center.y;
    e
}

fn d2d_rect(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F { left: r.x, top: r.y, right: r.x + r.w, bottom: r.y + r.h }
}

/// A filled box as Chrome snaps it. A rule (1 DIP or less, above 0) keeps its start rounded and at least one
/// device pixel of length, which matches `Frame`'s hairline; any other edge is rounded.
fn snap_fill(r: Rect, scale: f32) -> Rect {
    let axis = |origin: f32, len: f32| -> (f32, f32) {
        if len > 0.0 && len <= 1.0 {
            let start = (origin * scale).round();
            (start / scale, (len * scale).round().max(1.0) / scale)
        } else {
            let (a, b) = ((origin * scale).round(), ((origin + len) * scale).round());
            (a / scale, (b - a) / scale)
        }
    };
    let (x, w) = axis(r.x, r.w);
    let (y, h) = axis(r.y, r.h);
    Rect::new(x, y, w, h)
}

/// The rectangle moved to whole physical pixels.
fn snap(r: Rect, scale: f32) -> Rect {
    let f = |v: f32| (v * scale).round() / scale;
    let (x, y) = (f(r.x), f(r.y));
    Rect::new(x, y, f(r.x + r.w) - x, f(r.y + r.h) - y)
}
