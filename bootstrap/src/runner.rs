use std::time::Instant;
use glam::{Vec2, Vec4};
use moonwalk::MoonWalk;
use wgpu;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
    dpi::LogicalSize,
};

#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::app::Application;
use crate::window::WindowSettings;

struct AppState {
    window: &'static Window,
    moonwalk: MoonWalk,
    last_frame_time: Instant,
}

struct AppRunner<A> {
    app: A,
    settings: WindowSettings,
    state: Option<AppState>,
}

impl<A: Application> ApplicationHandler for AppRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes()
            .with_title(&self.settings.title)
            .with_inner_size(LogicalSize::new(self.settings.initial_size.x, self.settings.initial_size.y))
            .with_resizable(self.settings.resizable)
            .with_transparent(self.settings.transparent)
            .with_decorations(self.settings.decorated);

        if let Some(min) = self.settings.min_size {
            attributes = attributes.with_min_inner_size(LogicalSize::new(min.x, min.y));
        }
        
        let window = event_loop.create_window(attributes)
            .expect("Failed to create window");
        let static_window: &'static Window = Box::leak(Box::new(window));

        let initial_size = static_window.inner_size();
        let scale_factor = static_window.scale_factor();
        let logical_size = initial_size.to_logical::<f32>(scale_factor);

        if let Some(state) = &mut self.state {
            state.window = static_window;
            state.moonwalk.recreate_surface(static_window, initial_size.width, initial_size.height);
            state.moonwalk.set_viewport(initial_size.width, initial_size.height);
            state.moonwalk.set_scale_factor(scale_factor as f32); 
            self.app.on_resize(&mut state.moonwalk, Vec2::new(logical_size.width, logical_size.height));
            
            return;
        }

        let mut moonwalk = MoonWalk::new(
            static_window, 
            initial_size.width, 
            initial_size.height
        ).expect("Failed to init MoonWalk");
        
        moonwalk.set_viewport(initial_size.width, initial_size.height);
        moonwalk.set_scale_factor(scale_factor as f32); 

        self.app.on_start(&mut moonwalk, Vec2::new(logical_size.width, logical_size.height));

        self.state = Some(AppState {
            window: static_window,
            moonwalk,
            last_frame_time: Instant::now(),
        });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                self.app.on_exit();
                event_loop.exit();
            },
            
            WindowEvent::Resized(physical_size) => {
                state.moonwalk.set_viewport(physical_size.width, physical_size.height);
                
                let scale = state.window.scale_factor();
                let logical = physical_size.to_logical::<f32>(scale);
                
                self.app.on_resize(&mut state.moonwalk, Vec2::new(logical.width, logical.height));
            },

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                state.moonwalk.set_scale_factor(scale_factor as f32);
            },

            WindowEvent::RedrawRequested => {
                state.window.request_redraw();

                let now = Instant::now();
                let delta_time = now.duration_since(state.last_frame_time).as_secs_f32();
                state.last_frame_time = now;

                self.app.on_update(delta_time);
                self.app.on_draw(&mut state.moonwalk);
                
                match state.moonwalk.render_frame(Vec4::new(0.02, 0.02, 0.05, 1.0)) {
                    Ok(_) => {},
                    
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = state.window.inner_size();
                        if size.width > 0 && size.height > 0 {
                            state.moonwalk.recreate_surface(
                                state.window, 
                                size.width, 
                                size.height
                            );
                        }
                    },
                    
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("Out of memory!");
                        event_loop.exit();
                    },
                    
                    Err(e) => eprintln!("Render error: {}", e),
                }
            },

            _ => {}
        }
    }
}

pub struct Runner;

impl Runner {
    #[cfg(not(target_os = "android"))]
    pub fn run<A: Application + 'static>(app: A, settings: WindowSettings) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut runner = AppRunner {
            app,
            settings,
            state: None,
        };

        event_loop.run_app(&mut runner)?;

        Ok(())
    }

    #[cfg(target_os = "android")]
    pub fn run<A: Application + 'static>(app: A, settings: WindowSettings, android_app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::builder()
            .with_android_app(android_app)
            .build()?;
            
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut runner = AppRunner {
            app,
            settings,
            state: None,
        };

        event_loop.run_app(&mut runner)?;

        Ok(())
    }
}