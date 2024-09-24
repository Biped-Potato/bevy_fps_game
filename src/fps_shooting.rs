use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::bullet_tracer::BulletTracer;
use crate::enemy::{HeadCollider, BodyCollider, LegCollider, Enemy};
use crate::fps_camera::FPSCamera;
use crate::gun_control::{translate_gun_position, GunController};
use crate::rotation_operations::quaternion_look_rotation;
use crate::score_ui::ScoreText;
use crate::vector_operations::move_towards;
use crate::{AnimationEntityLink, Animations};

#[derive(Component)]
pub struct ShootableTarget {
    pub health: f32,
    pub max_health: f32,
}
pub fn play_gun_animations(
    _buttons: Res<Input<MouseButton>>,
    mut player_query: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    mut gun_query: Query<
        (&mut GunController, &mut Transform, &AnimationEntityLink),
        (Without<FPSCamera>, Without<ShootableTarget>),
    >,
) {
    for (gun_controller, _gun_transform, animation_entity) in gun_query.iter_mut() {
        if let Ok(mut player) = player_query.get_mut(animation_entity.0) {
            if gun_controller.reloading_timer >= 0. {
                player.play(animations.0[1].clone_weak());
            } else {
                if gun_controller.time_since_last_shot >= 0.2 {
                    player.play(animations.0[0].clone_weak()).repeat();
                }
                if gun_controller.shoot == true {
                    player.play(animations.0[0].clone_weak());
                    player.play(animations.0[2].clone_weak());
                }
            }
        }
    }
}
pub fn update_bullet_params(
    mut gun_query: Query<
        (&mut GunController, &mut Transform, &AnimationEntityLink),
        (Without<FPSCamera>, Without<ShootableTarget>),
    >,
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
) {
    for (mut gun_controller, _gun_transform, _animation_entity) in gun_query.iter_mut() {
        if gun_controller.time_since_last_shot >= gun_controller.recoil_reset_time {
            gun_controller.spray_index = 0;
        }
        gun_controller.timer -= time.delta_seconds();
        gun_controller.time_since_last_shot += time.delta_seconds();
        gun_controller.reloading_timer -= time.delta_seconds();
        if gun_controller.reloading_timer < 0. {
            gun_controller.shoot = false;
            if buttons.pressed(MouseButton::Left) {
                
                if gun_controller.timer <= 0. {
                    gun_controller.shoot = true;
                    gun_controller.bullets -= 1;
                    gun_controller.spray_index += 1;
                    if gun_controller.bullets <= 0 {
                        gun_controller.bullets = gun_controller.magazine_size;
                        gun_controller.reloading_timer = gun_controller.reloading_time;
                        gun_controller.spray_index = 0;
                    }
                    gun_controller.timer = gun_controller.cooldown;
                    
                }
            }
        }
    }
}
pub fn update_shots(
    mut gun_query: Query<
        (&mut GunController, &mut Transform, &AnimationEntityLink),
        (Without<FPSCamera>, Without<ShootableTarget>),
    >,
    _score_query: Query<&mut ScoreText, With<Text>>,
    mut camera_query: Query<(
        &Camera,
        &GlobalTransform,
        &mut Transform,
        Entity,
        &mut FPSCamera,
    )>,
    mut enemy_query : Query<&mut Enemy>,
    mut head_query: Query<&mut HeadCollider>,
    mut body_query: Query<&mut BodyCollider>,
    mut leg_query: Query<&mut LegCollider>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
) {
    for (mut gun_controller, mut gun_transform, _animation_entity) in gun_query.iter_mut() {
        if gun_controller.reloading_timer < 0. {
            if gun_controller.shoot == true {

                let window = windows.single();

                for (
                    camera,
                    camera_transform,
                    camera_transform_non_global,
                    entity,
                    mut fps_camera,
                ) in camera_query.iter_mut()
                {
                    gun_controller.time_since_last_shot = 0.;
                    let Some(ray) = camera.viewport_to_world(camera_transform, Vec2::new(window.width()/2.,window.height()/2.)) else { return; };

                    let x_quat =
                        Quat::from_axis_angle(Vec3::new(0., 1., 0.), fps_camera.rotation.y);

                    let y_quat =
                        Quat::from_axis_angle(Vec3::new(1., 0., 0.), fps_camera.rotation.x);

                    let mut camera_transform_non_corrupted = Transform::from_xyz(
                        camera_transform_non_global.translation.x,
                        camera_transform_non_global.translation.y,
                        camera_transform_non_global.translation.z,
                    );

                    camera_transform_non_corrupted.rotation = x_quat * y_quat;

                    let mut rng = rand::thread_rng();

                    let ray_direction;
                    let spray_rand_movement_added =
                        gun_controller.spray_rand + gun_controller.movement_inaccuracy;
                    if gun_controller.spray_index > 4 {
                        ray_direction = (camera_transform_non_corrupted.forward()
                            + (camera_transform_non_corrupted.up()
                                * (rng.gen_range(
                                    -spray_rand_movement_added..spray_rand_movement_added,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .y))
                            + (camera_transform_non_corrupted.right()
                                * (rng.gen_range(
                                    -spray_rand_movement_added..spray_rand_movement_added,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .x)))
                            .normalize();
                    }
                    else if gun_controller.spray_index == 1
                    {
                        ray_direction = (camera_transform_non_corrupted.forward()
                            + (camera_transform_non_corrupted.up()
                                * (rng.gen_range(
                                    -gun_controller.spray_rand / 200.0
                                        ..gun_controller.spray_rand /200.0,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .y))
                            + (camera_transform_non_corrupted.right()
                                * (rng.gen_range(
                                    -spray_rand_movement_added / 24.0
                                        ..spray_rand_movement_added / 24.0,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .x)))
                            .normalize();
                    } 
                    else {
                        ray_direction = (camera_transform_non_corrupted.forward()
                            + (camera_transform_non_corrupted.up()
                                * (rng.gen_range(
                                    -gun_controller.spray_rand / 3.0
                                        ..gun_controller.spray_rand / 3.0,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .y))
                            + (camera_transform_non_corrupted.right()
                                * (rng.gen_range(
                                    -spray_rand_movement_added / 3.0
                                        ..spray_rand_movement_added / 3.0,
                                ) + gun_controller.spray_pattern[gun_controller.spray_index-1]
                                    .x)))
                            .normalize();
                    }

                    let hit = rapier_context.cast_ray_and_get_normal(
                        ray.origin,
                        ray_direction,
                        //ray.direction,
                        f32::MAX,
                        true,
                        QueryFilter::new().exclude_collider(entity),
                    );
                    fps_camera.recoil_shake =
                        (ray_direction - camera_transform_non_corrupted.forward()) * 0.7;
                    gun_controller.recoil_shake =
                        (ray_direction - camera_transform_non_corrupted.forward()) * 1.;

                    let mut placebo_camera = Transform::from_xyz(
                        camera_transform_non_global.translation.x,
                        camera_transform_non_global.translation.y,
                        camera_transform_non_global.translation.z,
                    );

                    gun_controller.recoil_shake = move_towards(
                        gun_controller.recoil_shake,
                        Vec3::ZERO,
                        time.delta_seconds() * gun_controller.smooth_scale,
                    );

                    let x_quat = Quat::from_axis_angle(
                        Vec3::new(0., 1., 0.),
                        fps_camera.rotation.y - gun_controller.recoil_shake.x,
                    );

                    let y_quat = Quat::from_axis_angle(
                        Vec3::new(1., 0., 0.),
                        fps_camera.rotation.x + gun_controller.recoil_shake.y,
                    );

                    placebo_camera.rotation = x_quat * y_quat;

                    gun_transform.translation = translate_gun_position(&placebo_camera);

                    gun_transform.look_at(
                        camera_transform_non_global.translation + placebo_camera.forward() * 100.,
                        Vec3::Y,
                    );
                    gun_transform.scale = Vec3::new(
                        gun_controller.gun_scale,
                        gun_controller.gun_scale,
                        gun_controller.gun_scale,
                    );

                    if let Some((entity, ray_intersection)) = hit {
                        let bullet_tracer_material = materials.add(StandardMaterial {
                            emissive: Color::rgb_linear(100., 100., 50.0), // 4. Put something bright in a dark environment to see the effect
                            ..default()
                        });
                        commands.spawn((
                            PbrBundle {
                                transform: Transform::from_xyz(0., 100000., 0.),
                                mesh: meshes.add(Mesh::from(shape::Cube { size: 1. })),
                                material: bullet_tracer_material,
                                ..default()
                            },
                            NotShadowCaster {},
                            BulletTracer {
                                direction: Vec3::new(0., 0., 0.),
                                start_position: gun_transform.translation
                                    + (gun_transform.up() * 0.56704 * gun_controller.gun_scale)
                                    + (gun_transform.forward()
                                        * 3.13735
                                        * gun_controller.gun_scale),

                                end_position: ray_intersection.point,
                                life_time: 0.3,
                            },
                        ));
                        let mut spawn_bullet_hole = true;
                        if let Ok(head) = head_query.get_mut(entity)
                        {
                            if let Ok(mut enemy) = enemy_query.get_mut(head.enemy_reference)
                            {
                                enemy.health-= 100.;
                            }
                            spawn_bullet_hole = false;
                        }
                        if let Ok(leg) = leg_query.get_mut(entity)
                        {
                            if let Ok(mut enemy) = enemy_query.get_mut(leg.enemy_reference)
                            {
                                enemy.health-= 10.;
                            }
                            spawn_bullet_hole = false;
                        }
                        if let Ok(body) = body_query.get_mut(entity)
                        {
                            if let Ok(mut enemy) = enemy_query.get_mut(body.enemy_reference)
                            {
                                enemy.health-= 20.;
                            }
                            spawn_bullet_hole = false;
                        }
                        if spawn_bullet_hole == true {
                            let texture_handle = asset_server.load("bullet_hole.png");

                            // create a new quad mesh. this is what we will apply the texture to
                            let quad_width = 0.07;
                            let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                                quad_width, quad_width,
                            ))));

                            // this material renders the texture normally
                            let material_handle = materials.add(StandardMaterial {
                                base_color: Color::rgba(1., 1., 1., 1.),
                                base_color_texture: Some(texture_handle.clone()),
                                alpha_mode: AlphaMode::Blend,

                                cull_mode: None,
                                unlit: true,
                                ..default()
                            });

                            let hole_position =
                                ray_intersection.point + ray_intersection.normal * 0.045;

                            let offseted_normal =
                                ray_intersection.normal + Vec3::new(0.00001, 0.00001, 0.00001);
                            let mut hole_transform = Transform::from_xyz(
                                hole_position.x,
                                hole_position.y,
                                hole_position.z,
                            );
                            hole_transform.rotation =
                                quaternion_look_rotation(offseted_normal, Vec3::Y);
                            let _hole_entity_front = commands
                                .spawn(PbrBundle {
                                    mesh: quad_handle.clone(),
                                    material: material_handle,
                                    transform: hole_transform,
                                    ..default()
                                })
                                .insert(NotShadowCaster)
                                .id();
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_target_position(rng: &mut rand::rngs::ThreadRng) -> Vec3 {
    return Vec3::new(
        rng.gen_range(-5..5) as f32 * 0.4,
        rng.gen_range(1..11) as f32 * 0.4,
        0.,
    );
}
pub fn update_targets(
    _commands: Commands,
    mut target_query: Query<(&mut ShootableTarget, &mut Transform)>,
) {
    let mut pos_vec = Vec::new();
    for (_target, transform) in target_query.iter() {
        pos_vec.push(transform.translation);
    }
    let mut i = 0;
    for (mut target, mut transform) in target_query.iter_mut() {
        let original_position;
        if target.health <= 0. {
            let mut rng: rand::rngs::ThreadRng = rand::thread_rng();
            original_position = pos_vec[i];
            pos_vec[i] = generate_target_position(&mut rng);
            transform.translation = pos_vec[i];
            target.health = target.max_health;

            let mut unique = false;
            while unique == false {
                for j in 0..pos_vec.len() {
                    unique = true;
                    if (pos_vec[i] == pos_vec[j] || pos_vec[i] == original_position) && i != j {
                        pos_vec[i] = generate_target_position(&mut rng);
                        transform.translation = pos_vec[i];
                        unique = false;
                        break;
                    }
                }
            }
        }
        i += 1;
    }
}
