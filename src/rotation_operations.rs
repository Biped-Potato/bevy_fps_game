use bevy::prelude::*;

pub fn from_to_rotation(a_from : Vec3, a_to : Vec3)->Quat
{
    let axis = Vec3::cross(a_from, a_to);
    let angle = angle_func(a_from, a_to);
    return Quat::from_axis_angle(Vec3::normalize(axis),angle);
}
pub fn angle_func(from : Vec3, to : Vec3)->f32
{
    return f32::acos(f32::clamp(Vec3::dot(Vec3::normalize(from), Vec3::normalize(to)), -1.0, 1.0)) * 57.29578;
}
pub fn quaternion_look_rotation(mut forward: Vec3, up:Vec3)->Quat
{
    forward = forward.normalize();

    let vector  = Vec3::normalize(forward);
    let vector2 = Vec3::normalize(Vec3::cross(up, vector));
    let vector3 = Vec3::cross(vector, vector2);
    
    let m00 = vector2.x;
    let m01 = vector2.y;
    let m02 = vector2.z;
    let m10 = vector3.x;
    let m11 = vector3.y;
    let m12 = vector3.z;
    let m20 = vector.x;
    let m21 = vector.y;
    let m22 = vector.z;


    let num8 = (m00 + m11) + m22;
    let mut quaternion = Quat::IDENTITY;
    if num8 > 0.
    {
        let mut num = f32::sqrt(num8 + 1.);
        quaternion.w = num * 0.5;
        num = 0.5 / num;
        quaternion.x = (m12 - m21) * num;
        quaternion.y = (m20 - m02) * num;
        quaternion.z = (m01 - m10) * num;
        return quaternion;
    }
    if (m00 >= m11) && (m00 >= m22)
    {
        let num7 = f32::sqrt(((1. + m00) - m11) - m22);
        let num4 = 0.5 / num7;
        quaternion.x = 0.5 * num7;
        quaternion.y = (m01 + m10) * num4;
        quaternion.z = (m02 + m20) * num4;
        quaternion.w = (m12 - m21) * num4;
        return quaternion;
    }
    if m11 > m22
    {
        let num6 = f32::sqrt(((1. + m11) - m00) - m22);
        let num3 = 0.5 / num6;
        quaternion.x = (m10+ m01) * num3;
        quaternion.y = 0.5 * num6;
        quaternion.z = (m21 + m12) * num3;
        quaternion.w = (m20 - m02) * num3;
        return quaternion; 
    }
    let num5 = f32::sqrt(((1. + m22) - m00) - m11);
    let num2 = 0.5 / num5;
    quaternion.x = (m20 + m02) * num2;
    quaternion.y = (m21 + m12) * num2;
    quaternion.z = 0.5 * num5;
    quaternion.w = (m01 - m10) * num2;
    return quaternion;
}