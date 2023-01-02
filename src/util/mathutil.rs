const EPSILON: f32 = 1.0 / 32.0;

pub fn point_in_box(point: glm::Vec3, min: glm::Vec3, max: glm::Vec3) -> bool {
    return (min.x <= point.x && point.x <= max.x && min.y <= point.y && point.y <= max.y && min.z <= point.z && point.z <= max.z) ||
	   (min.x >= point.x && point.x >= max.x && min.y >= point.y && point.y >= max.y && min.z >= point.z && point.z >= max.z);
}

pub fn point_in_plane(point: glm::Vec3, normal: glm::Vec3, dist: f32) -> bool {
    return (glm::dot(point, normal) - dist).abs() < EPSILON;
}
