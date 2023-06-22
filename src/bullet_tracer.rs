use bevy::prelude::*;

use crate::vector_operations::move_towards;

#[derive(Component)]
pub struct BulletTracer {
    pub start_position: Vec3,
    pub end_position: Vec3,
    pub life_time: f32,
    pub direction: Vec3,
}

pub fn update_tracers(
    mut commands: Commands,
    mut tracer_query: Query<(&mut BulletTracer, &mut Transform, Entity)>,
    time: Res<Time>,
) {
    for (mut tracer, mut transform, entity) in tracer_query.iter_mut() {
        tracer.life_time -= time.delta_seconds();

        transform.translation = (tracer.start_position + tracer.end_position) / 2.;
        transform.scale.z = Vec3::distance(tracer.start_position, tracer.end_position);
        transform.scale.y = 0.003;
        transform.scale.x = 0.003;
        transform.look_at(tracer.end_position, Vec3::Y);

        if tracer.direction == Vec3::new(0., 0., 0.) {
            tracer.direction = transform.forward();
        }
        //tracer.start_position+=direction*time.delta_seconds()*20.0;
        tracer.start_position = move_towards(
            tracer.start_position,
            tracer.end_position,
            time.delta_seconds() * 50.,
        );
        if tracer.start_position == tracer.end_position {
            commands.entity(entity).despawn();
        }
    }
}
