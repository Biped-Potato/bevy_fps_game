use bevy::{core::Zeroable, input::mouse::MouseMotion, prelude::*};

use crate::{lock_cursor::CursorLockState, vector_operations::move_towards};

#[derive(Component)]
pub struct FPSCamera {
    pub speed: f32,
    pub sensitivity: f32,
    pub rotate_lock: f32,

    pub rotation: Vec3,
    pub recoil_shake: Vec3,
    pub camera_shake_readjustment_factor: f32,
}

pub fn move_camera(
    cursor_lock_state: Res<CursorLockState>,
    mut motion_evr: EventReader<MouseMotion>,
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut FPSCamera)>,
) {
    if cursor_lock_state.state {
        for (mut transform, mut camera) in camera_query.iter_mut() {
            for ev in motion_evr.iter() {
                camera.rotation.y -= ev.delta.x * camera.sensitivity;
                camera.rotation.x -= ev.delta.y * camera.sensitivity;

                camera.rotation.x =
                    f32::clamp(camera.rotation.x, -camera.rotate_lock, camera.rotate_lock);

                //transform.rotation += Quat::from_axis_angle(Vec3::new(0., 1., 0.), angle);
                //println!("{}",x_quat);
            }
            camera.recoil_shake = move_towards(
                camera.recoil_shake,
                Vec3::ZERO,
                time.delta_seconds() * camera.camera_shake_readjustment_factor,
            );
            let x_quat = Quat::from_axis_angle(
                Vec3::new(0., 1., 0.),
                camera.rotation.y - camera.recoil_shake.x,
            );

            let y_quat = Quat::from_axis_angle(
                Vec3::new(1., 0., 0.),
                camera.rotation.x + camera.recoil_shake.y,
            );

            transform.rotation = x_quat * y_quat;
        }
    }
}
