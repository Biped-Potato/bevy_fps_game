use bevy::prelude::*;

pub fn move_towards(current:Vec3,target:Vec3, max_dist_delta:f32)->Vec3
{
    let a: Vec3 = target - current;
    let magnitude = Vec3::length(a);
    if magnitude <= max_dist_delta || magnitude == 0.
    {
        return target;
    }
    return current + a / magnitude * max_dist_delta;
}