// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

use mlua::{UserData, UserDataMethods, Error};
use glam::{Vec3, Vec4, Mat4};
use moonwalk::MoonWalk;

use lunar3d::{LunarFactory, LunarScene, ShadowQuality, LightId};
use lunar3d::resources::MeshData;
use lunar3d::core::types::{MeshId, ObjectId}; 

use super::MoonWalkLuaWrapper;

fn clone_mesh_data(mw: &MoonWalk, src: &MeshData) -> MeshData {
    let vb = mw.create_vertex_buffer(bytemuck::cast_slice(&src.vertices));
    let ib = mw.create_index_buffer_u32(bytemuck::cast_slice(&src.indices));
    
    MeshData {
        vertex_buffer: vb,
        index_buffer: ib,
        index_count: src.indices.len() as u32,
        local_material: src.local_material.clone(),
        vertices: src.vertices.clone(),
        indices: src.indices.clone(),
    }
}

pub struct LuaLunarFactory(pub LunarFactory);

impl UserData for LuaLunarFactory {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load_obj", |_, this, (mw, content): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, String)| {
            let ids = this.0.load_obj(mw.get(), content.as_bytes());
            Ok(ids.into_iter().map(|id| id.0).collect::<Vec<_>>())
        });

        methods.add_method_mut("load_gltf", |_, this, (mw, content): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, String)| {
            let ids = this.0.load_gltf(mw.get_mut(), content.as_bytes());
            Ok(ids.into_iter().map(|id| id.0).collect::<Vec<_>>())
        });

        methods.add_method("new_scene", |_, this, (mw, w, h): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, u32, u32)| {
            let scene = this.0.new_scene(mw.get_mut(), w, h);
            Ok(LuaLunarScene(scene))
        });

        methods.add_method_mut("add_mesh", |_, this, mut mesh: mlua::UserDataRefMut<LuaMeshData>| {
            let data = mesh.0.take().ok_or(Error::RuntimeError("MeshData already consumed".into()))?;
            let id = this.0.add_mesh(data);
            Ok(id.0)
        });

        methods.add_method_mut("register_pipeline", |_, this, (mw, name, src): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, String, String)| {
            this.0.register_pipeline(mw.get(), &name, &src);
            Ok(())
        });
        
        methods.add_method("get_mesh_data", |_, this, (mw, id): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, usize)| {
            if let Some(mesh) = this.0.get_mesh(MeshId(id)) {
                let cloned = clone_mesh_data(mw.get(), mesh);
                Ok(Some(LuaMeshData(Some(cloned))))
            } else {
                Ok(None)
            }
        });
    }
}

pub struct LuaLunarScene(pub LunarScene);

impl UserData for LuaLunarScene {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("render", |_, this, (mw, factory): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, mlua::UserDataRef<LuaLunarFactory>)| {
            let tex_id = this.0.render(mw.get_mut(), &factory.0);
            Ok(tex_id)
        });

        methods.add_method_mut("set_shadow_quality", |_, this, (mw, q): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, String)| {
            let quality = match q.as_str() {
                "high" => ShadowQuality::High,
                "medium" => ShadowQuality::Medium,
                "low" => ShadowQuality::Low,
                _ => ShadowQuality::Off,
            };

            this.0.set_shadow_quality(mw.get_mut(), quality);
            Ok(())
        });

        methods.add_method_mut("set_shadow_pause", |_, this, paused: bool| {
            this.0.set_shadow_pause(paused);
            Ok(())
        });

        methods.add_method_mut("new_object", |_, this, mesh_id: usize| {
            Ok(this.0.new_object(MeshId(mesh_id)).0)
        });

        methods.add_method_mut("remove_object", |_, this, id: usize| {
            this.0.remove_object(ObjectId(id));
            Ok(())
        });

        methods.add_method_mut("remove_all", |_, this, ()| {
            this.0.remove_all();
            Ok(())
        });

        methods.add_method_mut("set_position", |_, this, (id, x, y, z): (usize, f32, f32, f32)| {
            this.0.set_position(ObjectId(id), Vec3::new(x, y, z));
            Ok(())
        });
        
        methods.add_method_mut("set_rotation", |_, this, (id, x, y, z): (usize, f32, f32, f32)| {
            this.0.set_rotation(ObjectId(id), Vec3::new(x, y, z));
            Ok(())
        });

        methods.add_method_mut("set_scale", |_, this, (id, x, y, z): (usize, f32, f32, f32)| {
            this.0.set_scale(ObjectId(id), Vec3::new(x, y, z));
            Ok(())
        });

        methods.add_method_mut("set_scale_uniform", |_, this, (id, s): (usize, f32)| {
            this.0.set_scale(ObjectId(id), Vec3::splat(s));
            Ok(())
        });

        methods.add_method_mut("set_color", |_, this, (id, r, g, b, a): (usize, f32, f32, f32, f32)| {
            this.0.set_color(ObjectId(id), Vec4::new(r, g, b, a));
            Ok(())
        });

        methods.add_method_mut("set_metallic", |_, this, (id, v): (usize, f32)| {
            this.0.set_metallic(ObjectId(id), v);
            Ok(())
        });

        methods.add_method_mut("set_roughness", |_, this, (id, v): (usize, f32)| {
            this.0.set_roughness(ObjectId(id), v);
            Ok(())
        });
        
        methods.add_method_mut("set_texture", |_, this, (id, tex_id): (usize, u32)| {
            this.0.set_texture(ObjectId(id), tex_id);
            Ok(())
        });

        methods.add_method_mut("set_normal_map", |_, this, (id, tex_id): (usize, u32)| {
            this.0.set_normal_map(ObjectId(id), tex_id);
            Ok(())
        });

        methods.add_method_mut("set_unlit", |_, this, (id, v): (usize, bool)| {
            this.0.set_unlit(ObjectId(id), v);
            Ok(())
        });

        methods.add_method_mut("set_layer", |_, this, (id, name): (usize, String)| {
            this.0.set_layer(ObjectId(id), &name);
            Ok(())
        });

        methods.add_method_mut("set_all_layers", |_, this, name: String| {
            this.0.set_all_layers(&name);
            Ok(())
        });

        // --- Getters ---
        methods.add_method("get_position", |_, this, id: usize| {
            let v = this.0.get_position(ObjectId(id));
            Ok((v.x, v.y, v.z))
        });

        methods.add_method("get_rotation", |_, this, id: usize| {
            let v = this.0.get_rotation(ObjectId(id));
            Ok((v.x, v.y, v.z))
        });

        methods.add_method("get_scale", |_, this, id: usize| {
            let v = this.0.get_scale(ObjectId(id));
            Ok((v.x, v.y, v.z))
        });

        methods.add_method("get_color", |_, this, id: usize| {
            let v = this.0.get_color(ObjectId(id));
            Ok((v.x, v.y, v.z, v.w))
        });

        methods.add_method_mut("set_camera", |_, this, (px, py, pz, tx, ty, tz): (f32,f32,f32, f32,f32,f32)| {
            this.0.camera_pos = Vec3::new(px, py, pz);
            this.0.camera_target = Vec3::new(tx, ty, tz);
            Ok(())
        });

        methods.add_method_mut("set_ambient", |_, this, (r, g, b): (f32, f32, f32)| {
            this.0.ambient_color = Vec3::new(r, g, b);
            Ok(())
        });

        methods.add_method_mut("new_light", |_, this, ()| {
            Ok(this.0.new_light().0)
        });

        methods.add_method_mut("remove_light", |_, this, id: usize| {
            this.0.remove_light(LightId(id));
            Ok(())
        });

        methods.add_method_mut("remove_all_lights", |_, this, ()| {
            this.0.remove_all_lights();
            Ok(())
        });

        methods.add_method_mut("set_light_pos", |_, this, (id, x, y, z): (usize, f32, f32, f32)| {
            this.0.set_light_position(LightId(id), Vec3::new(x, y, z));
            Ok(())
        });

        methods.add_method_mut("set_light_color", |_, this, (id, r, g, b): (usize, f32, f32, f32)| {
            this.0.set_light_color(LightId(id), Vec3::new(r, g, b));
            Ok(())
        });

        methods.add_method_mut("set_light_intensity", |_, this, (id, v): (usize, f32)| {
            this.0.set_light_intensity(LightId(id), v);
            Ok(())
        });

        methods.add_method_mut("resize", |_, this, (mw, w, h): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, u32, u32)| {
            this.0.resize(mw.get_mut(), w, h);
            Ok(())
        });
    }
}

// --- Wrapper for MeshData (Tools) ---
// Оборачиваем Option, чтобы можно было "забрать" (consume) меш.
pub struct LuaMeshData(pub Option<MeshData>);

impl UserData for LuaMeshData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        
        // Helper to borrow inner mesh data
        fn get_mesh<'a>(lua_mesh: &'a LuaMeshData) -> mlua::Result<&'a MeshData> {
            lua_mesh.0.as_ref().ok_or_else(|| Error::RuntimeError("MeshData consumed or invalid".into()))
        }

        methods.add_method("cut_mesh", |_, this, (mw, px, py, pz, nx, ny, nz, fill): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, f32,f32,f32, f32,f32,f32, bool)| {
            let mesh = get_mesh(&this)?;
            let point = Vec3::new(px, py, pz);
            let normal = Vec3::new(nx, ny, nz);
            
            if let Some((m1, m2)) = lunar3d::tools::knife::cut_mesh(mw.get(), mesh, point, normal, fill) {
                Ok(Some(vec![LuaMeshData(Some(m1)), LuaMeshData(Some(m2))]))
            } else {
                Ok(None)
            }
        });

        methods.add_method("shatter", |_, this, (mw, iter, seed, fill): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, usize, u64, bool)| {
            let mesh = get_mesh(&this)?;
            let shards = lunar3d::tools::shatter::shatter_mesh(mw.get(), mesh, iter, seed, fill);
            Ok(shards.into_iter().map(|m| LuaMeshData(Some(m))).collect::<Vec<_>>())
        });

        methods.add_method("subdivide", |_, this, (mw, iter): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, usize)| {
            let mesh = get_mesh(&this)?;
            let res = lunar3d::tools::subdivide::subdivide(mw.get(), mesh, iter);
            Ok(LuaMeshData(Some(res)))
        });

        methods.add_method("bake", |_, this, (mw, px,py,pz, rx,ry,rz, sx,sy,sz): (mlua::UserDataRefMut<MoonWalkLuaWrapper>, f32,f32,f32, f32,f32,f32, f32,f32,f32)| {
             let mesh = get_mesh(&this)?;
             let t = Mat4::from_scale_rotation_translation(
                 Vec3::new(sx, sy, sz),
                 glam::Quat::from_euler(glam::EulerRot::XYZ, rx, ry, rz),
                 Vec3::new(px, py, pz)
             );
             let res = lunar3d::tools::transform::bake(mw.get(), mesh, t);
             Ok(LuaMeshData(Some(res)))
        });

        methods.add_method("recalculate_normals", |_, this, mw: mlua::UserDataRefMut<MoonWalkLuaWrapper>| {
            let mesh = get_mesh(&this)?;
            let res = lunar3d::tools::transform::recalculate_normals(mw.get(), mesh);
            Ok(LuaMeshData(Some(res)))
        });
        
        methods.add_method("flip_faces", |_, this, mw: mlua::UserDataRefMut<MoonWalkLuaWrapper>| {
            let mesh = get_mesh(&this)?;
            let res = lunar3d::tools::transform::flip_faces(mw.get(), mesh);
            Ok(LuaMeshData(Some(res)))
        });
    }
}

pub fn register<'lua, M: UserDataMethods<'lua, MoonWalkLuaWrapper>>(methods: &mut M) {
    methods.add_method("new_lunar_factory", |_, this, ()| {
        Ok(LuaLunarFactory(LunarFactory::new(this.get_mut())))
    });

    methods.add_method("joint_meshes", |_, this, meshes: Vec<mlua::UserDataRef<LuaMeshData>>| {
        // Собираем ссылки на MeshData. Если хоть один consumed - ошибка.
        let mut refs = Vec::new();
        for m in &meshes {
            if let Some(inner) = &m.0 {
                refs.push(inner);
            } else {
                return Err(Error::RuntimeError("One of the meshes is consumed".into()));
            }
        }
        
        let res = lunar3d::tools::joint::joint_meshes(this.get(), refs, None);
        Ok(res.map(|m| LuaMeshData(Some(m))))
    });
}
