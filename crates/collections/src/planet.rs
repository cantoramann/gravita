use gravita_math::Vec2;
use gravita_renderer::{draw_circle, draw_line};

/// A simple 2D planet / gravity well visual.
///
/// Intended to pair with physics `RadialForce` or `FieldForce` to show
/// areas of influence in space or arena-style games.
pub struct Planet {
    /// Center position in world space.
    pub center: Vec2,
    /// Radius of the solid planet body.
    pub radius: f32,
    /// Optional radius of an "atmosphere" or influence field.
    pub field_radius: Option<f32>,
    /// Solid planet color.
    pub color: [u8; 4],
    /// Color used for the field/atmosphere outline.
    pub field_color: [u8; 4],
}

impl Planet {
    /// Create a new planet with a soft blue body and faint outer field.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            field_radius: Some(radius * 2.5),
            color: [0x40, 0x80, 0xFF, 0xFF],
            field_color: [0x40, 0x80, 0xFF, 0x60],
        }
    }

    /// Set the radius of the planet's field/atmosphere in world units.
    pub fn set_field_radius(&mut self, r: Option<f32>) {
        self.field_radius = r;
    }

    /// Render the planet body and optional field.
    pub fn render(&self, frame: &mut [u8], width: u32, height: u32) {
        // Draw field first so it appears behind the solid body.
        if let Some(field_r) = self.field_radius {
            draw_circle(frame, self.center, field_r, self.field_color, width, height);
        }

        // Draw the solid planet.
        draw_circle(frame, self.center, self.radius, self.color, width, height);

        // Simple axis marker to aid orientation (a short line "north" of the planet).
        let north = self.center + Vec2::new(0.0, self.radius * 0.8);
        let marker_color = [0xFF, 0xFF, 0xFF, 0xFF];
        draw_line(frame, self.center, north, marker_color, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn new_planet_at_position() {
        let planet = Planet::new(Vec2::new(100.0, 200.0), 50.0);
        assert_eq!(planet.center, Vec2::new(100.0, 200.0));
        assert_eq!(planet.radius, 50.0);
    }

    #[test]
    fn new_planet_has_default_field_radius() {
        let planet = Planet::new(Vec2::ZERO, 50.0);
        // Default field radius is 2.5x the body radius
        assert_eq!(planet.field_radius, Some(125.0));
    }

    #[test]
    fn new_planet_has_default_colors() {
        let planet = Planet::new(Vec2::ZERO, 50.0);
        // Color should be set (not all zeros)
        assert!(planet.color[3] > 0);
        assert!(planet.field_color[3] > 0);
    }

    // =========================================================================
    // Configuration
    // =========================================================================

    #[test]
    fn set_field_radius_to_custom_value() {
        let mut planet = Planet::new(Vec2::ZERO, 50.0);
        planet.set_field_radius(Some(200.0));
        assert_eq!(planet.field_radius, Some(200.0));
    }

    #[test]
    fn set_field_radius_to_none() {
        let mut planet = Planet::new(Vec2::ZERO, 50.0);
        planet.set_field_radius(None);
        assert_eq!(planet.field_radius, None);
    }

    // =========================================================================
    // Rendering (smoke test)
    // =========================================================================

    #[test]
    fn render_does_not_panic() {
        let planet = Planet::new(Vec2::new(400.0, 300.0), 50.0);
        let mut frame = vec![0u8; 800 * 600 * 4];
        planet.render(&mut frame, 800, 600);
    }

    #[test]
    fn render_without_field_does_not_panic() {
        let mut planet = Planet::new(Vec2::new(400.0, 300.0), 50.0);
        planet.set_field_radius(None);
        let mut frame = vec![0u8; 800 * 600 * 4];
        planet.render(&mut frame, 800, 600);
    }

    #[test]
    fn render_at_edge_does_not_panic() {
        let planet = Planet::new(Vec2::new(0.0, 0.0), 50.0);
        let mut frame = vec![0u8; 800 * 600 * 4];
        planet.render(&mut frame, 800, 600);
    }
}
