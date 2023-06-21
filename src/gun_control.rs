use std::time::Duration;

use bevy::{prelude::*, render::render_resource::encase::rts_array::Length};

use crate::{fps_camera::FPSCamera, vector_operations::move_towards, Animations, AnimationEntityLink};
#[derive(Component)]
pub struct GunController
{
    pub magazine_size : usize,
    pub timer : f32,
    pub cooldown : f32,
    pub offset : Vec3,
    pub gun_scale : f32,
    pub shoot : bool,
    pub dynamic_offset : Vec3,
    pub target_offset : Vec3,
    pub spray_pattern : Vec<Vec2>,
    pub spray_index : usize,
    pub recoil_reset_time : f32,
    pub time_since_last_shot : f32,
    pub smooth_scale : f32,
    pub current_camera_transform : Transform,
    pub recoil_shake : Vec3,
    pub aiming_down_sights : bool,
    pub spray_rand : f32,
}
pub fn translate_gun_position(camera_transform : &Transform)->Vec3
{
    let mut position = Vec3::new(0.,0.,0.);
    position = camera_transform.translation;
    position+=camera_transform.forward()*0.35;
    position+=camera_transform.right()*0.4;
    position+=camera_transform.down()*0.3; 
    return position;
}
pub fn update_gun_control(
    mut player_query: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
    time: Res<Time>,
    mut gun_query: Query<(&mut Transform,&mut GunController, &AnimationEntityLink),(Without<FPSCamera>)>,
    mut camera_query: Query<(&mut Transform,&FPSCamera),(Without<GunController>)>
)
{
    if let Ok ((camera_transform,camera)) = camera_query.get_single_mut()
    {
        if let Ok ((mut transform,mut gun_controller,animation_entity)) = gun_query.get_single_mut()
        {
            if let Ok(mut player) = player_query.get_mut(animation_entity.0) {
                player.play(animations.0[0].clone_weak()).repeat();
            }
            let mut placebo_camera = Transform::from_xyz(camera_transform.translation.x,camera_transform.translation.y,camera_transform.translation.z);


            gun_controller.recoil_shake = move_towards(gun_controller.recoil_shake,Vec3::ZERO,time.delta_seconds()*gun_controller.smooth_scale);
            
            let x_quat = Quat::from_axis_angle(Vec3::new(0., 1., 0.), camera.rotation.y-gun_controller.recoil_shake.x);

            let y_quat = Quat::from_axis_angle(Vec3::new(1., 0., 0.), camera.rotation.x+gun_controller.recoil_shake.y);

            
            placebo_camera.rotation = x_quat * y_quat;
            transform.translation = translate_gun_position(&placebo_camera);

            transform.look_at(camera_transform.translation+placebo_camera.forward()*100., Vec3::Y);            
            transform.scale = Vec3::new(gun_controller.gun_scale,gun_controller.gun_scale,gun_controller.gun_scale);
            if gun_controller.shoot
            {
                let mut rng = rand::thread_rng();
                //gun_cotnroller.target_offset = Vec3::new(rng::gen_range())
                gun_controller.shoot = false;
            }
        }    
    }
   
}