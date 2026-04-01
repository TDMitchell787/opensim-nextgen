use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::SystemTime;
use std::collections::VecDeque;
use dashmap::DashMap;
use parking_lot::Mutex;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpamAction {
    Allow,
    Mute30s,
    Mute5m,
    Kick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpamCategory {
    Chat,
    Sound,
    ViewerEffect,
}

struct AgentSpamState {
    chat_timestamps: Mutex<VecDeque<u64>>,
    chat_recent_texts: Mutex<VecDeque<(u64, u64)>>,
    sound_count: AtomicU32,
    effect_count: AtomicU32,
    second_epoch: AtomicU64,
    muted_until: AtomicU64,
    warning_count: AtomicU32,
}

impl AgentSpamState {
    fn new() -> Self {
        Self {
            chat_timestamps: Mutex::new(VecDeque::with_capacity(64)),
            chat_recent_texts: Mutex::new(VecDeque::with_capacity(16)),
            sound_count: AtomicU32::new(0),
            effect_count: AtomicU32::new(0),
            second_epoch: AtomicU64::new(0),
            muted_until: AtomicU64::new(0),
            warning_count: AtomicU32::new(0),
        }
    }
}

pub struct SpamProtection {
    agents: DashMap<Uuid, AgentSpamState>,
    max_chat_per_second: u32,
    max_chat_per_minute: u32,
    max_sound_per_second: u32,
    max_effect_per_second: u32,
    repeat_threshold: u32,
    repeat_window_secs: u64,
}

impl SpamProtection {
    pub fn new() -> Self {
        Self {
            agents: DashMap::new(),
            max_chat_per_second: 5,
            max_chat_per_minute: std::env::var("OPENSIM_CHAT_RATE_LIMIT")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(60),
            max_sound_per_second: 10,
            max_effect_per_second: 20,
            repeat_threshold: 3,
            repeat_window_secs: 10,
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn text_hash(text: &str) -> u64 {
        let mut h: u64 = 5381;
        for b in text.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        h
    }

    pub fn check_chat(&self, agent_id: Uuid, text: &str) -> SpamAction {
        let now = Self::now_secs();
        let state = self.agents.entry(agent_id).or_insert_with(AgentSpamState::new);

        let muted_until = state.muted_until.load(Ordering::Relaxed);
        if now < muted_until {
            return SpamAction::Mute30s;
        }

        {
            let mut timestamps = state.chat_timestamps.lock();
            timestamps.push_back(now);

            while timestamps.front().map_or(false, |&t| now - t > 60) {
                timestamps.pop_front();
            }

            let in_last_second = timestamps.iter().filter(|&&t| now - t < 1).count();
            if in_last_second > self.max_chat_per_second as usize {
                return self.escalate(&state, agent_id, now, SpamCategory::Chat);
            }

            if timestamps.len() > self.max_chat_per_minute as usize {
                return self.escalate(&state, agent_id, now, SpamCategory::Chat);
            }
        }

        if !text.is_empty() {
            let hash = Self::text_hash(text);
            let mut recent = state.chat_recent_texts.lock();
            recent.push_back((now, hash));
            while recent.front().map_or(false, |&(t, _)| now - t > self.repeat_window_secs) {
                recent.pop_front();
            }
            let repeat_count = recent.iter().filter(|&&(_, h)| h == hash).count();
            if repeat_count >= self.repeat_threshold as usize {
                warn!("[SPAM] Agent {} repeating same text {} times in {}s",
                      agent_id, repeat_count, self.repeat_window_secs);
                return self.escalate(&state, agent_id, now, SpamCategory::Chat);
            }
        }

        SpamAction::Allow
    }

    pub fn check_sound(&self, agent_id: Uuid) -> SpamAction {
        let now = Self::now_secs();
        let state = self.agents.entry(agent_id).or_insert_with(AgentSpamState::new);

        let muted_until = state.muted_until.load(Ordering::Relaxed);
        if now < muted_until {
            return SpamAction::Mute30s;
        }

        let stored = state.second_epoch.load(Ordering::Relaxed);
        if now != stored {
            state.sound_count.store(0, Ordering::Relaxed);
            state.effect_count.store(0, Ordering::Relaxed);
            state.second_epoch.store(now, Ordering::Relaxed);
        }

        let count = state.sound_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count > self.max_sound_per_second {
            return self.escalate(&state, agent_id, now, SpamCategory::Sound);
        }

        SpamAction::Allow
    }

    pub fn check_viewer_effect(&self, agent_id: Uuid) -> SpamAction {
        let now = Self::now_secs();
        let state = self.agents.entry(agent_id).or_insert_with(AgentSpamState::new);

        let muted_until = state.muted_until.load(Ordering::Relaxed);
        if now < muted_until {
            return SpamAction::Mute30s;
        }

        let stored = state.second_epoch.load(Ordering::Relaxed);
        if now != stored {
            state.sound_count.store(0, Ordering::Relaxed);
            state.effect_count.store(0, Ordering::Relaxed);
            state.second_epoch.store(now, Ordering::Relaxed);
        }

        let count = state.effect_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count > self.max_effect_per_second {
            return self.escalate(&state, agent_id, now, SpamCategory::ViewerEffect);
        }

        SpamAction::Allow
    }

    fn escalate(&self, state: &AgentSpamState, agent_id: Uuid, now: u64, category: SpamCategory) -> SpamAction {
        let warnings = state.warning_count.fetch_add(1, Ordering::Relaxed) + 1;

        match warnings {
            1..=2 => {
                state.muted_until.store(now + 30, Ordering::Relaxed);
                warn!("[SPAM] Agent {} muted 30s for {:?} spam (warning {})", agent_id, category, warnings);
                SpamAction::Mute30s
            }
            3..=5 => {
                state.muted_until.store(now + 300, Ordering::Relaxed);
                warn!("[SPAM] Agent {} muted 5min for {:?} spam (warning {})", agent_id, category, warnings);
                SpamAction::Mute5m
            }
            _ => {
                warn!("[SPAM] Agent {} kicked for repeated {:?} spam (warning {})", agent_id, category, warnings);
                SpamAction::Kick
            }
        }
    }

    pub fn is_muted(&self, agent_id: Uuid) -> bool {
        if let Some(state) = self.agents.get(&agent_id) {
            let now = Self::now_secs();
            let muted_until = state.muted_until.load(Ordering::Relaxed);
            now < muted_until
        } else {
            false
        }
    }

    pub fn remove_agent(&self, agent_id: Uuid) {
        self.agents.remove(&agent_id);
    }
}
