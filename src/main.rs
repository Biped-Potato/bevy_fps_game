use bevy::{input::common_conditions::input_toggle_active, prelude::*, window::{WindowResolution, WindowMode, PrimaryWindow}, reflect::erased_serde::__private::serde::__private::de, core_pipeline::{tonemapping::Tonemapping, bloom::BloomSettings}, core::Zeroable};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use fps_camera::FPSCamera;
use gun_control::GunController;
use rand::Rng;

pub mod score_ui;
pub mod fps_camera;
pub mod fps_movement;
pub mod fps_shooting;
pub mod lock_cursor;
pub mod bullet_tracer;
pub mod vector_operations;
pub mod rotation_operations;
pub mod gun_control;
pub mod bloom;
fn main() {
    App::new()
        
        .insert_resource(ClearColor(Color::rgb(0.5, 0.8, 0.9)))
        
        .insert_resource(lock_cursor::CursorLockState { state: false,allow_lock:true })

        .add_system(fps_movement::player_movement)
        .add_system(fps_camera::move_camera.after(fps_movement::player_movement))

        .add_system(gun_control::update_gun_control.after(fps_camera::move_camera))
        .add_system(bloom::update_bloom_settings)
        .add_system(fps_shooting::update_shots)
        .add_system(lock_cursor::lock_cursor_position)
        .add_system(fps_shooting::update_targets)
        .add_system(bullet_tracer::update_tracers)
        .add_system(score_ui::update_score)
        .add_system(link_animations)
        /*
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
              position : WindowPosition::Centered((MonitorSelection::Primary)),
              resolution: WindowResolution::new(1980.,1080.),
              mode:WindowMode::BorderlessFullscreen,
              ..default()
            }),
            ..default()
          }))
        */
        
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
        primary_window: Some(Window {
            position : WindowPosition::Centered((MonitorSelection::Primary)),
            resolution: WindowResolution::new(1280.,720.),
            mode:WindowMode::Windowed,
            ..default()
        }),
        ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())

        
        .add_plugin(WorldInspectorPlugin::default(), )
        

        .add_startup_system(setup)
        .add_startup_system(setup_ui)
        .add_startup_system(setup_physics)
        .run();
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
fn setup_physics(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    let texture_handle = asset_server.load("sand.png");

    let wall_mat = materials.add(StandardMaterial {
        base_color : Color::WHITE,
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
            transform: Transform::from_xyz(0.0, 0., -5.0).with_scale(Vec3::new(
                30.,
                15.,
                1.,
            )),
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
pub fn setup_ui( asset_server: Res<AssetServer>,mut commands : Commands,primary_query:Query<&Window,With<PrimaryWindow>>)
{
    let Ok(primary) = primary_query.get_single() else
    {
        return;
    };
    
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0),Val::Percent(100.0)),
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        
        .with_children(|parent| {
            parent
            .spawn(ImageBundle {
                image: UiImage { texture: asset_server.load("crosshair.png"),..default()},
                style: Style {
                    size: Size::new(Val::Px(9.),Val::Px(9.)),
                    position:UiRect::new(
                    Val::Px(primary.width()/2. - 4.5),
                    Val::Px(primary.width()/2. + 4.5),
                    Val::Px(primary.height()/2. -4.5),
                    Val::Px(primary.height()/2. + 4.5)),
                    ..default()
                },
                ..default()
            });
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(50.0)),
                    ..default()
                },
                background_color: Color::rgba(0.15, 0.15, 0.15,0.).into(),
                ..default()
            })
            .with_children(|parent| {
                // text
                parent.spawn((
                    score_ui::ScoreText{score:0},
                    TextBundle::from_section(
                        "Score:",
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
                    // Because this is a distinct label widget and
                    // not button/list item text, this is necessary
                    // for accessibility to treat the text accordingly.
                    Label,
                ));
            });
        });

        
}
#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);
pub fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) 
{
    commands.insert_resource(Animations(vec![
        asset_server.load("gun.glb#Shoot"),
        asset_server.load("gun.glb#Idle"),
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
    

    commands.spawn(SpatialBundle {
        visibility:Visibility::Visible,
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    },)
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
                projection: Projection::Perspective(PerspectiveProjection { fov: (103.0 / 360.0) * (std::f32::consts::PI * 2.0),..Default::default()}),
                transform: Transform::from_xyz(0.0, 0.0, 4.0),
                
                ..default()
            },
            BloomSettings
            {
                ..default()
            },
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED_Z,
            Velocity {
                linvel: Vec3::new(0.0, 0.0, 0.0),
                angvel: Vec3::new(0.0, 0.0, 0.0),
            },
            Collider::cuboid(0.2,1.4,0.2),
            fps_camera::FPSCamera {
                camera_shake_readjustment_factor : 0.3,
                recoil_shake : Vec3::ZERO,
                rotation: Vec3::new(0., 0., 0.),
                speed: 300.,
                rotate_lock: 88. * 0.0174533,
                sensitivity: (0.173)/900.,
            },
            Damping { linear_damping: 4., angular_damping: 1.0 },
            GravityScale(1.),
            fps_movement::FPSMovement
            {
                speed:1.8,
                acceleration : 400.
            }
        ));
    });
    let mut spray_pattern_primary = Vec::new();
    spray_pattern_primary.push(Vec2::new(0.,0.));
    spray_pattern_primary.push(Vec2::new(0.,0.007));
    spray_pattern_primary.push(Vec2::new(0.,0.011));
    spray_pattern_primary.push(Vec2::new(0.008,0.019));
    spray_pattern_primary.push(Vec2::new(-0.001,0.032));
    spray_pattern_primary.push(Vec2::new(-0.007,0.042));
    spray_pattern_primary.push(Vec2::new(-0.003,0.07));
    spray_pattern_primary.push(Vec2::new(0.0008,0.09));
    spray_pattern_primary.push(Vec2::new(0.01,0.12));
    spray_pattern_primary.push(Vec2::new(0.0068,0.144));
    spray_pattern_primary.push(Vec2::new(0.002,0.158));
    spray_pattern_primary.push(Vec2::new(0.01,0.161));
    spray_pattern_primary.push(Vec2::new(0.001,0.179));
    //random ish
    spray_pattern_primary.push(Vec2::new(0.008,0.2));
    spray_pattern_primary.push(Vec2::new(0.018,0.21));
    spray_pattern_primary.push(Vec2::new(0.038,0.19));
    spray_pattern_primary.push(Vec2::new(0.04,0.17));
    spray_pattern_primary.push(Vec2::new(0.082,0.2));
    spray_pattern_primary.push(Vec2::new(0.11,0.22));
    spray_pattern_primary.push(Vec2::new(0.06,0.2));
    spray_pattern_primary.push(Vec2::new(0.04,0.21));
    spray_pattern_primary.push(Vec2::new(0.,0.18));
    spray_pattern_primary.push(Vec2::new(-0.01,0.206));
    spray_pattern_primary.push(Vec2::new(-0.033,0.19));
    spray_pattern_primary.push(Vec2::new(-0.022,0.2));
    commands.spawn((
        SceneBundle 
        {    
            transform : Transform::from_xyz(0., 0., 0.),
            scene: asset_server.load("gun.glb#Scene0"),
            ..default()
        },
        gun_control::GunController{
            spray_rand : 0.01,
            aiming_down_sights : false,
            recoil_shake : Vec3::ZERO,
            current_camera_transform : Transform::from_xyz(0.0, 0.0, 4.0),
            smooth_scale: 0.6,
            magazine_size : 25,
            spray_index : 0,
            recoil_reset_time : 0.32,
            time_since_last_shot : 0.,
            cooldown : 0.1,
            timer : 0.,
            dynamic_offset : Vec3::ZERO,
            target_offset: Vec3::ZERO,
            spray_pattern : spray_pattern_primary,
            shoot : false,
            gun_scale : 0.26,
            offset:Vec3::new(0.,0.,0.)
        },
    ));
    let mut rng = rand::thread_rng();

    let mut pos_vec = Vec::new();
    for i in 0..5
    {
        pos_vec.push(fps_shooting::generate_target_position(&mut rng));
        if i != 0
        {
            let mut unique = false;
            while unique == false
            {
                for j in 0..pos_vec.len()-1
                {
                    unique = true;
                    if pos_vec[j].x as i32 == pos_vec[i].x as i32 && pos_vec[j].y as i32 == pos_vec[i].y as i32 && pos_vec[j].z as i32 == pos_vec[i].z as i32 && i!=j
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
        },)
        //.spawn(TransformBundle::from(Transform::from_rotation(Quat::from_rotation_x(0.2),)))
        .with_children(|child| {
            child.spawn((
                PbrBundle {
                    transform: Transform::from_xyz(pos_vec[i].x,pos_vec[i].y,pos_vec[i].z),
                    mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.2,..default() })),
                    
                    material: materials.add(Color::rgb(1., 0., 0.).into()),
                    ..default()
                },
                ColliderDebugColor(Color::BLUE),
                RigidBody::KinematicPositionBased,
                Collider::cuboid(0.2, 0.2, 0.2),
                fps_shooting::ShootableTarget{health:1.,max_health:1.},
            ));
        });
    }
    
}