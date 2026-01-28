local lunar = {
    factory = nil,
    scene = nil,
    display_id = nil,
    cube_mesh = nil,
    floor = nil,
    cube1 = nil,
    cube2 = nil,
    time = 0.0,
    light_id = nil
}

local CUBE_OBJ = [[
v -0.5 -0.5  0.5
v  0.5 -0.5  0.5
v -0.5  0.5  0.5
v  0.5  0.5  0.5
v -0.5  0.5 -0.5
v  0.5  0.5 -0.5
v -0.5 -0.5 -0.5
v  0.5 -0.5 -0.5
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vn  0.0  0.0  1.0
vn  0.0  1.0  0.0
vn  0.0  0.0 -1.0
vn  0.0 -1.0  0.0
vn  1.0  0.0  0.0
vn -1.0  0.0  0.0
f 1/1/1 2/2/1 4/4/1
f 4/4/1 3/3/1 1/1/1
f 3/1/2 4/2/2 6/4/2
f 6/4/2 5/3/2 3/1/2
f 5/1/3 6/2/3 8/4/3
f 8/4/3 7/3/3 5/1/3
f 7/1/4 8/2/4 2/4/4
f 2/4/4 1/3/4 7/1/4
f 2/1/5 8/2/5 6/4/5
f 6/4/5 4/3/5 2/1/5
f 7/1/6 1/2/6 3/4/6
f 3/4/6 5/3/6 7/1/6
]]

bootstrap.set_on_start(function(mw, event)
    lunar.factory = mw:new_lunar_factory()
    
    local meshes = lunar.factory:load_obj(mw, CUBE_OBJ)
    if #meshes == 0 then print("Error loading obj") return end
    lunar.cube_mesh = meshes[1]
    
    local scale = mw:get_scale_factor()
    local phys_w = math.floor(event.width * scale)
    local phys_h = math.floor(event.height * scale)

    lunar.scene = lunar.factory:new_scene(mw, phys_w, phys_h)
    lunar.scene:set_ambient(0.3, 0.3, 0.4)
    lunar.scene:set_shadow_quality(mw, "off")

    lunar.floor = lunar.scene:new_object(lunar.cube_mesh)
    lunar.scene:set_position(lunar.floor, 0.0, -1.5, 0.0)
    lunar.scene:set_scale(lunar.floor, 10.0, 0.1, 10.0)
    lunar.scene:set_color(lunar.floor, 0.7, 0.7, 0.7, 1.0)
    
    lunar.cube1 = lunar.scene:new_object(lunar.cube_mesh)
    lunar.scene:set_position(lunar.cube1, 0.0, 0.0, 0.0)
    lunar.scene:set_scale_uniform(lunar.cube1, 1.5)
    lunar.scene:set_color(lunar.cube1, 0.8, 0.2, 0.2, 1.0)
    lunar.scene:set_metallic(lunar.cube1, 0.0)
    lunar.scene:set_roughness(lunar.cube1, 0.5)

    lunar.cube2 = lunar.scene:new_object(lunar.cube_mesh)
    lunar.scene:set_position(lunar.cube2, 3.0, 0.0, 0.0)
    lunar.scene:set_scale_uniform(lunar.cube2, 1.5)
    lunar.scene:set_color(lunar.cube2, 0.9, 0.9, 0.9, 1.0)
    lunar.scene:set_metallic(lunar.cube2, 1.0)
    lunar.scene:set_roughness(lunar.cube2, 0.3)

    lunar.light_id = lunar.scene:new_light()
    lunar.scene:set_light_pos(lunar.light_id, 10.0, 20.0, 10.0)
    lunar.scene:set_light_color(lunar.light_id, 1.0, 0.95, 0.9)
    lunar.scene:set_light_intensity(lunar.light_id, 2.0)

    lunar.display_id = mw:new_rect()
    mw:set_size(lunar.display_id, event.width, event.height)
end)

bootstrap.set_on_update(function(dt)
    lunar.time = lunar.time + dt
end)

bootstrap.set_on_draw(function(mw)
    if not lunar.scene then return end

    lunar.scene:set_rotation(lunar.cube1, 0.0, lunar.time * 0.3, 0.0)
    lunar.scene:set_rotation(lunar.cube2, 0.0, -lunar.time * 0.2, 0.0)
    
    local sun_x = 10.0 + math.sin(lunar.time * 0.5) * 5.0
    local sun_z = 10.0 + math.cos(lunar.time * 0.5) * 5.0
    lunar.scene:set_light_pos(lunar.light_id, sun_x, 20.0, sun_z)

    local tex_id = lunar.scene:render(mw, lunar.factory)
    mw:set_texture(lunar.display_id, tex_id)
end)

bootstrap.set_on_resize(function(mw, evt)
    local scale = mw:get_scale_factor()
    local phys_w = math.floor(evt.width * scale)
    local phys_h = math.floor(evt.height * scale)

    mw:set_viewport(phys_w, phys_h)
    
    if lunar.display_id then
        mw:set_size(lunar.display_id, evt.width, evt.height)
    end

    lunar.scene:resize(mw, phys_w, phys_h)
end)