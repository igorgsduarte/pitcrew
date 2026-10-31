//! The matrix test: fixed screens that show if the color order, the wiring and the brightness are
//! right. The user steps through them in the setup's Test step and changes the matrix settings
//! while a screen shows; the board applies each change at once.

use crate::pack::Animation;

pub struct Step {
    pub id: &'static str,
    /// What the matrix must show.
    pub expect: &'static str,
    /// What to change when it does not.
    pub fix: &'static str,
}

const COLOR_FIX: &str = "If it shows another color, change Color order until red, green and blue are right.";

pub const STEPS: [Step; 6] = [
    Step { id: "red", expect: "The whole matrix is red.", fix: COLOR_FIX },
    Step { id: "green", expect: "The whole matrix is green.", fix: COLOR_FIX },
    Step { id: "blue", expect: "The whole matrix is blue.", fix: COLOR_FIX },
    Step {
        id: "white",
        expect: "The whole matrix is white.",
        fix: "A dim white is normal: the current limit lowers the brightness when every LED is on.",
    },
    Step {
        id: "orientation",
        expect: "Top row red, left column green, the top-left LED white, the bottom-right LED blue.",
        fix: "If the white LED is in another corner, change Rotate until it is top left. If the image zigzags or is mirrored, change Zigzag wiring or Mirror.",
    },
    Step {
        id: "brightness",
        expect: "One row, from dim on the left to bright on the right.",
        fix: "Change Brightness to set how bright the flags are.",
    },
];

/// The steps to show for the matrix. A one-color matrix cannot show the color screens.
pub fn steps(one_color: bool) -> Vec<usize> {
    (0..STEPS.len()).filter(|&i| !one_color || matches!(STEPS[i].id, "white" | "orientation")).collect()
}

/// The settings that fix what step `id` checks.
pub fn fix_keys(id: &str, one_color: bool) -> &'static [&'static str] {
    match id {
        "red" | "green" | "blue" => &["colorOrder", "colorCorrection"],
        "white" if !one_color => &["maxMilliamps"],
        "orientation" if one_color => &["rotate", "flipX", "flipY"],
        "orientation" => &["rotate", "serpentine", "flipX", "flipY"],
        "brightness" => &["brightness"],
        _ => &[],
    }
}

/// Every screen's name starts with this.
pub const NAME_PREFIX: &str = "matrix test: ";

/// The screen of step `i` as a one-frame animation, named `matrix test: <step>`.
pub fn screen(i: usize) -> Animation {
    let step = &STEPS[i.min(STEPS.len() - 1)];
    let mut frame = [0u8; 64];
    let colors = match step.id {
        "orientation" => {
            frame[..8].fill(1);
            for y in 0..8 {
                frame[y * 8] = 2;
            }
            frame[0] = 3;
            frame[63] = 4;
            vec![0x000000, 0xFF0000, 0x00FF00, 0xFFFFFF, 0x0000FF]
        }
        "brightness" => {
            for x in 0..8 {
                frame[4 * 8 + x] = x as u8 + 1;
            }
            let mut c = vec![0x000000];
            c.extend((1..=8u32).map(|n| 0x010101 * (32 * n - 1)));
            c
        }
        id => {
            frame = [1; 64];
            let solid = match id {
                "red" => 0xFF0000,
                "green" => 0x00FF00,
                "blue" => 0x0000FF,
                _ => 0xFFFFFF,
            };
            vec![0x000000, solid]
        }
    };
    let letters = ".abcdefghijklmno".chars().take(colors.len()).collect();
    Animation { id: format!("{NAME_PREFIX}{}", step.id), letters, colors, fps: 1, looped: true, frames: vec![frame] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_one_color_matrix_skips_the_color_screens() {
        assert_eq!(steps(false), (0..STEPS.len()).collect::<Vec<_>>());
        let mono: Vec<&str> = steps(true).into_iter().map(|i| STEPS[i].id).collect();
        assert_eq!(mono, vec!["white", "orientation"]);
    }

    #[test]
    fn every_step_has_a_screen_with_a_valid_palette() {
        for i in 0..STEPS.len() {
            let a = screen(i);
            assert!(a.colors.len() <= 16, "{}", a.id);
            assert_eq!(a.colors[0], 0, "{}: index 0 is off", a.id);
            assert!(a.frames[0].iter().all(|&p| (p as usize) < a.colors.len()), "{}", a.id);
        }
    }

    #[test]
    fn the_orientation_screen_marks_the_top_row_the_left_column_and_two_corners() {
        let a = screen(4);
        let f = &a.frames[0];
        let color = |x: usize, y: usize| a.colors[f[y * 8 + x] as usize];
        assert_eq!(color(0, 0), 0xFFFFFF);
        assert_eq!(color(7, 7), 0x0000FF);
        assert!((1..8).all(|x| color(x, 0) == 0xFF0000));
        assert!((1..8).all(|y| color(0, y) == 0x00FF00));
        assert_eq!(f.iter().filter(|&&p| p != 0).count(), 16);
    }

    #[test]
    fn the_brightness_screen_goes_from_dim_to_full_along_one_row() {
        let a = screen(5);
        let row: Vec<u32> = (0..8).map(|x| a.colors[a.frames[0][4 * 8 + x] as usize]).collect();
        assert!(row.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(row[7], 0xFFFFFF);
        assert_eq!(a.frames[0].iter().filter(|&&p| p != 0).count(), 8);
    }
}
