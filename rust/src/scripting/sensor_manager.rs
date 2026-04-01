use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

use super::LSLValue;

const MAX_SENSOR_RESULTS: usize = 16;

#[derive(Debug, Clone)]
pub struct SensorParams {
    pub name: String,
    pub key: Uuid,
    pub sensor_type: i32,
    pub range: f32,
    pub arc: f32,
}

struct SensorEntry {
    script_id: Uuid,
    params: SensorParams,
    repeat_interval: Option<f64>,
    next_fire: Instant,
}

#[derive(Debug, Clone)]
pub struct SensorResult {
    pub key: Uuid,
    pub name: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub object_type: i32,
    pub distance: f32,
}

pub struct SensorManager {
    sensors: HashMap<Uuid, SensorEntry>,
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            sensors: HashMap::new(),
        }
    }

    pub fn add_sensor(
        &mut self,
        script_id: Uuid,
        params: SensorParams,
        repeat_interval: Option<f64>,
    ) {
        let now = Instant::now();
        let next_fire = if repeat_interval.is_some() {
            now
        } else {
            now
        };

        self.sensors.insert(script_id, SensorEntry {
            script_id,
            params,
            repeat_interval,
            next_fire,
        });
    }

    pub fn remove_sensor(&mut self, script_id: Uuid) {
        self.sensors.remove(&script_id);
    }

    pub fn check_sensors(&mut self) -> Vec<(Uuid, SensorParams)> {
        let now = Instant::now();
        let mut triggered = Vec::new();
        let mut to_remove = Vec::new();

        for (script_id, entry) in self.sensors.iter_mut() {
            if now >= entry.next_fire {
                triggered.push((*script_id, entry.params.clone()));

                if let Some(interval) = entry.repeat_interval {
                    entry.next_fire = now + std::time::Duration::from_secs_f64(interval);
                } else {
                    to_remove.push(*script_id);
                }
            }
        }

        for script_id in to_remove {
            self.sensors.remove(&script_id);
        }

        triggered
    }

    pub fn remove_all_for_script(&mut self, script_id: Uuid) {
        self.sensors.remove(&script_id);
    }

    pub fn filter_results(results: &[SensorResult], max: usize) -> Vec<SensorResult> {
        let mut sorted = results.to_vec();
        sorted.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(max.min(MAX_SENSOR_RESULTS));
        sorted
    }

    pub fn results_to_detect_params(results: &[SensorResult]) -> Vec<Vec<LSLValue>> {
        results.iter().map(|r| {
            vec![
                LSLValue::Key(r.key),
                LSLValue::String(r.name.clone()),
                LSLValue::Integer(r.object_type),
                LSLValue::Vector(super::LSLVector::new(r.position.0, r.position.1, r.position.2)),
                LSLValue::Rotation(super::LSLRotation {
                    x: r.rotation.0,
                    y: r.rotation.1,
                    z: r.rotation.2,
                    s: r.rotation.3,
                }),
                LSLValue::Vector(super::LSLVector::new(r.velocity.0, r.velocity.1, r.velocity.2)),
            ]
        }).collect()
    }

    pub fn has_sensor(&self, script_id: Uuid) -> bool {
        self.sensors.contains_key(&script_id)
    }

    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_one_shot_sensor() {
        let mut mgr = SensorManager::new();
        let script_id = Uuid::new_v4();

        mgr.add_sensor(script_id, SensorParams {
            name: "".to_string(),
            key: Uuid::nil(),
            sensor_type: 1,
            range: 10.0,
            arc: std::f32::consts::PI,
        }, None);

        assert!(mgr.has_sensor(script_id));

        let triggered = mgr.check_sensors();
        assert_eq!(triggered.len(), 1);

        assert!(!mgr.has_sensor(script_id));
    }

    #[test]
    fn test_add_repeating_sensor() {
        let mut mgr = SensorManager::new();
        let script_id = Uuid::new_v4();

        mgr.add_sensor(script_id, SensorParams {
            name: "".to_string(),
            key: Uuid::nil(),
            sensor_type: 1,
            range: 20.0,
            arc: std::f32::consts::PI,
        }, Some(1.0));

        let triggered = mgr.check_sensors();
        assert_eq!(triggered.len(), 1);
        assert!(mgr.has_sensor(script_id));
    }

    #[test]
    fn test_remove_sensor() {
        let mut mgr = SensorManager::new();
        let script_id = Uuid::new_v4();

        mgr.add_sensor(script_id, SensorParams {
            name: "".to_string(),
            key: Uuid::nil(),
            sensor_type: 1,
            range: 10.0,
            arc: std::f32::consts::PI,
        }, Some(1.0));

        mgr.remove_sensor(script_id);
        assert!(!mgr.has_sensor(script_id));
    }

    #[test]
    fn test_filter_results_by_distance() {
        let results = vec![
            SensorResult {
                key: Uuid::new_v4(),
                name: "Far".to_string(),
                position: (100.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                velocity: (0.0, 0.0, 0.0),
                object_type: 1,
                distance: 100.0,
            },
            SensorResult {
                key: Uuid::new_v4(),
                name: "Near".to_string(),
                position: (5.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                velocity: (0.0, 0.0, 0.0),
                object_type: 1,
                distance: 5.0,
            },
        ];

        let filtered = SensorManager::filter_results(&results, 16);
        assert_eq!(filtered[0].name, "Near");
        assert_eq!(filtered[1].name, "Far");
    }
}
