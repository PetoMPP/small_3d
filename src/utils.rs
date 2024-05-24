use bevy::{math::bounding::Bounded2d, prelude::*};

pub fn contains_point(rect: Rect, corner_radius: f32, point: Vec2) -> bool {
    // check rectangles between the corners
    let width = rect.width() - corner_radius * 2.0;
    let height = rect.height() - corner_radius * 2.0;
    if width > 0.0 {
        let vert_rect = Rect::new(
            rect.min.x + corner_radius,
            rect.min.y,
            rect.max.x - corner_radius,
            rect.max.y,
        );
        if vert_rect.contains(point) {
            return true;
        }
    }
    if height > 0.0 {
        let horz_rect = Rect::new(
            rect.min.x,
            rect.min.y + corner_radius,
            rect.max.x,
            rect.max.y - corner_radius,
        );
        if horz_rect.contains(point) {
            return true;
        }
    }

    // check bounding rect
    if !rect.contains(point) {
        return false;
    }

    // check corner circles
    let corner_radius = corner_radius.min(rect.width()).min(rect.height());
    let lt = rect.min + Vec2::splat(corner_radius);
    let rt = Vec2::new(rect.max.x - corner_radius, rect.min.y + corner_radius);
    let lb = Vec2::new(rect.min.x + corner_radius, rect.max.y - corner_radius);
    let rb = rect.max - Vec2::splat(corner_radius);
    let lt_rect = Rect::new(rect.min.x, rect.min.y, lt.x, lt.y);
    let lt_circle = Circle::new(corner_radius).bounding_circle(lt, 0.0);
    if lt_rect.contains(point) && lt_circle.closest_point(point) == point {
        return true;
    }
    let rt_rect = Rect::new(rt.x, rect.min.y, rect.max.x, rt.y);
    let rt_circle = Circle::new(corner_radius).bounding_circle(rt, 0.0);
    if rt_rect.contains(point) && rt_circle.closest_point(point) == point {
        return true;
    }
    let lb_rect = Rect::new(rect.min.x, lb.y, lb.x, rect.max.y);
    let lb_circle = Circle::new(corner_radius).bounding_circle(lb, 0.0);
    if lb_rect.contains(point) && lb_circle.closest_point(point) == point {
        return true;
    }
    let rb_rect = Rect::new(rb.x, rb.y, rect.max.x, rect.max.y);
    let rb_circle = Circle::new(corner_radius).bounding_circle(rb, 0.0);
    if rb_rect.contains(point) && rb_circle.closest_point(point) == point {
        return true;
    }

    false
}

pub fn rotate_point(point: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let offset = point - center;
    let rot = Quat::from_rotation_z(angle);
    center + rot.mul_vec3(offset.extend(0.0)).truncate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_point() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let corner_radius = 10.0;

        // test corners
        assert!(!contains_point(rect, corner_radius, Vec2::new(0.0, 0.0)));
        assert!(!contains_point(rect, corner_radius, Vec2::new(100.0, 0.0)));
        assert!(!contains_point(rect, corner_radius, Vec2::new(0.0, 100.0)));
        assert!(!contains_point(
            rect,
            corner_radius,
            Vec2::new(100.0, 100.0)
        ));

        // test edges
        assert!(contains_point(rect, corner_radius, Vec2::new(50.0, 0.0)));
        assert!(contains_point(rect, corner_radius, Vec2::new(0.0, 50.0)));
        assert!(contains_point(rect, corner_radius, Vec2::new(50.0, 100.0)));
        assert!(contains_point(rect, corner_radius, Vec2::new(100.0, 50.0)));

        // test inside
        assert!(contains_point(rect, corner_radius, Vec2::new(50.0, 50.0)));

        // test outside
        assert!(!contains_point(rect, corner_radius, Vec2::new(101.0, 50.0)));
        assert!(!contains_point(rect, corner_radius, Vec2::new(50.0, 101.0)));
        assert!(!contains_point(rect, corner_radius, Vec2::new(-1.0, 50.0)));
        assert!(!contains_point(rect, corner_radius, Vec2::new(50.0, -1.0)));
    }
}
