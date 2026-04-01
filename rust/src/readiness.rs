use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug)]
pub struct ReadinessTracker {
    database_connected: AtomicBool,
    migrations_complete: AtomicBool,
    robust_registered: AtomicBool,
    udp_bound: AtomicBool,
    terrain_loaded: AtomicBool,
    scene_loaded: AtomicBool,
    scripts_initialized: AtomicBool,
    login_ready: AtomicBool,
    started_at: Instant,
}

impl ReadinessTracker {
    pub fn new() -> Self {
        Self {
            database_connected: AtomicBool::new(false),
            migrations_complete: AtomicBool::new(false),
            robust_registered: AtomicBool::new(false),
            udp_bound: AtomicBool::new(false),
            terrain_loaded: AtomicBool::new(false),
            scene_loaded: AtomicBool::new(false),
            scripts_initialized: AtomicBool::new(false),
            login_ready: AtomicBool::new(false),
            started_at: Instant::now(),
        }
    }

    pub fn set_database_connected(&self) {
        self.database_connected.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Database connected");
        self.check_all_ready();
    }

    pub fn set_migrations_complete(&self) {
        self.migrations_complete.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Migrations complete");
        self.check_all_ready();
    }

    pub fn set_robust_registered(&self) {
        self.robust_registered.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Region registered with Robust");
        self.check_all_ready();
    }

    pub fn set_udp_bound(&self) {
        self.udp_bound.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] UDP socket bound");
        self.check_all_ready();
    }

    pub fn set_terrain_loaded(&self) {
        self.terrain_loaded.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Terrain loaded");
        self.check_all_ready();
    }

    pub fn set_scene_loaded(&self) {
        self.scene_loaded.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Scene objects loaded");
        self.check_all_ready();
    }

    pub fn set_scripts_initialized(&self) {
        self.scripts_initialized.store(true, Ordering::SeqCst);
        tracing::info!("[READINESS] Script engine initialized");
        self.check_all_ready();
    }

    pub fn is_login_ready(&self) -> bool {
        self.login_ready.load(Ordering::SeqCst)
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    pub fn status_breakdown(&self) -> Vec<(&'static str, bool)> {
        vec![
            ("database_connected", self.database_connected.load(Ordering::SeqCst)),
            ("migrations_complete", self.migrations_complete.load(Ordering::SeqCst)),
            ("robust_registered", self.robust_registered.load(Ordering::SeqCst)),
            ("udp_bound", self.udp_bound.load(Ordering::SeqCst)),
            ("terrain_loaded", self.terrain_loaded.load(Ordering::SeqCst)),
            ("scene_loaded", self.scene_loaded.load(Ordering::SeqCst)),
            ("scripts_initialized", self.scripts_initialized.load(Ordering::SeqCst)),
        ]
    }

    fn check_all_ready(&self) {
        let all_ready = self.database_connected.load(Ordering::SeqCst)
            && self.migrations_complete.load(Ordering::SeqCst)
            && self.robust_registered.load(Ordering::SeqCst)
            && self.udp_bound.load(Ordering::SeqCst)
            && self.terrain_loaded.load(Ordering::SeqCst)
            && self.scene_loaded.load(Ordering::SeqCst)
            && self.scripts_initialized.load(Ordering::SeqCst);

        if all_ready && !self.login_ready.load(Ordering::SeqCst) {
            self.login_ready.store(true, Ordering::SeqCst);
            tracing::info!("[READINESS] ALL SYSTEMS READY — login is now safe (startup took {}s)", self.uptime_secs());
        }
    }
}

pub fn new_shared_tracker() -> Arc<ReadinessTracker> {
    Arc::new(ReadinessTracker::new())
}
