pub const CLOCK_FACE_ICON_SIZE: u32 = 32;

pub fn clock_face_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = vec![0_u8; (size * size * 4) as usize];

    let centre = (size as f32 - 1.0) / 2.0;
    let outer_radius = size as f32 * 0.45;
    let border_inner_radius = size as f32 * 0.36;
    let hour_hand_length = size as f32 * 0.18;
    let minute_hand_length = size as f32 * 0.28;
    let hand_thickness = (size as f32 * 0.05).max(1.25);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - centre;
            let dy = y as f32 - centre;
            let distance = (dx * dx + dy * dy).sqrt();
            let pixel = ((y * size + x) * 4) as usize;

            let mut colour = None;

            if distance <= outer_radius {
                colour = Some([245, 245, 240, 255]);
            }

            if distance >= border_inner_radius && distance <= outer_radius {
                colour = Some([52, 58, 64, 255]);
            }

            if on_vertical_hand(dx, dy, hour_hand_length, hand_thickness)
                || on_horizontal_hand(dx, dy, minute_hand_length, hand_thickness)
            {
                colour = Some([36, 42, 48, 255]);
            }

            if distance <= size as f32 * 0.06 {
                colour = Some([40, 180, 140, 255]);
            }

            if let Some([r, g, b, a]) = colour {
                rgba[pixel] = r;
                rgba[pixel + 1] = g;
                rgba[pixel + 2] = b;
                rgba[pixel + 3] = a;
            }
        }
    }

    rgba
}

fn on_vertical_hand(dx: f32, dy: f32, length: f32, thickness: f32) -> bool {
    dx.abs() <= thickness && dy <= thickness && dy >= -length
}

fn on_horizontal_hand(dx: f32, dy: f32, length: f32, thickness: f32) -> bool {
    dy.abs() <= thickness && dx >= -thickness && dx <= length
}

#[cfg(test)]
mod tests {
    use super::{clock_face_icon_rgba, CLOCK_FACE_ICON_SIZE};

    #[test]
    fn icon_buffer_matches_dimensions() {
        let rgba = clock_face_icon_rgba(CLOCK_FACE_ICON_SIZE);
        assert_eq!(
            rgba.len(),
            (CLOCK_FACE_ICON_SIZE * CLOCK_FACE_ICON_SIZE * 4) as usize
        );
    }

    #[test]
    fn icon_contains_non_transparent_pixels() {
        let rgba = clock_face_icon_rgba(CLOCK_FACE_ICON_SIZE);
        assert!(rgba.chunks_exact(4).any(|pixel| pixel[3] > 0));
    }
}
