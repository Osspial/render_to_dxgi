use glutin_wgl_sys::{wgl, wgl_extra};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    platform::windows::WindowExtWindows,
};
use winapi::{
    shared::{dxgi, dxgitype, dxgiformat, winerror, minwindef, dxgi1_2, dxgi1_3},
    um::{d3dcommon, libloaderapi, winuser, wingdi, d3d11},
};
use std::{ptr, mem, ffi::CString};

fn main() {
    unsafe {
        let event_loop = EventLoop::new();

        let width: u32 = 512;
        let height: u32 = 512;

        let window = WindowBuilder::new()
            .with_title("A fantastic window!")
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .build(&event_loop)
            .unwrap();

        let class_name = "Dummy GL Window\0";
        let class = winuser::WNDCLASSEXA {
            cbSize: mem::size_of::<winuser::WNDCLASSEXA>() as _,
            style: winuser::CS_HREDRAW | winuser::CS_VREDRAW | winuser::CS_OWNDC,
            lpfnWndProc: Some(winuser::DefWindowProcA),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: libloaderapi::GetModuleHandleW(ptr::null()),
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(), // must be null in order for cursor state to work properly
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr() as *const i8,
            hIconSm: ptr::null_mut(),
        };

        winuser::RegisterClassExA(&class);

        let gl_hwnd = winuser::CreateWindowExA(
            0,
            class_name.as_ptr() as *const i8,
            "Dummy Window\0".as_ptr() as *const i8,
            winuser::WS_OVERLAPPEDWINDOW,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            ptr::null_mut(),
            ptr::null_mut(),
            libloaderapi::GetModuleHandleW(ptr::null()),
            ptr::null_mut(),
        );
        assert_ne!(gl_hwnd, ptr::null_mut());

        let gl_hdc = winuser::GetDC(gl_hwnd);
        assert_ne!(gl_hdc, ptr::null_mut());

        {
            let pfd = wingdi::PIXELFORMATDESCRIPTOR {
                nSize: mem::size_of::<wingdi::PIXELFORMATDESCRIPTOR>() as _,
                nVersion: 1,
                dwFlags: wingdi::PFD_SUPPORT_OPENGL,
                ..mem::zeroed()
            };

            let pixel_format = wingdi::ChoosePixelFormat(gl_hdc, &pfd);
            assert!(0 != wingdi::SetPixelFormat(gl_hdc, pixel_format, &pfd));
        }

        let wgl_extra: wgl_extra::Wgl;
        {
            let dummy_context = wgl::CreateContext(gl_hdc as _);
            assert_ne!(ptr::null(), dummy_context);

            wgl::MakeCurrent(gl_hdc as _, dummy_context);
            wgl_extra = wgl_extra::Wgl::load_with(|s| {
                let cs = CString::new(s).unwrap();
                wgl::GetProcAddress(cs.as_ptr()) as *const _
            });
            wgl::DeleteContext(dummy_context);
        }

        let context_attribs = [
            wgl_extra::CONTEXT_MAJOR_VERSION_ARB, 4,
            wgl_extra::CONTEXT_MINOR_VERSION_ARB, 3,
            wgl_extra::CONTEXT_PROFILE_MASK_ARB, wgl_extra::CONTEXT_CORE_PROFILE_BIT_ARB,
            wgl_extra::CONTEXT_FLAGS_ARB, wgl_extra::CONTEXT_DEBUG_BIT_ARB,
            0
        ];

        let context = wgl_extra.CreateContextAttribsARB(gl_hdc as _, ptr::null_mut(), context_attribs.as_ptr() as *const i32);
        assert_ne!(ptr::null(), context);
        wgl::MakeCurrent(gl_hdc as _, context);

        let h_opengl32 = libloaderapi::LoadLibraryA("OpenGL32.dll\0".as_ptr() as *const i8);

        gl::load_with(|s| {
            let cs = CString::new(s).unwrap();
            let f = wgl::GetProcAddress(cs.as_ptr());
            if f != ptr::null_mut() {
                f as _
            } else {
                libloaderapi::GetProcAddress(h_opengl32, cs.as_ptr()) as *const _
            }
        });

        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        use std::os::raw::c_void;
        use glutin_wgl_sys::wgl::types::*;
        if gl::DebugMessageCallback::is_loaded() {
            extern "system" fn debug_callback(
                source: GLenum,
                gltype: GLenum,
                id: GLuint,
                severity: GLenum,
                length: GLsizei,
                message: *const GLchar,
                _userParam: *mut c_void
            ) {
                unsafe {
                    use std::ffi::CStr;
                    let message = CStr::from_ptr(message);
                    println!("{:?}", message);
                    match source {
                        gl::DEBUG_SOURCE_API => println!("Source: API"),
                        gl::DEBUG_SOURCE_WINDOW_SYSTEM => println!("Source: Window System"),
                        gl::DEBUG_SOURCE_SHADER_COMPILER => println!("Source: Shader Compiler"),
                        gl::DEBUG_SOURCE_THIRD_PARTY => println!("Source: Third Party"),
                        gl::DEBUG_SOURCE_APPLICATION => println!("Source: Application"),
                        gl::DEBUG_SOURCE_OTHER => println!("Source: Other"),
                        _ => ()
                    }

                    match gltype {
                        gl::DEBUG_TYPE_ERROR => println!("Type: Error"),
                        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => println!("Type: Deprecated Behaviour"),
                        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => println!("Type: Undefined Behaviour"),
                        gl::DEBUG_TYPE_PORTABILITY => println!("Type: Portability"),
                        gl::DEBUG_TYPE_PERFORMANCE => println!("Type: Performance"),
                        gl::DEBUG_TYPE_MARKER => println!("Type: Marker"),
                        gl::DEBUG_TYPE_PUSH_GROUP => println!("Type: Push Group"),
                        gl::DEBUG_TYPE_POP_GROUP => println!("Type: Pop Group"),
                        gl::DEBUG_TYPE_OTHER => println!("Type: Other"),
                        _ => ()
                    }

                    match severity {
                        gl::DEBUG_SEVERITY_HIGH => println!("Severity: high"),
                        gl::DEBUG_SEVERITY_MEDIUM => println!("Severity: medium"),
                        gl::DEBUG_SEVERITY_LOW => println!("Severity: low"),
                        gl::DEBUG_SEVERITY_NOTIFICATION => println!("Severity: notification"),
                        _ => ()
                    }
                    panic!();
                }
            }
            gl::DebugMessageCallback(debug_callback, 0 as *mut _);
        }

        let mut swap_chain: *mut dxgi1_3::IDXGISwapChain2 = ptr::null_mut();
        let mut device: *mut d3d11::ID3D11Device = ptr::null_mut();
        let mut device_context: *mut d3d11::ID3D11DeviceContext = ptr::null_mut();

        let mut factory: *mut dxgi1_2::IDXGIFactory2 = ptr::null_mut();

        let result = dxgi::CreateDXGIFactory(
            &dxgi1_2::IID_IDXGIFactory2,
            &mut factory as *mut _ as *mut _
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        let result = d3d11::D3D11CreateDevice(
            ptr::null_mut(),
            d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
            ptr::null_mut(),
            d3d11::D3D11_CREATE_DEVICE_DEBUG,
            ptr::null(),
            0,
            d3d11::D3D11_SDK_VERSION,
            &mut device,
            ptr::null_mut(),
            &mut device_context
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        let swap_chain_descriptor = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: minwindef::FALSE,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: dxgi1_2::DXGI_SCALING_STRETCH,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: dxgi::DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
        };

        let result = (*factory).CreateSwapChainForHwnd(
            device as *mut _,
            window.hwnd() as _,
            &swap_chain_descriptor,
            ptr::null(),
            ptr::null_mut(),
            &mut swap_chain as *mut _ as *mut _,
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        let gl_handle_d3d = wgl_extra.DXOpenDeviceNV(device as *mut _);
        assert_ne!(gl_handle_d3d, ptr::null_mut());

        let frame_latency_waitable_object = (*swap_chain).GetFrameLatencyWaitableObject();


        let depth_buffer_descriptor = d3d11::D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: dxgiformat::DXGI_FORMAT_D24_UNORM_S8_UINT,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: d3d11::D3D11_USAGE_DEFAULT,
            BindFlags: d3d11::D3D11_BIND_DEPTH_STENCIL,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut depth_buffer: *mut d3d11::ID3D11Texture2D = ptr::null_mut();
        let result = (*device).CreateTexture2D(
            &depth_buffer_descriptor,
            ptr::null(),
            &mut depth_buffer
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        let depth_stencil_view_descriptor = d3d11::D3D11_DEPTH_STENCIL_VIEW_DESC {
            Format: dxgiformat::DXGI_FORMAT_D24_UNORM_S8_UINT,
            ViewDimension: d3d11::D3D11_DSV_DIMENSION_TEXTURE2D,
            Flags: 0,
            u: mem::zeroed(),
        };
        let mut depth_stencl_view: *mut d3d11::ID3D11DepthStencilView = ptr::null_mut();
        let result = (*device).CreateDepthStencilView(
            depth_buffer as *mut _,
            &depth_stencil_view_descriptor,
            &mut depth_stencl_view
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);


        let mut color_buffer: *mut d3d11::ID3D11Texture2D = ptr::null_mut();
        let result = (*swap_chain).GetBuffer(0, &d3d11::IID_ID3D11Texture2D, &mut color_buffer as *mut _ as *mut _);
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        let mut color_buffer_view: *mut d3d11::ID3D11RenderTargetView = ptr::null_mut();
        let result = (*device).CreateRenderTargetView(
            color_buffer as *mut _,
            ptr::null(),
            &mut color_buffer_view
        );
        assert_eq!(winerror::S_OK, result, "{:x}", result);

        (*device_context).OMSetRenderTargets(1, &color_buffer_view, depth_stencl_view);


        let mut dsv_gl = 0;
        gl::GenRenderbuffers(1, &mut dsv_gl);
        assert_ne!(0, dsv_gl);
        let mut color_gl = 0;
        gl::GenRenderbuffers(1, &mut color_gl);
        assert_ne!(0, color_gl);

        let color_handle_gl = wgl_extra.DXRegisterObjectNV(gl_handle_d3d, color_buffer as *mut _, dsv_gl, gl::RENDERBUFFER, wgl_extra::ACCESS_READ_WRITE_NV);
        assert_ne!(ptr::null(), color_handle_gl);
        let dsv_handle_gl = wgl_extra.DXRegisterObjectNV(gl_handle_d3d, depth_buffer as *mut _, color_gl, gl::RENDERBUFFER, wgl_extra::ACCESS_READ_WRITE_NV);
        assert_ne!(ptr::null(), dsv_handle_gl);

        let mut fbo = 0;
        gl::GenFramebuffers(1, &mut fbo);
        assert_ne!(0, fbo);

        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, color_gl);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::RENDERBUFFER, dsv_gl);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::STENCIL_ATTACHMENT, gl::RENDERBUFFER, dsv_gl);

        match gl::CheckFramebufferStatus(gl::FRAMEBUFFER) {
            gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => println!("incomplete: FRAMEBUFFER_INCOMPLETE_ATTACHMENT"),
            gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => println!("incomplete: FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"),
            o => println!("framebuffer status: {}", o),
        }

        event_loop.run(move |event, _, control_flow| {
            println!("{:?}", event);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => *control_flow = ControlFlow::Exit,
                Event::EventsCleared => {
                    gl::ClearColor(0.0, 0.5, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                _ => *control_flow = ControlFlow::Wait,
            }
        });
    }
}
