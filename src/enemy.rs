use bevy::{prelude::*, render::render_resource::encase::rts_array::Length};

use crate::{AnimationEntityLink, EnemyAnimations, fps_movement::FPSMovement};

#[derive(Component)]
pub struct Enemy
{
    pub parented : bool,
    pub shoot_timer: f32,
    pub shoot_cooldown : f32,
}

pub fn rotate_to_player(
    enemy_animations: Res<EnemyAnimations>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    mut player_query: Query<(&Transform,&FPSMovement)>,
    mut commands :Commands,
    mut enemy_query: Query<(&Enemy,&Children, &AnimationEntityLink)>,
    mut get_child_query : Query<(&Children)>,
    mut name_query: Query<(&Name)>,
    mut transform_query : Query<(&mut Transform),Without<FPSMovement>>,
)
{
    if let Ok((player_transform,movement)) = player_query.get_single()
    {
        for(enemy,child,animation_entity) in enemy_query.iter_mut()
        {
            if let Ok(mut player) = animation_player_query.get_mut(animation_entity.0) {
                player.play(enemy_animations.0[0].clone_weak());
                player.pause();
            }
            if let Ok(child_of_child) = get_child_query.get(child[0])
            {
                if let Ok(child_of_child_of_child) = get_child_query.get(child_of_child[0])
                {
                    for i in 0..child_of_child_of_child.len()
                    {
                        if let Ok(name) = name_query.get(child_of_child_of_child[i])
                        {
                            //println!("{} {}",child_of_child_of_child.len(),name.as_str());
                            if name.as_str()  == "Body"
                            {
                                if let Ok(mut transform) = transform_query.get_mut(child_of_child_of_child[i])
                                {
                                    
                                    let look_pos = player_transform.translation;
                                    transform.look_at(look_pos,Vec3::Y);
                                    println!("{}",transform.rotation);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}