use std::{cell::RefCell, collections::HashMap, rc::Rc};
use component::Component;
use glam::{Mat4, Vec2, Vec3};
use resource::{Resource, ResourceHandle};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BufferUsages};
use xenofrost_macros::{get_resource_id, world_query, Component};

use super::{input_manager::{self, InputManager}, render_engine::{camera::{Camera, CameraBindGroupLayout, CameraProjection, CameraUniform, OrthographicProjection}, mesh::QuadMesh, pipeline::Pipeline2D, AspectRatio, DrawMesh, InstanceRaw, RenderCircle, RenderCircleInstances, RenderEngine}};

pub mod component;
pub mod resource;

#[derive(Component)]
struct Transform2D {
    translation: Vec2,
    scale: Vec2,
    rotation: f32,
}

type EntityComponentMap = HashMap<Entity, Rc<RefCell<dyn Component>>>;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Entity(pub u64);

impl Into<u64> for Entity {
    fn into(self) -> u64 {
        self.0
    }
}

pub struct WorldHandler {
    systems: Vec<Box<dyn Fn(&mut World)>>,
    prepare_systems: Vec<Box<dyn Fn(&mut World)>>,
    render_systems: Vec<Box<dyn Fn(&mut World) -> Result<(), wgpu::SurfaceError>>>,
}

impl WorldHandler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            prepare_systems: Vec::new(),
            render_systems: Vec::new()
        }
    }

    pub fn initialize(&mut self, world: &mut World) {
        let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();

        self.add_system(Box::new(camera_controller_system));
        self.add_prepare_system(Box::new(camera_prepare_system));
        self.add_prepare_system(Box::new(circle_prepare_system));
        self.add_render_system(Box::new(circles_render_system));

        let quad_mesh = QuadMesh::new(&render_engine.data().device);
        world.add_resource(quad_mesh);

        let camera_bind_group_layout = CameraBindGroupLayout::new(&render_engine);
        world.add_resource(camera_bind_group_layout);
        
        let pipeline2d = Pipeline2D::new(world);
        world.add_resource(pipeline2d);
        world.add_resource(RenderCircleInstances::new(&render_engine.data().device));

        let aspect_ratio = world.query_resource::<AspectRatio>(get_resource_id!(AspectRatio)).unwrap();

        let camera_entity = world.spawn_entity();
        world.add_component_to_entity(camera_entity, Transform2D {
            translation: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0
        });
        let camera_component = Camera::new(
            "primary_camera", 
            CameraProjection::Orthographic(OrthographicProjection {
                width: 10.0,
                height: 10.0,
                near_clip: 0.1,
                far_clip: 1000.0,
                aspect_ratio: aspect_ratio.data().aspect_ratio
            }), 
            world
        );
        world.add_component_to_entity(camera_entity, camera_component);

        let circle = world.spawn_entity();
        world.add_component_to_entity(circle, RenderCircle);
        world.add_component_to_entity(circle, Transform2D {
            translation: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0
        });
    }

    pub fn update(&self, world: &mut World) {
        for system in self.systems.iter() {
            system(world);
        }

        for prepare_system in self.prepare_systems.iter() {
            prepare_system(world);
        }
    }

    pub fn render(&self, world: &mut World) -> bool {
        for render_system in self.render_systems.iter() {
            match render_system(world) {
                Ok(_) => {},
                //TODO capture the current window size somewhere to resize
                Err(wgpu::SurfaceError::Lost) => self.resize(100, 100, world),
                Err(wgpu::SurfaceError::OutOfMemory) => return false,
                Err(e) => eprintln!("{:?}", e),
            }
        }

        return true
    }

    pub fn resize(&self, new_width: u32, new_height: u32, world: &mut World) {
        let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();
        let aspect_ratio = world.query_resource::<AspectRatio>(get_resource_id!(AspectRatio)).unwrap();

        if new_width > 0 && new_height > 0 {
            render_engine.data_mut().config.width = new_width;
            render_engine.data_mut().config.height = new_height;
            aspect_ratio.data_mut().aspect_ratio = new_width as f32 / new_height as f32;
            render_engine.data().surface.configure(&render_engine.data().device, &render_engine.data().config);

            let camera_query = world_query!(Transform2D, mut Camera);
            for (_, transform2d, mut camera) in camera_query(world).iter() {
                camera.update_aspect_ratio(aspect_ratio.data().aspect_ratio);
                camera.update_uniform_buffer(
                    Vec3::new(transform2d.translation.x, transform2d.translation.y, -1.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    &render_engine.data().queue
                );
            }
        }
    }

    pub fn add_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.systems.push(function);
    }

    pub fn add_prepare_system(&mut self, function: Box<dyn Fn(&mut World)>) {
        self.prepare_systems.push(function);
    }

    pub fn add_render_system(&mut self, function: Box<dyn Fn(&mut World) -> Result<(), wgpu::SurfaceError>>) {
        self.render_systems.push(function);
    }
}

//Note: This struct is also used to create the Render World. If specific World only or RenderWorld only items are required
//      these should be split into two structs.
pub struct World {
    entities: Vec<Entity>,
    components: HashMap<u64, EntityComponentMap>,
    resources: HashMap<u64, Rc<RefCell<dyn Resource>>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
            components: HashMap::new(),
            resources: HashMap::new()
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity = Entity(self.entities.len() as u64);
        self.entities.push(entity);
        entity
    }

    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) -> &mut Self {
        let resource_id = resource.get_resource_id();
        let resource_ref: Rc<RefCell<dyn Resource>> = Rc::new(RefCell::new(resource));

        self.resources.insert(resource_id, resource_ref);

        self
    }

    pub fn query_resource<T: Resource>(&mut self, resource_id: u64) -> Option<ResourceHandle<T>> {
        let result = self.resources.get(&resource_id);
        match result {
            Some(resource) => {
                Some(ResourceHandle::new(Rc::clone(&resource)))
            },
            None => None
        }
    }

    pub fn add_component_to_entity<T: Component>(&mut self, entity: Entity, component: T) -> &mut Self {
        let component_id = component.get_component_id();
        let component_ref: Rc<RefCell<dyn Component>> = Rc::new(RefCell::new(component));

        let component_hash_table_option = self.components.get_mut(&component_id);
        match component_hash_table_option {
            Some(component_hash_table) => {
                component_hash_table.insert(entity, component_ref);
            }
            None => {
                let mut entity_component_hash_map = HashMap::new();
                entity_component_hash_map.insert(entity, component_ref);
                self.components.insert(component_id, entity_component_hash_map);
            }
        }

        self
    }

    pub fn get_entities_with_component(&self, entity_list: &Vec<Entity>, component_id: u64) -> Vec<Entity> {
        let mut result_entity_list: Vec<Entity> = Vec::new();

        let component_hash_map_option = self.components.get(&component_id);
        if let Some(component_hash_map) = component_hash_map_option {
            if entity_list.is_empty() {
                result_entity_list = self.entities.iter().cloned().filter(|entity| component_hash_map.contains_key(entity)).collect();
            }
            else {
                result_entity_list = entity_list.iter().cloned().filter(|entity| component_hash_map.contains_key(entity)).collect();
            }
        }

        result_entity_list
    }

    pub fn query_component(&self, entity: Entity, component_id: u64) -> Option<Rc<RefCell<dyn Component>>> {
        let mut result: Option<Rc<RefCell<dyn Component>>> = None;

        let component_hash_map_option = self.components.get(&component_id);
        if let Some(component_hash_map) = component_hash_map_option {
            let component_option = component_hash_map.get(&entity);
            if let Some(component) = component_option {
                result = Some(Rc::clone(component));
            }
        }

        result
    }
}

fn camera_controller_system(world: &mut World) {
    let speed = 0.01;

    let input_manager_handle = world.query_resource::<InputManager>(get_resource_id!(InputManager)).unwrap();
    let camera_query = world_query!(mut Transform2D, Camera);
    let camera_query_invoke = camera_query(world);
    let (_, mut transform2d, _) = camera_query_invoke.iter().next().unwrap();

    let input_manager = input_manager_handle.data();
    let left_key_state = input_manager.get_key_state("left").unwrap();
    let right_key_state = input_manager.get_key_state("right").unwrap();
    let up_key_state = input_manager.get_key_state("up").unwrap();
    let down_key_state = input_manager.get_key_state("down").unwrap();

    if left_key_state.get_is_down() {
        transform2d.translation.x -= speed;
    }
    if right_key_state.get_is_down() {
        transform2d.translation.x += speed;
    }
    if up_key_state.get_is_down() {
        transform2d.translation.y += speed;
    }
    if down_key_state.get_is_down() {
        transform2d.translation.y -= speed;
    }
}

fn camera_prepare_system(world: &mut World) {
    let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();

    let camera_query = world_query!(Transform2D, mut Camera);
    if let Some((_, transform2d, mut camera)) = camera_query(world).iter().next() {
        camera.update_uniform_buffer(
            Vec3::new(transform2d.translation.x, transform2d.translation.y, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            &render_engine.data().queue
        );
    }
}

fn circle_prepare_system(world: &mut World) {
    let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();
    let circle_instances = world.query_resource::<RenderCircleInstances>(get_resource_id!(RenderCircleInstances)).unwrap();
    let circles_query = world_query!(Transform2D, RenderCircle);

    circle_instances.data_mut().instances.clear();
    for (_, tranform2d, _) in circles_query(world).iter() {
        let raw_instance = InstanceRaw {
            model: Mat4::from_translation(Vec3::new(tranform2d.translation.x, tranform2d.translation.y, 0.0)).to_cols_array_2d()
        };
        circle_instances.data_mut().instances.push(raw_instance);
    }

    if circle_instances.data().instances.len() != circle_instances.data().prev_size {
        circle_instances.data_mut().instances_buffer.destroy();
        let new_instances_buffer = render_engine.data().device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Instance Buffer"),
            contents: bytemuck::cast_slice(&circle_instances.data().instances),
            usage: BufferUsages::VERTEX
        });
        circle_instances.data_mut().instances_buffer = new_instances_buffer;
    }
    else {
        render_engine.data().queue.write_buffer(&circle_instances.data().instances_buffer, 0, bytemuck::cast_slice(&circle_instances.data().instances));
    }


}

fn circles_render_system(world: &mut World) -> Result<(), wgpu::SurfaceError> {
    let render_engine = world.query_resource::<RenderEngine>(get_resource_id!(RenderEngine)).unwrap();
    let pipeline2d = world.query_resource::<Pipeline2D>(get_resource_id!(Pipeline2D)).unwrap();
    let circle_instances = world.query_resource::<RenderCircleInstances>(get_resource_id!(RenderCircleInstances)).unwrap();
    let quad_mesh = world.query_resource::<QuadMesh>(get_resource_id!(QuadMesh)).unwrap();
    let output = render_engine.data().surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let camera_query = world_query!(Camera);
    let camera_query_invoke = camera_query(world);
    let (_, camera) = camera_query_invoke.iter().next().unwrap();

    let mut encoder = render_engine.data().device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&pipeline2d.data().pipeline);
        render_pass.set_vertex_buffer(1, circle_instances.data().instances_buffer.slice(..));
        render_pass.draw_mesh_instanced(&quad_mesh.data().mesh, 0..1 as u32, &camera.camera_bind_group);
    }

    render_engine.data().queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}

#[cfg(test)]
mod tests {
    use glam::Vec3;
    use xenofrost_macros::{get_resource_id, world_query, Component, Resource};

    use super::{component::Component, World, resource::Resource};

    #[derive(Component)]
    struct Test1(u64);
    #[derive(Component)]
    struct Test2(f64);

    #[derive(Component)]
    struct Test3 {
        color: Vec3,
        position: Vec3
    }

    #[derive(Resource, Debug)]
    struct ResourceTest(u64);

    #[test]
    fn query_world_test() {
        let mut world = World::new();
        world.add_resource(ResourceTest(543));

        let resource_handle = world.query_resource::<ResourceTest>(get_resource_id!(ResourceTest)).unwrap();

        let entity1 = world.spawn_entity();
        world.add_component_to_entity(entity1, Test1(1));

        let entity2 = world.spawn_entity();
        world.add_component_to_entity(entity2, Test1(5))
        .add_component_to_entity(entity2, Test2(4.234));


        let entity3 = world.spawn_entity();
        world.add_component_to_entity(entity3, Test2(6.4353))
        .add_component_to_entity(entity3, Test3 {
        color: Vec3::new(1.0, 0.5, 0.5),
        position: Vec3::new(323.0, 434.4, 934.3) 
        });


        let entity4 = world.spawn_entity();
        world.add_component_to_entity(entity4, Test1(99))
        .add_component_to_entity(entity4, Test2(8453.34))
        .add_component_to_entity(entity4, Test3 {
            color: Vec3::new(0.0, 0.0, 1.0),
            position: Vec3::new(342.0, 965.0, 4.0)
        });

        let mut resource_data = resource_handle.data_mut();
        let query1 = world_query!(Test1, Test2, Test3);
        let result1 = query1(&world);
        for (entity, test1, test2, test3) in result1.iter() {
            println!("This is a valid query {} {} {} {} {}", entity.0, test1.0, test2.0, test3.color, test3.position);
            assert_eq!(resource_data.0, 543);
        }

        resource_data.0 = 19;
        let query2 = world_query!(Test1);
        let result2 = query2(&world);
        for (entity, test1) in result2.iter() {
            println!("This is a valid query {} {}", entity.0, test1.0);
            assert_eq!(resource_data.0, 19);
        }

        let query3 = world_query!(Test1, Test2);
        let result3 = query3(&world);
        for (entity, test1, test2) in result3.iter() {
            println!("This is a valid query {} {} {}", entity.0, test1.0, test2.0);
        }

        let query_mut = world_query!(mut Test2, Test3);
        let query_mut_result = query_mut(&world);
        for (entity, mut test2, test3) in query_mut_result.iter() {
            println!("This is a mut pre-query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
            test2.0 = 10.5;
            println!("This is a mut post-query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
        }

        let query4 = world_query!(Test2, Test3);
        let result4 = query4(&world);
        for (entity, test2, test3) in result4.iter() {
            println!("This is a valid query {} {} {} {}", entity.0, test2.0, test3.color, test3.position);
        }
    }
}