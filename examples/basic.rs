use avian3d::{prelude::*, PhysicsPlugins};
use bevy::{
    core_pipeline::tonemapping::Tonemapping, pbr::{NotShadowCaster, NotShadowReceiver}, prelude::*,
    // render::{
    //     RenderApp,
    //     batching::gpu_preprocessing::{GpuPreprocessingSupport, GpuPreprocessingMode}
    // }
};

// Add this
use bevy_trebuchet::{NewTrebuchets, TrebuchetPlugin};


fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK))
    .add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        TrebuchetPlugin::default()
    ))
    .add_systems(Startup, startup);

    // app.sub_app_mut(RenderApp)
    // .insert_resource(GpuPreprocessingSupport {
    //     max_supported_mode: GpuPreprocessingMode::PreprocessingOnly,
    // });

    app.run();
}

fn startup(
    mut cmd: Commands,
    mut al : ResMut<AmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>
) {
    al.brightness = 200.;
    cmd.spawn((
        Camera3d::default(),
        Transform::from_xyz(20., 15., 50.).looking_at(Vec3::ZERO.with_y(10.), Vec3::Y),
        Tonemapping::ReinhardLuminance
    ));
    cmd.spawn((
        DirectionalLight {
            illuminance: 50_000.,
            ..default()
        },
        Transform::IDENTITY.looking_at(Vec3::ZERO, Vec3::Y)
    ));

    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(500.)))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Collider::cuboid(500., 0.1, 500.),
        RigidBody::Static
    ));

    //  Add Trebuchets
    
    cmd.trigger(NewTrebuchets(vec![
        Transform::from_xyz(0., 0., 0.),
        Transform::from_xyz(15., 0., 0.),
    ]));

    let material = materials.add(Color::BLACK);
    let dummy_dim = Vec3::new(4., 4., 4.);
    let mesh_h = meshes.add(Cuboid::from_size(dummy_dim));
    let wall_pos = Vec3::new(0., 0., -150.);    
    for pos in wall(wall_pos + Vec3::new(-44., 0., -40.), dummy_dim, 110) {    
        cmd.spawn((
            Mesh3d(mesh_h.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
            RigidBody::Dynamic,
            Restitution::new(0.1).with_combine_rule(CoefficientCombine::Min),
            Collider::cuboid(dummy_dim.x, dummy_dim.y, dummy_dim.z),
            ColliderDensity(0.1),
            NotShadowCaster,
            NotShadowReceiver
        ));
    }


}

// ---

fn wall(start: Vec3, dummy_dim: Vec3, dummies_count: u32) -> impl Iterator<Item = Vec3> {
    let mut pos = start.with_y(dummy_dim.y * 0.55); 
    let mut step = Vec3::X * 1.1 * dummy_dim.z.max(dummy_dim.x) ;
    let mut in_row = 0;
    let mut row_cap = 20;
    (0 .. dummies_count).map(move |_| {
        if in_row == row_cap {
            row_cap -= 2; 
            in_row = 0;
            step *= -1.;
            pos.y += 1.1 * dummy_dim.y;
        } 
        in_row += 1;
        pos += step;
        pos
    })
} 
