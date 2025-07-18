
use bevy::{
    prelude::*,
    scene::SceneInstanceReady,
    asset::embedded_asset
};
use avian3d::prelude::*;
use std::time::Duration;
mod ball;
use ball::{BallSpawn, Released, BALL_RADIUS};

#[derive(Resource)]
pub(crate) struct TrebuchetSettings {
    counterweight_density: f32,
    ball_layer_mask: u32,
    unhooking_dot: f32,
    ball_density: f32

}

pub struct TrebuchetPlugin {
    pub counterweight_density: f32,
    pub ball_layer_mask: u32, 
    pub unhooking_dot: f32,
    pub ball_density: f32
}

impl Default for TrebuchetPlugin {
    fn default() -> Self {
        Self { 
            counterweight_density: CW_DENSITY,
             ball_layer_mask: 666,
             unhooking_dot:0.99,
             ball_density:BALL_DENSITY
        }
    }
}

impl Plugin for TrebuchetPlugin {
    fn build(&self, app: &mut App) {


        embedded_asset!(app, "files/trebuchet.glb");
        app
        .insert_resource(TrebuchetSettings{
            counterweight_density:self.counterweight_density,
            ball_layer_mask: self.ball_layer_mask,
            unhooking_dot: self.unhooking_dot,
            ball_density: self.ball_density,
        })
        .add_plugins(ball::BallPlugin)
        .add_systems(Update, reload.run_if(any_with_component::<Interval>))
        .add_systems(Update, do_tension.run_if(any_with_component::<StateTension>))
        .add_systems(Update, do_loose.run_if(any_with_component::<StateLoose>))
        .add_observer(new)
        .add_observer(setup)
        .add_observer(enter_idle)
        .add_observer(enter_tension)
        .add_observer(enter_arming)
        .add_observer(do_arming)
        ;    
    }
}

// ---

#[derive(Resource)]
struct TrebuchetModel(Handle<Scene>);

#[derive(Event)]
pub struct NewTrebuchets(pub Vec<Transform>);

#[derive(Component)]
struct Arm;

#[derive(Component)]
struct Pivot;

#[derive(Component)]
struct CounterWeight;

#[derive(Component)]
struct Lock;

#[derive(Component)]
struct Bar;

#[derive(Component)]
struct Trebuchet;

#[derive(Component)]
struct  SlingEnd;

// ---

#[derive(Component)]
struct StateIdle;

#[derive(Component)]
struct StateTension;

#[derive(Component)]
struct StateArming;

#[derive(Component)]
struct StateLoose;

// ---

#[derive(Component)]
struct Link;

#[derive(Component)]
struct Parts {
    pivot: Entity,
    se: Entity,
    arm: Entity,
    bar: Entity, 
    cw: Entity,
    link: Option<Entity>
}

impl Parts {
    fn new() -> Self{
        Self {pivot: Entity::PLACEHOLDER,
            se: Entity::PLACEHOLDER,
            arm: Entity::PLACEHOLDER,
            bar: Entity::PLACEHOLDER, 
            cw: Entity::PLACEHOLDER,
            link: None
        }
    }
}

#[derive(Component)]
struct Interval(Timer);

// -- CONSTANTS --

const ARM_DIM: Vec3 =  Vec3::new(1., 1., 15.);
const CW_DENSITY: f32 =  9.5;
const PIVOT_DAMPING: f32 = 0.1; 
const PIVOT_OFFSET: f32 = ARM_DIM.z  * 0.3; 
const ARM_LONG_END_Y: f32 = 1.0;



const SLING_ELEMENT_DENSITY: f32 = 200.;  // 100
const SLING_ELEMENT_COUNT: u32 = 8;
const SLING_LEN: f32 = ARM_DIM.z * 0.75;
const UNHOOKING_DOT: f32 = 0.99;
const BALL_DENSITY: f32 = 14.5;

const TREBUCHET_DIM: Vec3 = Vec3::new(4., 8., 16.);  // ROUGLY

// ---

fn new (
    tr: Trigger<NewTrebuchets>,
    tho: Option<Res<TrebuchetModel>>,
    assets: ResMut<AssetServer>,
    mut cmd: Commands,
) {
    let th  = tho.map_or_else(|| {
        let h = assets.load(GltfAssetLabel::Scene(0).from_asset("embedded://bevy_trebuchet/files/trebuchet.glb"));
        cmd.insert_resource(TrebuchetModel(h.clone()));
        h
    }, |a| a.0.clone());
 
    tr.event().0.iter().for_each(| trans | {
        cmd.spawn((
            SceneRoot(th.clone()),
            *trans,
            Trebuchet,
            RigidBody::Static,
        ))
        .observe(explore)
        ;
    });
}

// ---

fn explore(
    tr: Trigger<SceneInstanceReady>,
    props: Query<&GltfExtras>,
    children: Query<&Children>,
    mut cmd: Commands,
) {
    let mut parts = Parts::new();
    for c in children.iter_descendants_depth_first(tr.target()) {
        let Ok(ex) = props.get(c) else {
            continue;
        }; 
        if ex.value.contains("Arm") {
            parts.arm = c;
            cmd.entity(c).insert(Arm);
        } else if ex.value.contains("Pivot") {
            parts.pivot = c;
            cmd.entity(c).insert(Pivot);
        } else if ex.value.contains("CounterWeight") {
            parts.cw = c;
            cmd.entity(c).insert(CounterWeight);
        } else if  ex.value.contains("Lock") {
            cmd.entity(c).insert(Lock);
        } else if  ex.value.contains("Bar") {
            parts.bar = c;
            cmd.entity(c).insert(Bar);
        } else if  ex.value.contains("Hill") {
            cmd.entity(c).insert(Collider::cuboid(2., 0.125, 8.));
        }
    }
    cmd.entity(tr.target()).insert(parts);
}

// ---

fn setup(
    tr: Trigger<OnAdd, Parts>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
    mut parts_q: Query<&mut Parts>,
    arm_q: Query<&Transform, With<Arm>>,
    mut cmd: Commands,
    settings: Res<TrebuchetSettings>
) {
    let treb_e = tr.target();
    let mut parts = parts_q.get_mut(treb_e).unwrap(); 
    cmd.entity(parts.arm)
    .insert((
        RigidBody::Dynamic,
        MassPropertiesBundle::from_shape(&Collider::cuboid(ARM_DIM.x, ARM_DIM.y, ARM_DIM.z), 1.),
    ));
    let anchor_arm = Vec3::Z *  ARM_DIM.z * 0.5;

    // PIVOT ===========================================================

    cmd.entity(parts.pivot).insert(RigidBody::Static);
    let joint_id = cmd.spawn((
        RevoluteJoint::new(parts.pivot, parts.arm)
        .with_aligned_axis(Vec3::X)
        .with_local_anchor_2(-Vec3::Z * PIVOT_OFFSET)
        .with_angular_velocity_damping(PIVOT_DAMPING)
        ,
    )).id();

    cmd.entity(treb_e).add_child(joint_id);

    // CW ==============================================================

    cmd.entity(parts.cw)
    .insert((
        RigidBody::Dynamic,
        // RigidBody::Static,
        MassPropertiesBundle::from_shape(&Collider::cylinder(4., 2.), settings.counterweight_density)
    ));
            
    let joint_id = cmd.spawn(
        RevoluteJoint::new(parts.arm, parts.cw)
        .with_aligned_axis(Vec3::X)
        .with_local_anchor_1(-anchor_arm)
        .with_local_anchor_2(Vec3::Y)
    ).id();

    cmd.entity(treb_e).add_child(joint_id);

    // SLING ============================================================

    let arm_pos = arm_q.get(parts.arm).unwrap().translation;

    let element_dim = Vec3::new(0.1, 0.1, SLING_LEN / SLING_ELEMENT_COUNT as f32);

    let anchor_element = Vec3::Z * element_dim.z / 2.;
    let element_mesh = meshes.add(Cuboid::from_size(element_dim));
    let element_mat = materials.add(Color::BLACK);

    let mut pos = arm_pos +  anchor_arm + anchor_element; 
    let mut prev_element_id = parts.arm;
        

    for i in 0 .. SLING_ELEMENT_COUNT {
        let element_id = cmd.spawn((
            Mesh3d(element_mesh.clone()),
            MeshMaterial3d(element_mat.clone()),
            Transform::from_translation(pos),
            RigidBody::Dynamic,
            MassPropertiesBundle::from_shape(
                &Collider::cuboid(element_dim.x, element_dim.y, element_dim.z), 
            SLING_ELEMENT_DENSITY),
        )).id();
        cmd.entity(treb_e).add_child(element_id);

        // let joint_id = cmd.spawn(
        //     SphericalJoint::new(prev_element_id, element_id)
        //     .with_local_anchor_1(if i == 0 {anchor_arm} else {anchor_element})
        //     .with_local_anchor_2(-anchor_element)
        // ).id();

        let joint_id = cmd.spawn(
            RevoluteJoint::new(prev_element_id, element_id)
            .with_local_anchor_1(if i == 0 {anchor_arm} else {anchor_element})
            .with_local_anchor_2(-anchor_element)
            .with_aligned_axis(Vec3::X)
        ).id();
        
        cmd.entity(treb_e).add_child(joint_id);

        prev_element_id  = element_id;
        pos += 2. * anchor_element;
    }

    // ENDING =======================================================================
        
    let ending_radius = 0.1;
    let ending_id = cmd.spawn((
        Mesh3d(meshes.add(Sphere::new(ending_radius))),
        MeshMaterial3d(element_mat.clone()),
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        Friction::new(0.).with_combine_rule(CoefficientCombine::Min),
        Collider::sphere(ending_radius * 3.),
        CollisionMargin(0.1),
        SlingEnd
    ))
    .id()
    ;
    cmd.entity(treb_e).add_child(ending_id);

    let joint_id = cmd.spawn(
        SphericalJoint::new(prev_element_id, ending_id)
        .with_local_anchor_1(anchor_element)
    ).id();

    cmd.entity(treb_e).add_child(joint_id);

    parts.se = ending_id;    

    // BAR ==========================================================================

    cmd.entity(parts.bar)
    .insert((
        RigidBody::Static,
        Collider::cuboid(4., 0.5, 16.)
    ));

    cmd.entity(treb_e).insert(StateIdle);
} 

// ---

fn enter_idle(
    trigger: Trigger<OnAdd, StateIdle>,
    mut cmd: Commands,
) {
    cmd.entity(trigger.target()).insert(
        Interval(Timer::new(Duration::from_secs(fastrand::u64(5..10)), TimerMode::Once))
    );
}

// ---

fn reload(
    mut t_q: Query<(Entity, &mut Interval), With<Trebuchet>>,
    mut cmd: Commands,
    time: Res<Time>
) {
    for (e, mut interval) in & mut t_q {
        interval.0.tick(time.delta());
        if interval.0.finished() {
            cmd.entity(e)
            .remove::<StateIdle>()
            .remove::<Interval>()
            .insert(StateTension)
            ;
        }
    }
}

// ---

fn enter_tension(
    trigger: Trigger<OnAdd, StateTension>,
    mut parts_q: Query<&mut Parts>,
    mut cmd: Commands,
) {
    let treb_e = trigger.target();
    let Ok(mut parts) = parts_q.get_mut(treb_e) else {
        return;
    };
    
    let joint_id = cmd.spawn((
        DistanceJoint::new(parts.se, parts.bar)
        .with_limits(0.1, 20.)
        .with_local_anchor_2(Vec3::Z * TREBUCHET_DIM.z * 0.5 + Vec3::Y)
        ,
        Link
    )).id();
    parts.link = Some(joint_id);
}

// ---

fn do_tension(
    treb_q: Query<(Entity, &Parts), (With<Trebuchet>, With<StateTension>)>,
    arm_q: Query<&Transform, With<Arm>>,
    mut link_q: Query<&mut DistanceJoint, With<Link>>,
    mut cmd: Commands,
) {
    for (treb_e, treb_parts)  in treb_q.iter() {
        let Ok(arm_t) = arm_q.get(treb_parts.arm) else {
            continue;
        };
        
        let arm_long_end_y = (arm_t.translation - arm_t.forward() * ARM_DIM.z * 0.5).y;
        if arm_long_end_y  < ARM_LONG_END_Y {
            cmd.entity(treb_e)
            .remove::<StateTension>()
            .insert(StateArming)
            ;
            continue;
        }

        let Some(link_e) = treb_parts.link else {
            continue;
        };
        let Ok(mut joint)  =  link_q.get_mut(link_e) else {
            continue;
        }; 
        let Some(limits) = joint.length_limits else {
            continue;
        };
        let j = DistanceJoint::new(joint.entity1, joint.entity2)
                .with_local_anchor_1(joint.local_anchor1);

        if limits.max > 1. {
            *joint = j
            .with_local_anchor_2(joint.local_anchor2)
            .with_limits(joint.rest_length + 1., limits.max - 0.05)
            ;
        } else {
            *joint = j
            .with_local_anchor_2(joint.local_anchor2 - Vec3::Z * 0.05)
            .with_limits(limits.min, limits.max)
            ;
        }
    }
}

// ---

fn enter_arming(
    trigger: Trigger<OnAdd, StateArming>,
    mut cmd: Commands,
    treb_q: Query<&Transform>,
) {
    let treb_e = trigger.target();

    let Ok(t) = treb_q.get(treb_e) else {
        return;
    };
    cmd.trigger(BallSpawn(t.translation + t.forward() * TREBUCHET_DIM.z  + t.up() * TREBUCHET_DIM.y * 0.5));
}

// ---

fn do_arming(
    tr: Trigger<OnCollisionEnd>,
    se_q: Query<Entity, With<SlingEnd>>,
    parent_q: Query<&ChildOf>,
    parts_q: Query<&Parts>,
    mut cmd: Commands
) {
    let Some(se_e) = tr.event().body else {
        return;
    }; 
    if !se_q.contains(se_e) {
        return;
    }
    let ball_e = tr.target();

    for p in parent_q.iter_ancestors(se_e) {
        if let Ok(parts) =  parts_q.get(p) {
            let Some(link_e) = parts.link else {
                continue;
            };
            cmd.entity(link_e)
            .insert(
                DistanceJoint::new(se_e, ball_e)
                .with_rest_length(BALL_RADIUS * 2.)
                .with_linear_velocity_damping(1000.)
            );
            cmd.entity(p)
            .remove::<StateArming>()
            .insert(StateLoose);
            cmd.entity(ball_e)
            .insert(LinearVelocity(Vec3::ZERO))
            ;
            break;
        }
    }
}

// ---

fn do_loose(
    mut treb_q: Query<(Entity, &mut Parts, &Transform), (With<Trebuchet>, With<StateLoose>)>,
    mut cmd: Commands,
    se_q: Query<&GlobalTransform>,
    link_q: Query<&DistanceJoint>,
    settings: Res<TrebuchetSettings>
) {

    for (treb_e, mut treb_parts, treb_t)  in treb_q.iter_mut() {
        
        let Ok(se_t) = se_q.get(treb_parts.se) else {
            continue;
        };
        let Some(link) = treb_parts.link else {
            continue;
        };

        let Ok(link_j) = link_q.get(link) else {
            continue;
        };

        let center = treb_t.translation  + Vec3::Y * TREBUCHET_DIM.y * 0.5;
        let to_se = (se_t.translation() - center).normalize();
        let dot = to_se.dot(Vec3::Y);
        if dot > settings.unhooking_dot {
            cmd.entity(link_j.entity2).insert(Released);
            cmd.entity(link).despawn();
            cmd.entity(treb_e)
            .remove::<StateLoose>()
            .insert(StateIdle)
            ;
            treb_parts.link = None;
        } 
    }
}
