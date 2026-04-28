use crate::ai::npc_avatar::{CameraWaypoint, CinemaLight};

pub fn generate_orbit_waypoints(
    center: [f32; 3],
    radius: f32,
    height: f32,
    steps: u32,
) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    let cam_y = center[2] + height;
    for i in 0..steps {
        let angle = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let x = center[0] + radius * angle.cos();
        let y = center[1] + radius * angle.sin();
        waypoints.push(CameraWaypoint {
            position: [x, y, cam_y],
            focus: center,
            fov: 60.0,
            dwell: 0.5,
        });
    }
    waypoints
}

pub fn generate_dolly_waypoints(
    start: [f32; 3],
    end: [f32; 3],
    focus: [f32; 3],
    steps: u32,
) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        waypoints.push(CameraWaypoint {
            position: [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ],
            focus,
            fov: 60.0,
            dwell: 0.3,
        });
    }
    waypoints
}

pub fn generate_crane_waypoints(
    base: [f32; 3],
    height: f32,
    focus: [f32; 3],
    steps: u32,
) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = t * std::f32::consts::FRAC_PI_4;
        let z = base[2] + height * t;
        let offset = height * 0.3 * angle.sin();
        waypoints.push(CameraWaypoint {
            position: [base[0] - offset, base[1], z],
            focus,
            fov: 60.0 + 10.0 * t,
            dwell: 0.4,
        });
    }
    waypoints
}

pub fn generate_flyby_waypoints(subject: [f32; 3], steps: u32) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    let start = [subject[0] - 15.0, subject[1] - 10.0, subject[2] + 8.0];
    let end = [subject[0] + 15.0, subject[1] + 10.0, subject[2] + 3.0];
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        waypoints.push(CameraWaypoint {
            position: [
                start[0] + (end[0] - start[0]) * t,
                start[1] + (end[1] - start[1]) * t,
                start[2] + (end[2] - start[2]) * t,
            ],
            focus: subject,
            fov: 50.0 + 20.0 * t,
            dwell: 0.2,
        });
    }
    waypoints
}

pub fn generate_reveal_waypoints(subject: [f32; 3], steps: u32) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let dist = 3.0 + 20.0 * t;
        let fov = 30.0 + 50.0 * t;
        waypoints.push(CameraWaypoint {
            position: [subject[0] - dist, subject[1], subject[2] + dist * 0.3],
            focus: subject,
            fov,
            dwell: 0.5,
        });
    }
    waypoints
}

pub fn generate_tracking_waypoints(
    start: [f32; 3],
    end: [f32; 3],
    offset: f32,
    steps: u32,
) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let perp_x = -dy / len * offset;
    let perp_y = dx / len * offset;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let subject = [
            start[0] + dx * t,
            start[1] + dy * t,
            start[2] + (end[2] - start[2]) * t,
        ];
        waypoints.push(CameraWaypoint {
            position: [subject[0] + perp_x, subject[1] + perp_y, subject[2] + 1.5],
            focus: subject,
            fov: 55.0,
            dwell: 0.3,
        });
    }
    waypoints
}

pub fn generate_dutch_waypoints(
    subject: [f32; 3],
    distance: f32,
    steps: u32,
) -> Vec<CameraWaypoint> {
    let cam_pos = [
        subject[0] - distance,
        subject[1],
        subject[2] + distance * 0.4,
    ];
    let mut waypoints = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let roll_offset = 0.8 * (t * std::f32::consts::PI).sin();
        waypoints.push(CameraWaypoint {
            position: [cam_pos[0], cam_pos[1] + roll_offset, cam_pos[2]],
            focus: subject,
            fov: 45.0,
            dwell: 0.6,
        });
    }
    waypoints
}

pub fn generate_push_in_waypoints(subject: [f32; 3], steps: u32) -> Vec<CameraWaypoint> {
    let mut waypoints = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let dist = 20.0 - 15.0 * t;
        let fov = 70.0 - 30.0 * t;
        waypoints.push(CameraWaypoint {
            position: [subject[0] - dist, subject[1], subject[2] + 2.0],
            focus: subject,
            fov,
            dwell: 0.4,
        });
    }
    waypoints
}

pub fn generate_shot_waypoints(shot_type: &str, subject: [f32; 3]) -> Vec<CameraWaypoint> {
    match shot_type {
        "orbit" => generate_orbit_waypoints(subject, 12.0, 5.0, 24),
        "dolly" => {
            let start = [subject[0] - 15.0, subject[1], subject[2] + 2.0];
            let end = [subject[0] + 15.0, subject[1], subject[2] + 2.0];
            generate_dolly_waypoints(start, end, subject, 20)
        }
        "crane" => {
            let base = [subject[0] - 8.0, subject[1], subject[2]];
            generate_crane_waypoints(base, 15.0, subject, 20)
        }
        "flyby" => generate_flyby_waypoints(subject, 16),
        "reveal" => generate_reveal_waypoints(subject, 20),
        "tracking" => {
            let start = [subject[0] - 10.0, subject[1], subject[2]];
            let end = [subject[0] + 10.0, subject[1], subject[2]];
            generate_tracking_waypoints(start, end, 5.0, 20)
        }
        "dutch" => generate_dutch_waypoints(subject, 10.0, 16),
        "push_in" => generate_push_in_waypoints(subject, 20),
        _ => generate_orbit_waypoints(subject, 12.0, 5.0, 24),
    }
}

pub fn generate_lighting_preset(
    preset: &str,
    subject_pos: [f32; 3],
    distance: f32,
) -> Vec<CinemaLight> {
    match preset {
        "rembrandt" => vec![
            CinemaLight {
                name: "Key Light".into(),
                position: [
                    subject_pos[0] - distance * 0.7,
                    subject_pos[1] + distance * 0.7,
                    subject_pos[2] + distance * 0.5,
                ],
                color: [1.0, 0.95, 0.85],
                intensity: 0.9,
                radius: distance * 2.0,
                falloff: 0.5,
            },
            CinemaLight {
                name: "Fill Light".into(),
                position: [
                    subject_pos[0] + distance * 0.5,
                    subject_pos[1] + distance * 0.5,
                    subject_pos[2] + distance * 0.2,
                ],
                color: [0.8, 0.85, 1.0],
                intensity: 0.4,
                radius: distance * 2.0,
                falloff: 0.7,
            },
            CinemaLight {
                name: "Rim Light".into(),
                position: [
                    subject_pos[0],
                    subject_pos[1] - distance * 0.8,
                    subject_pos[2] + distance * 0.6,
                ],
                color: [1.0, 1.0, 1.0],
                intensity: 0.6,
                radius: distance * 1.5,
                falloff: 0.3,
            },
        ],
        "butterfly" => vec![
            CinemaLight {
                name: "Key Light".into(),
                position: [
                    subject_pos[0],
                    subject_pos[1] + distance * 0.3,
                    subject_pos[2] + distance,
                ],
                color: [1.0, 1.0, 0.95],
                intensity: 1.0,
                radius: distance * 2.0,
                falloff: 0.4,
            },
            CinemaLight {
                name: "Fill Light".into(),
                position: [
                    subject_pos[0],
                    subject_pos[1] + distance * 0.3,
                    subject_pos[2] - distance * 0.3,
                ],
                color: [0.9, 0.9, 1.0],
                intensity: 0.3,
                radius: distance * 1.5,
                falloff: 0.8,
            },
        ],
        "split" => vec![
            CinemaLight {
                name: "Key Light".into(),
                position: [
                    subject_pos[0] - distance,
                    subject_pos[1],
                    subject_pos[2] + distance * 0.3,
                ],
                color: [1.0, 1.0, 1.0],
                intensity: 1.0,
                radius: distance * 2.0,
                falloff: 0.3,
            },
            CinemaLight {
                name: "Edge Light".into(),
                position: [
                    subject_pos[0] + distance,
                    subject_pos[1],
                    subject_pos[2] + distance * 0.3,
                ],
                color: [0.7, 0.75, 0.8],
                intensity: 0.3,
                radius: distance * 1.5,
                falloff: 0.6,
            },
        ],
        "rim" => vec![
            CinemaLight {
                name: "Rim Left".into(),
                position: [
                    subject_pos[0] - distance * 0.5,
                    subject_pos[1] - distance * 0.8,
                    subject_pos[2] + distance * 0.4,
                ],
                color: [1.0, 1.0, 1.0],
                intensity: 0.8,
                radius: distance * 2.0,
                falloff: 0.4,
            },
            CinemaLight {
                name: "Rim Right".into(),
                position: [
                    subject_pos[0] + distance * 0.5,
                    subject_pos[1] - distance * 0.8,
                    subject_pos[2] + distance * 0.4,
                ],
                color: [1.0, 1.0, 1.0],
                intensity: 0.8,
                radius: distance * 2.0,
                falloff: 0.4,
            },
        ],
        "studio" => vec![
            CinemaLight {
                name: "Key Light".into(),
                position: [
                    subject_pos[0] - distance * 0.7,
                    subject_pos[1] + distance * 0.5,
                    subject_pos[2] + distance * 0.5,
                ],
                color: [1.0, 0.98, 0.95],
                intensity: 0.85,
                radius: distance * 2.0,
                falloff: 0.5,
            },
            CinemaLight {
                name: "Fill Light".into(),
                position: [
                    subject_pos[0] + distance * 0.6,
                    subject_pos[1] + distance * 0.4,
                    subject_pos[2] + distance * 0.3,
                ],
                color: [0.9, 0.92, 1.0],
                intensity: 0.5,
                radius: distance * 2.0,
                falloff: 0.6,
            },
            CinemaLight {
                name: "Rim Light".into(),
                position: [
                    subject_pos[0],
                    subject_pos[1] - distance * 0.7,
                    subject_pos[2] + distance * 0.5,
                ],
                color: [1.0, 1.0, 1.0],
                intensity: 0.4,
                radius: distance * 1.5,
                falloff: 0.4,
            },
        ],
        "golden_hour" => vec![
            CinemaLight {
                name: "Sun Light".into(),
                position: [
                    subject_pos[0] - distance,
                    subject_pos[1],
                    subject_pos[2] + distance * 0.2,
                ],
                color: [1.0, 0.7, 0.3],
                intensity: 0.9,
                radius: distance * 2.5,
                falloff: 0.5,
            },
            CinemaLight {
                name: "Sky Fill".into(),
                position: [subject_pos[0], subject_pos[1], subject_pos[2] + distance],
                color: [0.5, 0.6, 0.9],
                intensity: 0.3,
                radius: distance * 3.0,
                falloff: 0.8,
            },
            CinemaLight {
                name: "Warm Backlight".into(),
                position: [
                    subject_pos[0] + distance * 0.8,
                    subject_pos[1] - distance * 0.5,
                    subject_pos[2] + distance * 0.3,
                ],
                color: [1.0, 0.75, 0.4],
                intensity: 0.5,
                radius: distance * 2.0,
                falloff: 0.4,
            },
        ],
        "noir" => vec![CinemaLight {
            name: "Hard Side Light".into(),
            position: [
                subject_pos[0] - distance,
                subject_pos[1],
                subject_pos[2] + distance * 0.3,
            ],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            radius: distance * 1.5,
            falloff: 0.2,
        }],
        _ => generate_lighting_preset("studio", subject_pos, distance),
    }
}

pub fn default_lighting_for_shot(shot_type: &str) -> &'static str {
    match shot_type {
        "orbit" | "dolly" | "tracking" => "studio",
        "crane" | "reveal" => "golden_hour",
        "flyby" => "rim",
        "dutch" | "push_in" => "noir",
        _ => "studio",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_waypoints() {
        let wps = generate_orbit_waypoints([128.0, 128.0, 25.0], 10.0, 5.0, 12);
        assert_eq!(wps.len(), 12);
        for wp in &wps {
            assert_eq!(wp.focus, [128.0, 128.0, 25.0]);
            assert!((wp.position[2] - 30.0).abs() < 0.01);
            let dx = wp.position[0] - 128.0;
            let dy = wp.position[1] - 128.0;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!((dist - 10.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_dolly_waypoints() {
        let wps = generate_dolly_waypoints(
            [118.0, 128.0, 27.0],
            [138.0, 128.0, 27.0],
            [128.0, 128.0, 25.0],
            10,
        );
        assert_eq!(wps.len(), 11);
        assert!((wps[0].position[0] - 118.0).abs() < 0.01);
        assert!((wps[10].position[0] - 138.0).abs() < 0.01);
    }

    #[test]
    fn test_crane_waypoints() {
        let wps = generate_crane_waypoints([120.0, 128.0, 25.0], 15.0, [128.0, 128.0, 25.0], 10);
        assert_eq!(wps.len(), 11);
        assert!(wps.last().unwrap().position[2] > wps[0].position[2]);
    }

    #[test]
    fn test_flyby_waypoints() {
        let wps = generate_flyby_waypoints([128.0, 128.0, 25.0], 8);
        assert_eq!(wps.len(), 9);
    }

    #[test]
    fn test_reveal_waypoints() {
        let wps = generate_reveal_waypoints([128.0, 128.0, 25.0], 10);
        assert_eq!(wps.len(), 11);
        assert!(wps[0].fov < wps.last().unwrap().fov);
    }

    #[test]
    fn test_tracking_waypoints() {
        let wps = generate_tracking_waypoints([118.0, 128.0, 25.0], [138.0, 128.0, 25.0], 5.0, 10);
        assert_eq!(wps.len(), 11);
    }

    #[test]
    fn test_dutch_waypoints() {
        let wps = generate_dutch_waypoints([128.0, 128.0, 25.0], 10.0, 8);
        assert_eq!(wps.len(), 9);
    }

    #[test]
    fn test_push_in_waypoints() {
        let wps = generate_push_in_waypoints([128.0, 128.0, 25.0], 10);
        assert_eq!(wps.len(), 11);
        assert!(wps[0].fov > wps.last().unwrap().fov);
    }

    #[test]
    fn test_generate_shot_all_types() {
        for shot in &[
            "orbit", "dolly", "crane", "flyby", "reveal", "tracking", "dutch", "push_in",
        ] {
            let wps = generate_shot_waypoints(shot, [128.0, 128.0, 25.0]);
            assert!(
                !wps.is_empty(),
                "Shot type '{}' should generate waypoints",
                shot
            );
        }
    }

    #[test]
    fn test_lighting_presets_all() {
        for preset in &[
            "rembrandt",
            "butterfly",
            "split",
            "rim",
            "studio",
            "golden_hour",
            "noir",
        ] {
            let lights = generate_lighting_preset(preset, [128.0, 128.0, 25.0], 5.0);
            assert!(
                !lights.is_empty(),
                "Preset '{}' should generate lights",
                preset
            );
            for light in &lights {
                assert!(light.intensity > 0.0 && light.intensity <= 1.0);
                assert!(light.radius > 0.0);
            }
        }
    }

    #[test]
    fn test_rembrandt_is_three_point() {
        let lights = generate_lighting_preset("rembrandt", [128.0, 128.0, 25.0], 5.0);
        assert_eq!(lights.len(), 3);
        assert_eq!(lights[0].name, "Key Light");
        assert_eq!(lights[1].name, "Fill Light");
        assert_eq!(lights[2].name, "Rim Light");
        assert!(lights[0].intensity > lights[1].intensity);
    }

    #[test]
    fn test_noir_single_light() {
        let lights = generate_lighting_preset("noir", [128.0, 128.0, 25.0], 5.0);
        assert_eq!(lights.len(), 1);
    }

    #[test]
    fn test_default_lighting_for_shots() {
        assert_eq!(default_lighting_for_shot("orbit"), "studio");
        assert_eq!(default_lighting_for_shot("crane"), "golden_hour");
        assert_eq!(default_lighting_for_shot("dutch"), "noir");
    }
}
