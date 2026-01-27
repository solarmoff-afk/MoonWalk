use moonwalk::*; 
use moonwalk_bootstrap::{Application, Runner, WindowSettings, TouchPhase};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    mvp: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
}

struct CubeApp {
    custom_paint: Option<CustomPaint>,
    pipeline: Option<CustomPipeline>,
    
    vertex_buffer: Option<MoonBuffer>,
    index_buffer: Option<MoonBuffer>,
    uniform_buffer: Option<MoonBuffer>,
    bind_group: Option<MoonBindGroup>,
    
    render_target_id: u32,
    
    rotation_x: f32,
    rotation_y: f32,
}

impl CubeApp {
    fn new() -> Self {
        Self {
            custom_paint: None,
            pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer: None,
            bind_group: None,
            render_target_id: 0,
            rotation_x: 0.0,
            rotation_y: 0.0,
        }
    }
}

impl Application for CubeApp {
    fn on_start(&mut self, mw: &mut MoonWalk, viewport: Vec2) {
        let scale = mw.get_scale_factor(); 
        let tex_size = (600.0 * scale) as u32;
        let mut cp = mw.new_custom_paint(tex_size, tex_size, "Cube 3D");

        // Куб
        let vertices = [
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 0.0] },
        ];

        let indices: [u16; 36] = [
            0, 1, 2, 2, 3, 0, 1, 5, 6, 6, 2, 1, 5, 4, 7, 7, 6, 5,
            4, 0, 3, 3, 7, 4, 3, 2, 6, 6, 7, 3, 4, 5, 1, 1, 0, 4,
        ];

        self.vertex_buffer = Some(mw.create_vertex_buffer(bytemuck::cast_slice(&vertices)));
        self.index_buffer = Some(mw.create_index_buffer_u16(bytemuck::cast_slice(&indices)));

        let bind_group_desc = BindGroup::new().add_uniform(0, ShaderStage::Vertex);
        let layout = mw.create_bind_group_layout(bind_group_desc).unwrap();

        let pipeline_desc = MoonPipeline::new(SHADER_SOURCE)
            .vertex_shader("vs_main")
            .fragment_shader("fs_main")
            .add_vertex_layout(
                VertexLayout::new()
                    .stride(std::mem::size_of::<Vertex>() as u32)
                    .step_mode(StepMode::Vertex)
                    .add_attr(VertexAttr::new().format(Format::Float32x3).location(0).offset(0))
                    .add_attr(VertexAttr::new().format(Format::Float32x3).location(1).offset(12))
            )
            .cull(CullMode::Back)
            .topology(Topology::TriangleList)
            .depth_test(true)
            .depth_write(true)
            .label("Cube pipeline");

        self.pipeline = Some(mw.compile_pipeline(pipeline_desc, &[&layout]).unwrap());

        let uniforms = Uniforms {
            mvp: Mat4::IDENTITY.to_cols_array_2d()
        };

        let u_buffer = mw.create_uniform_buffer(bytemuck::bytes_of(&uniforms));

        let bg = mw.create_bind_group(&layout, &[BindResource::Uniform(&u_buffer)]).unwrap();
        self.uniform_buffer = Some(u_buffer);
        self.bind_group = Some(bg);
        
        cp.set_render_pass(MoonRenderPass::new().set_clear_color(Some(Vec4::ZERO)));

        self.render_target_id = cp.snapshot(mw);
        self.custom_paint = Some(cp);

        let id = mw.new_rect();
        mw.set_size(id, Vec2::new(600.0, 600.0));
        mw.set_position(id, (viewport - 600.0) / 2.0);
        mw.set_texture(id, self.render_target_id);
    }

    fn on_update(&mut self, dt: f32) {
        self.rotation_x += dt * 1.0;
        self.rotation_y += dt * 0.7;
    }

    fn on_draw(&mut self, mw: &mut MoonWalk) {
        let cp = match &mut self.custom_paint { Some(c) => c, None => return };
        let pipe = match &self.pipeline { Some(p) => p, None => return };
        let vb = match &self.vertex_buffer { Some(v) => v, None => return };
        let ib = match &self.index_buffer { Some(i) => i, None => return };
        let ub = match &self.uniform_buffer { Some(u) => u, None => return };
        let bg = match &self.bind_group { Some(b) => b, None => return };

        let aspect = cp.width as f32 / cp.height as f32;
        let projection = Mat4::perspective_rh(45.0f32.to_radians(), aspect, 0.1, 100.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::ZERO,
            Vec3::Y
        );

        let model = Mat4::from_rotation_x(self.rotation_x) * Mat4::from_rotation_y(self.rotation_y);
        let mvp = projection * view * model;
        let uniform_data = Uniforms { 
            mvp: mvp.to_cols_array_2d()
        };

        mw.update_buffer(ub, bytemuck::bytes_of(&uniform_data));
        cp.set_render_pass(
            MoonRenderPass::new()
                .set_clear_color(Some(Vec4::new(0.1, 0.1, 0.1, 1.0)))
                .set_clear_depth(true)
        );

        cp.set_pipeline(pipe);
        cp.set_bind_group(0, bg);
        cp.set_vertex_buffer(0, vb);
        cp.set_index_buffer(ib);
        cp.draw_indexed(mw, 0..36, 0, 0..1);
        cp.update_snapshot(mw, self.render_target_id);
    }

    fn on_resize(&mut self, _mw: &mut MoonWalk, _viewport: Vec2) {

    }

    fn on_touch(&mut self, _mw: &mut MoonWalk, _phase: TouchPhase, _pos: Vec2) {

    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    let app = CubeApp::new();
    let settings = WindowSettings::new("MoonWalk 3D", 800.0, 600.0);
    Runner::run(app, settings).unwrap();
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: android_activity::AndroidApp) {
    let app_logic = CubeApp::new();
    let settings = WindowSettings::new("MoonWalk 3D", 0.0, 0.0);
    Runner::run(app_logic, settings, app).unwrap();
}
