use std::sync::atomic::AtomicBool;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    render::view::NoFrustumCulling,
    window::{PrimaryWindow, WindowMode, WindowResolution},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;


pub mod bloom;
pub mod bullet_tracer;
pub mod enemy;
pub mod fps_camera;
pub mod fps_movement;
pub mod fps_shooting;
pub mod gun_control;
pub mod lock_cursor;
pub mod rotation_operations;
pub mod score_ui;
pub mod vector_operations;
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.8, 0.9)))
        .insert_resource(lock_cursor::CursorLockState {
            state: false,
            allow_lock: true,
        })
        
        .add_system(fps_movement::player_movement)
        .add_system(fps_camera::move_camera.after(fps_movement::player_movement))
        .add_system(gun_control::update_gun_control.after(fps_camera::move_camera))
        .add_system(bloom::update_bloom_settings)
        .add_system(fps_shooting::update_shots.after(fps_shooting::update_bullet_params))
        .add_system(fps_shooting::update_bullet_params.before(fps_shooting::update_shots))
        .add_system(fps_shooting::play_gun_animations.after(fps_shooting::update_shots))
        .add_system(fps_shooting::update_targets)
        .add_system(lock_cursor::lock_cursor_position)
        .add_system(bullet_tracer::update_tracers)
        .add_system(score_ui::update_score)
        .add_system(link_animations)
        .add_system(gun_control::update_ammo_count_text)
        .add_system(gun_control::apply_movement_inaccuracy.before(fps_shooting::update_shots))
        .add_system(enemy::rotate_to_player.in_base_set(CoreSet::PostUpdate))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: WindowResolution::new(1920., 1080.),
                        mode: WindowMode::BorderlessFullscreen,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())

        .add_system(check_assets_ready)
        .init_resource::<AssetsLoading>()

        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(setup_ui)
        
        .add_startup_system(setup_physics)
        .run();
}

#[derive(Resource,Default)]
pub struct AssetsLoading(Vec<HandleUntyped>);

fn check_assets_ready(
    commands: Commands,
    server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    loading: Res<AssetsLoading>,
) {
    use bevy::asset::LoadState;
    static SETUP_PHYSICS_CALLED: AtomicBool = AtomicBool::new(false);
    match server.get_group_load_state(loading.0.iter().map(|h| h.id())) {
        LoadState::Failed => {
            // one of our assets had an error
        }
        LoadState::Loaded => {
            if !SETUP_PHYSICS_CALLED.load(std::sync::atomic::Ordering::Relaxed) {
                setup_map(commands, server, meshes, materials);
                SETUP_PHYSICS_CALLED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}
#[derive(Component)]
pub struct AnimationEntityLink(pub Entity);

fn get_top_parent(mut curr_entity: Entity, parent_query: &Query<&Parent>) -> Entity {
    //Loop up all the way to the top parent
    loop {
        if let Ok(parent) = parent_query.get(curr_entity) {
            curr_entity = parent.get();
        } else {
            break;
        }
    }
    curr_entity
}

pub fn link_animations(
    player_query: Query<Entity, Added<AnimationPlayer>>,
    parent_query: Query<&Parent>,
    animations_entity_link_query: Query<&AnimationEntityLink>,
    mut commands: Commands,
) {
    // Get all the Animation players which can be deep and hidden in the heirachy
    for entity in player_query.iter() {
        let top_entity = get_top_parent(entity, &parent_query);

        // If the top parent has an animation config ref then link the player to the config
        if animations_entity_link_query.get(top_entity).is_ok() {
            warn!("Problem with multiple animationsplayers for the same top parent");
        } else {
            commands
                .entity(top_entity)
                .insert(AnimationEntityLink(entity.clone()));
        }
    }
}
fn setup_map(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,)
{
    let x_shape: Handle<Mesh> = asset_server.load("map.glb#Mesh0/Primitive0");

    let m = &meshes.get(&x_shape);
    
    let x_shape = Collider::from_bevy_mesh(m.unwrap(), &ComputedColliderShape::TriMesh).unwrap();
    if Collider::from_bevy_mesh(m.unwrap(), &ComputedColliderShape::TriMesh).is_none()
    {
        println!("{}","the mesh failed to load");
    }
    //println!("{}",x_shape);
    commands.spawn((
        SceneBundle {
            transform: Transform::from_xyz(0., 0., 0.),
            scene: asset_server.load("map.glb#Scene0"),
            ..default()
        },

    )).insert(x_shape);
}
fn setup_physics(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<AssetsLoading>
) {
    let x_shape: Handle<Mesh> = asset_server.load("map.glb#Mesh0/Primitive0");
    loading.0.push(x_shape.clone_untyped());
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    let texture_handle = asset_server.load("sand.png");

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture_handle.clone()),
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            transform: Transform::from_xyz(0.0, -ground_height, 0.0).with_scale(Vec3::new(
                ground_size,
                ground_height,
                ground_size,
            )),
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: wall_mat.clone(),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(ground_size, ground_height, ground_size),
    ));

    commands.spawn((
        PbrBundle {
            transform: Transform::from_xyz(0.0, 0., -5.0).with_scale(Vec3::new(30., 15., 1.)),
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: wall_mat.clone(),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));

    
    /*
    commands
    .spawn(SpatialBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    },)
    //.spawn(TransformBundle::from(Transform::from_rotation(Quat::from_rotation_x(0.2),)))
    .with_children(|child| {
        child.spawn((
            PbrBundle {
                transform: Transform::from_xyz(0., 5., 0.),
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1. })),
                material: materials.add(Color::rgb(1., 1., 1.).into()),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
        ));
    });
    */
}
pub fn setup_ui(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    primary_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(primary) = primary_query.get_single() else
    {
        return;
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage {
                    texture: asset_server.load("crosshair.png"),
                    ..default()
                },
                style: Style {
                    size: Size::new(Val::Px(9.), Val::Px(9.)),
                    position: UiRect::new(
                        Val::Px(primary.width() / 2. - 4.5),
                        Val::Px(primary.width() / 2. + 4.5),
                        Val::Px(primary.height() / 2. - 4.5),
                        Val::Px(primary.height() / 2. + 4.5),
                    ),
                    ..default()
                },
                ..default()
            });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position: UiRect::new(
                            Val::Percent(0.),
                            Val::Percent(100.),
                            Val::Px(primary.height() - 30.),
                            Val::Px(primary.height() + 0.),
                        ),
                        ..default()
                    },
                    background_color: Color::rgba(0.15, 0.15, 0.15, 0.).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        gun_control::AmmoText {},
                        TextBundle::from_section(
                            "25/100",
                            TextStyle {
                                font: asset_server.load("font.ttf"),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        }),
                        Label,
                    ));
                });
        });
}
#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Resource)]
pub struct EnemyAnimations(Vec<Handle<AnimationClip>>);
pub fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Animations(vec![
        asset_server.load("gun.glb#Animation0"),
        asset_server.load("gun.glb#Animation1"),
        asset_server.load("gun.glb#Animation2"),
    ]));

    commands.insert_resource(EnemyAnimations(vec![
        asset_server.load("person.glb#Animation0"),
        asset_server.load("person.glb#Animation1")
    ]));
    WindowResolution::new(1980., 1080.);
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..default()
    });

    commands
        .spawn(SpatialBundle {
            visibility: Visibility::Visible,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        //.spawn(TransformBundle::from(Transform::from_rotation(Quat::from_rotation_x(0.2),)))
        .with_children(|child| {
            child.spawn((
                Visibility::Visible,
                Camera3dBundle {
                    camera: Camera {
                        hdr: true, // 1. HDR is required for bloom
                        ..default()
                    },
                    tonemapping: Tonemapping::TonyMcMapface,
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: (103.0 / 360.0) * (std::f32::consts::PI * 2.0),
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.0, 4.0),

                    ..default()
                },
                BloomSettings {
                    intensity: 0.2,
                    ..default()
                },
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED_X
                    | LockedAxes::ROTATION_LOCKED_Y
                    | LockedAxes::ROTATION_LOCKED_Z,
                Velocity {
                    linvel: Vec3::new(0.0, 0.0, 0.0),
                    angvel: Vec3::new(0.0, 0.0, 0.0),
                },
                Collider::cuboid(0.2, 1.4, 0.2),
                fps_camera::FPSCamera {
                    camera_shake_readjustment_factor: 0.3,
                    recoil_shake: Vec3::ZERO,
                    rotation: Vec3::new(0., 0., 0.),
                    speed: 300.,
                    rotate_lock: 88. * 0.0174533,
                    sensitivity: (0.173) / 900.,
                },
                Damping {
                    linear_damping: 4.,
                    angular_damping: 1.0,
                },
                GravityScale(1.),
                fps_movement::FPSMovement {
                    speed: 2.2,
                    acceleration: 400.,
                },
            ));
        });
    let mut spray_pattern_primary = Vec::new();
    spray_pattern_primary.push(Vec2::new(0., 0.));
    spray_pattern_primary.push(Vec2::new(0., 0.007));
    spray_pattern_primary.push(Vec2::new(0., 0.011));
    spray_pattern_primary.push(Vec2::new(0.008, 0.019));
    spray_pattern_primary.push(Vec2::new(-0.001, 0.032));
    spray_pattern_primary.push(Vec2::new(-0.007, 0.042));
    spray_pattern_primary.push(Vec2::new(-0.003, 0.07));
    spray_pattern_primary.push(Vec2::new(0.0008, 0.09));
    spray_pattern_primary.push(Vec2::new(0.01, 0.12));
    spray_pattern_primary.push(Vec2::new(0.0068, 0.144));
    spray_pattern_primary.push(Vec2::new(0.002, 0.158));
    spray_pattern_primary.push(Vec2::new(0.01, 0.161));
    spray_pattern_primary.push(Vec2::new(0.001, 0.179));
    //random ish
    spray_pattern_primary.push(Vec2::new(0.008, 0.2));
    spray_pattern_primary.push(Vec2::new(0.018, 0.21));
    spray_pattern_primary.push(Vec2::new(0.038, 0.19));
    spray_pattern_primary.push(Vec2::new(0.04, 0.17));
    spray_pattern_primary.push(Vec2::new(0.082, 0.2));
    spray_pattern_primary.push(Vec2::new(0.11, 0.22));
    spray_pattern_primary.push(Vec2::new(0.06, 0.2));
    spray_pattern_primary.push(Vec2::new(0.04, 0.21));
    spray_pattern_primary.push(Vec2::new(0., 0.18));
    spray_pattern_primary.push(Vec2::new(-0.01, 0.206));
    spray_pattern_primary.push(Vec2::new(-0.033, 0.19));
    spray_pattern_primary.push(Vec2::new(-0.022, 0.2));
    commands.spawn((
        SceneBundle {
            transform: Transform::from_xyz(0., 0., 0.),
            scene: asset_server.load("gun.glb#Scene0"),
            ..default()
        },
        gun_control::GunController {
            movement_inaccuracy: 0.,
            reloading_time: 1.0,
            reloading_timer: 0.,
            spray_rand: 0.01,
            aiming_down_sights: false,
            recoil_shake: Vec3::ZERO,
            current_camera_transform: Transform::from_xyz(0.0, 0.0, 4.0),
            smooth_scale: 0.6,
            magazine_size: 25,
            bullets: 25,
            spray_index: 0,
            recoil_reset_time: 0.32,
            time_since_last_shot: 0.,
            cooldown: 0.1,
            timer: 0.,
            dynamic_offset: Vec3::ZERO,
            target_offset: Vec3::ZERO,
            spray_pattern: spray_pattern_primary,
            shoot: false,
            gun_scale: 0.26,
            offset: Vec3::new(0., 0., 0.),
        },
    ));
    let mut person_transform = Transform::from_xyz(0., 0., 0.);
    person_transform.scale = Vec3::new(2.5, 2.5, 2.5);
    commands.spawn((
        SceneBundle {
            transform: person_transform,
            scene: asset_server.load("person.glb#Scene0"),
            ..default()
        },
        enemy::Enemy {
            health:100.,
            shoot_timer: 3.,
            shoot_cooldown: 3.,
            added_colliders: false,
        },
        NoFrustumCulling,
    ));
    person_transform.translation +=Vec3::new(2.,0.,0.);
    commands.spawn((
        SceneBundle {
            transform: person_transform,
            scene: asset_server.load("person.glb#Scene0"),
            ..default()
        },
        enemy::Enemy {
            health:100.,
            shoot_timer: 3.,
            shoot_cooldown: 3.,
            added_colliders: false,
        },
        NoFrustumCulling,
    ));
    person_transform.translation +=Vec3::new(2.,0.,0.);

    commands.spawn((
        SceneBundle {
            transform: person_transform,
            scene: asset_server.load("person.glb#Scene0"),
            ..default()
        },
        enemy::Enemy {
            health:100.,
            shoot_timer: 3.,
            shoot_cooldown: 3.,
            added_colliders: false,
        },
        NoFrustumCulling,
    ));
    let _rng = rand::thread_rng();
    /*
    let mut pos_vec = Vec::new();
    for i in 0..5 {
        pos_vec.push(fps_shooting::generate_target_position(&mut rng));
        if i != 0 {
            let mut unique = false;
            while unique == false {
                for j in 0..pos_vec.len() - 1 {
                    unique = true;
                    if pos_vec[j].x as i32 == pos_vec[i].x as i32
                        && pos_vec[j].y as i32 == pos_vec[i].y as i32
                        && pos_vec[j].z as i32 == pos_vec[i].z as i32
                        && i != j
                    {
                        pos_vec[i] = fps_shooting::generate_target_position(&mut rng);
                        unique = false;
                        break;
                    }
                }
            }
        }

        commands
            .spawn(SpatialBundle {
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            })
            //.spawn(TransformBundle::from(Transform::from_rotation(Quat::from_rotation_x(0.2),)))
            .with_children(|child| {
                child.spawn((
                    PbrBundle {
                        transform: Transform::from_xyz(pos_vec[i].x, pos_vec[i].y, pos_vec[i].z),
                        mesh: meshes.add(Mesh::from(shape::UVSphere {
                            radius: 0.2,
                            ..default()
                        })),

                        material: materials.add(Color::rgb(1., 0., 0.).into()),
                        ..default()
                    },
                    ColliderDebugColor(Color::BLUE),
                    RigidBody::KinematicPositionBased,
                    Collider::cuboid(0.2, 0.2, 0.2),
                    fps_shooting::ShootableTarget {
                        health: 1.,
                        max_health: 1.,
                    },
                ));
            });
    }
    
     */
    
}
