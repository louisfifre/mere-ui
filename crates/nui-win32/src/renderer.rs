use std::collections::HashMap;
use std::mem::ManuallyDrop;

use nui_base::{Cmd, Color, DrawList, Gfx, ImageId, Mat4, Material, MeshId, Rect, Vertex, observe};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::core::{Interface, PCSTR, Result, s, w};

// Direct2D draws only into B8G8R8A8 surfaces, so the picture is in that order; textures uploaded by the
// application stay RGBA, which a shader reads the same way.
const PICTURE: DXGI_FORMAT = DXGI_FORMAT_B8G8R8A8_UNORM;

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
    rtv: ID3D11RenderTargetView,
    dsv: ID3D11DepthStencilView,
    // Kept so Direct2D draws into the same texture as Direct3D.
    _bitmap: ID2D1Bitmap1,
}

pub struct Renderer {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    d2d: ID2D1DeviceContext,
    dwrite: IDWriteFactory,
    brush: ID2D1SolidColorBrush,
    target: Target,
    views: Option<Views>,
    width: u32,
    height: u32,
    scale: f32,
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
    // One text format a size: DirectWrite formats are immutable, and sizes are few.
    formats: HashMap<u32, IDWriteTextFormat>,
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
            let brush = d2d.CreateSolidColorBrush(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, None)?;

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
                    ..Default::default()
                },
                Some(&mut raster),
            )?;

            let mut r = Renderer {
                device,
                ctx,
                d2d,
                dwrite,
                brush,
                target,
                views: None,
                width,
                height,
                scale,
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
                formats: HashMap::new(),
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
            let mut rtv = None;
            self.device.CreateRenderTargetView(&picture, None, Some(&mut rtv))?;

            let mut depth = None;
            self.device.CreateTexture2D(
                &D3D11_TEXTURE2D_DESC {
                    Width: self.width.max(1),
                    Height: self.height.max(1),
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_D32_FLOAT,
                    SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
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
            self.views = Some(Views { rtv: rtv.unwrap(), dsv: dsv.unwrap(), _bitmap: bitmap });
        }
        Ok(())
    }

    /// A new size or scale for a window's picture. Every view on the old images must be released first, or
    /// the swap chain refuses to resize.
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) -> Result<()> {
        if (width, height, scale) == (self.width, self.height, self.scale) || width == 0 || height == 0 {
            return Ok(());
        }
        unsafe {
            self.d2d.SetTarget(None);
            self.ctx.OMSetRenderTargets(None, None);
            self.views = None;
            self.ctx.Flush();
            match &self.target {
                Target::Window(chain) => chain.ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))?,
                Target::Offscreen(_) => self.target = Target::Offscreen(picture_texture(&self.device, width, height)?),
            }
        }
        (self.width, self.height, self.scale) = (width, height, scale);
        self.make_views()
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

    fn format(&mut self, size: f32) -> Option<IDWriteTextFormat> {
        let key = (size * 64.0) as u32;
        if let Some(f) = self.formats.get(&key) {
            return Some(f.clone());
        }
        // Consolas until IBM Plex Mono is loaded from the application's own file (next step).
        let made = unsafe {
            self.dwrite.CreateTextFormat(
                w!("Consolas"),
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )
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
            for cmd in &list.cmds {
                match cmd {
                    Cmd::Fill { rect, color } => {
                        self.brush.SetColor(&d2d_color(*color));
                        self.d2d.FillRectangle(&d2d_rect(*rect), &self.brush);
                    }
                    Cmd::Frame { rect, width_px, color } => {
                        // A line of n physical pixels centred on a pixel edge would straddle two pixels and
                        // blur; centring it half a line inside the rectangle lands it on whole pixels.
                        let wd = width_px / self.scale;
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
                    Cmd::Text { at, text, size, color } => {
                        let Some(format) = self.format(*size) else { continue };
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
        let Some(views) = &self.views else { return };
        unsafe {
            self.ctx.ClearRenderTargetView(&views.rtv, &[ground.r, ground.g, ground.b, ground.a]);
            self.ctx.ClearDepthStencilView(&views.dsv, D3D11_CLEAR_DEPTH.0 as u32, 0.0, 0);
        }
    }

    fn draw_mesh(&mut self, viewport: Rect, mesh: MeshId, view_proj: &Mat4, material: Material) {
        if let Err(e) = self.try_draw_mesh(viewport, mesh, view_proj, material) {
            observe!(Error, "draw mesh {:?}: {e}", mesh);
        }
    }

    fn draw_list(&mut self, list: &DrawList) {
        if let Err(e) = self.try_draw_list(list) {
            observe!(Error, "draw list: {e}");
        }
    }

    fn end(&mut self) {
        match &self.target {
            Target::Window(chain) => {
                // Waits for the next refresh of the screen: one frame a refresh at most, never torn.
                let hr = unsafe { chain.Present(1, DXGI_PRESENT(0)) };
                if hr.is_err() {
                    observe!(Error, "present: {hr:?}");
                }
            }
            Target::Offscreen(_) => unsafe { self.ctx.Flush() },
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
            s!("nui"),
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

fn d2d_rect(r: Rect) -> D2D_RECT_F {
    D2D_RECT_F { left: r.x, top: r.y, right: r.x + r.w, bottom: r.y + r.h }
}

/// The rectangle moved to whole physical pixels.
fn snap(r: Rect, scale: f32) -> Rect {
    let f = |v: f32| (v * scale).round() / scale;
    let (x, y) = (f(r.x), f(r.y));
    Rect::new(x, y, f(r.x + r.w) - x, f(r.y + r.h) - y)
}
