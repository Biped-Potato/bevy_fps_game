use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::fps_camera::FPSCamera;
#[derive(Component)]
pub struct FPSMovement {
    pub acceleration: f32,

    pub speed: f32,
}

pub fn player_movement(
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut movement_query: Query<(
        &Transform,
        &mut Damping,
        &mut FPSCamera,
        &mut FPSMovement,
        &mut Velocity,
    )>,
    key: Res<Input<KeyCode>>,
) {
    for (transform, mut damping, camera, movement, mut velocity) in
        movement_query.iter_mut()
    {
        
        
        let mut direction = Vec2::new(0., 0.);

        let mut air_modifier = 1.0;

        let mut grounded = false;
        let ray = Ray {
            origin: transform.translation - 0.5,
            direction: Vec3::new(0., -1., 0.),
        };

        // Then cast the ray.
        let hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction,
            1.,
            true,
            QueryFilter::only_fixed(),
        );

        if let Some((_entity, _toi)) = hit {
            grounded = true;
        }

        if grounded == false {
            damping.linear_damping = 0.;
            air_modifier = 0.05;
        } else {
            damping.linear_damping = 8.;
        }
        if key.pressed(KeyCode::W) {
            direction.y += -f32::cos(camera.rotation.y);
            direction.x += -f32::sin(camera.rotation.y);
        } else if key.pressed(KeyCode::S) {
            direction.y += f32::cos(camera.rotation.y);
            direction.x += f32::sin(camera.rotation.y);
        }
        if key.pressed(KeyCode::D) {
            direction.y += f32::cos(camera.rotation.y + f32::to_radians(90.));
            direction.x += f32::sin(camera.rotation.y + f32::to_radians(90.));
        } else if key.pressed(KeyCode::A) {
            direction.y += f32::cos(camera.rotation.y - f32::to_radians(90.));
            direction.x += f32::sin(camera.rotation.y - f32::to_radians(90.));
        }

        if key.just_pressed(KeyCode::Space) {
            if grounded {
                velocity.linvel.y = 4.;
            }
        }

        if direction.length() != 0. {
            direction = direction.normalize();
        }

        velocity.linvel.x +=
            direction.x * movement.acceleration * time.delta_seconds() * air_modifier;
        velocity.linvel.z +=
            direction.y * movement.acceleration * time.delta_seconds() * air_modifier;

        let net_velocity = Vec2::new(velocity.linvel.x, velocity.linvel.z).length();
        let multiplier;
        if net_velocity > movement.speed {
            multiplier = movement.speed / net_velocity;
        } else {
            multiplier = 1.0;
        }

        velocity.linvel.x *= multiplier;
        velocity.linvel.z *= multiplier;
        //velocity.linvel.x = f32::clamp(velocity.linvel.x,-movement.speed,movement.speed);
        //velocity.linvel.z = f32::clamp(velocity.linvel.z,-movement.speed,movement.speed);
    }
}
