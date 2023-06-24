use crate::{fps_movement::FPSMovement, AnimationEntityLink, EnemyAnimations};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub added_colliders: bool,
    pub shoot_timer: f32,
    pub shoot_cooldown: f32,
    pub health : f32,
}
#[derive(Component)]
pub struct HeadCollider {
    pub enemy_reference: Entity,
}
#[derive(Component)]
pub struct BodyCollider {
    pub enemy_reference: Entity,
}
#[derive(Component)]
pub struct LegCollider {
    pub enemy_reference: Entity,
}

pub fn rotate_to_player(
    enemy_animations: Res<EnemyAnimations>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    player_query: Query<(&Transform, &FPSMovement)>,
    mut commands: Commands,
    mut enemy_query: Query<(&mut Enemy, Entity, &Children, &AnimationEntityLink)>,
    get_child_query: Query<&Children>,
    name_query: Query<&Name>,
    _transform_query: Query<&mut Transform, Without<FPSMovement>>,
) {
    if let Ok((_player_transform, _movement)) = player_query.get_single() {
        for (mut enemy, entity, child, animation_entity) in enemy_query.iter_mut() {
            if let Ok(mut player) = animation_player_query.get_mut(animation_entity.0) {
                if enemy.health <=0.
                {
                    player.play(enemy_animations.0[1].clone_weak());
                }
                else {
                    player.play(enemy_animations.0[0].clone_weak());
                }
                
            }
            if enemy.added_colliders == false {
                if let Ok(child_of_child) = get_child_query.get(child[0]) {
                    if let Ok(child_of_child_of_child) = get_child_query.get(child_of_child[0]) {
                        for i in 0..child_of_child_of_child.len() {
                            if let Ok(name) = name_query.get(child_of_child_of_child[i]) {
                                //println!("{} {}",child_of_child_of_child.len(),name.as_str());
                                if name.as_str() == "Rear" {
                                    let collider_entity_body = commands
                                    .spawn(Collider::cuboid(0.5, 1.225, 0.5))
                                    .insert(TransformBundle {
                                        local: Transform::from_xyz(0., -1.55, 0.),
                                        ..default()
                                    })
                                    .insert(ColliderDebugColor(Color::RED))
                                    .insert(LegCollider {
                                        enemy_reference: entity,
                                    })
                                    .id();
                                    commands
                                        .entity(child_of_child_of_child[i])
                                        .push_children(&[collider_entity_body]);
                                    }
                                else if name.as_str() == "Body" {
                                    let collider_entity_body = commands
                                        .spawn(Collider::cuboid(0.5, 1.5, 0.5))
                                        .insert(TransformBundle {
                                            local: Transform::from_xyz(0., 0.8, 0.),
                                            ..default()
                                        })
                                        
                                        .insert(ColliderDebugColor(Color::BLUE))
                                        .insert(BodyCollider {
                                            enemy_reference: entity,
                                        })
                                        .id();
                                    commands
                                        .entity(child_of_child_of_child[i])
                                        .push_children(&[collider_entity_body]);

                                    if let Ok(child_of_child_of_child_body_child) =
                                        get_child_query.get(child_of_child_of_child[i])
                                    {
                                        for j in 0..child_of_child_of_child_body_child.len() {
                                            if let Ok(body_child_name) = name_query
                                                .get(child_of_child_of_child_body_child[j])
                                            {
                                                if body_child_name.as_str() == "Head" {
                                                    let collider_entity = commands
                                                        .spawn(Collider::cuboid(0.2, 0.4, 0.2))
                                                        .insert(TransformBundle {
                                                            local: Transform::from_xyz(0., 0.5, 0.),
                                                            ..default()
                                                        })
                                                        .insert(HeadCollider {
                                                            enemy_reference: entity,
                                                        })
                                                        .insert(ColliderDebugColor(Color::GREEN))
                                                        .id();
                                                    commands
                                                        .entity(
                                                            child_of_child_of_child_body_child[j],
                                                        )
                                                        .push_children(&[collider_entity]);
                                                    enemy.added_colliders = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
