use bevy::{
    prelude::*,
    pbr::{NotShadowCaster, NotShadowReceiver}
};
use avian3d::prelude::*;
use std::time::Duration;

use crate::TrebuchetSettings;

pub struct BallPlugin;
impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, despawn_on_time.run_if(any_with_component::<LifeTime>))
        .add_systems(Update, despawn_on_collision.run_if(on_event::<CollisionEnded>))
        .add_observer(spawn)
        ;
    }
}

// ---

#[derive(Component)]
pub struct  Ball;

#[derive(Component)]
pub struct  Released;

#[derive(Component)]
pub struct LifeTime(pub Timer);

#[derive(Event)]
pub struct BallSpawn(pub Vec3); 

#[derive(Resource)]
pub struct BallMM(Handle<Mesh>, Handle<StandardMaterial>);


pub const BALL_RADIUS: f32 = 0.55;

// ---

fn startup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.insert_resource(
        BallMM(
            meshes.add(Sphere::new(BALL_RADIUS)),
            materials.add(Color::hsl(150., 1.0, 0.5)),
        )
    );  
}

// ---

fn spawn(
    trigger: Trigger<BallSpawn>,
    mut cmd: Commands,
    mm: Res<BallMM>,
    setings: Res<TrebuchetSettings>
) {
    let event = trigger.event();

    cmd.spawn((
        Mesh3d(mm.0.clone()),
        MeshMaterial3d(mm.1.clone()),
        NotShadowCaster,
        NotShadowReceiver,
        Ball, 
        Name::new("Ball"), 
        Transform::from_translation(event.0),
        LifeTime(Timer::new(Duration::from_secs(60), TimerMode::Once)),
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        Collider::sphere(BALL_RADIUS), 
        ColliderDensity(setings.ball_density), 
        CollisionEventsEnabled,
        CollisionLayers::new(LayerMask(setings.ball_layer_mask), LayerMask::ALL)
    ));
    
    

}

// ---

fn despawn_on_collision(
    mut collision_events: EventReader<CollisionEnded>,
    mut t_q: Query<(Entity, &mut LifeTime), With<Released>>,
) {
    for CollisionEnded(e1, e2) in collision_events.read() {
        t_q.iter_mut().for_each(|(e, mut lt)| {
            if e == *e1 || e == *e2 {
                lt.0.set_duration(Duration::ZERO); 
            }
        });
    }
}

// ---

fn despawn_on_time(
    mut t_q: Query<(Entity, &mut LifeTime)>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    for (e, mut lt) in &mut t_q {
        lt.0.tick(time.delta());
        if lt.0.finished() {
            cmd.entity(e).despawn();
        }
    }
}
