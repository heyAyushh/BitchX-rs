use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloodType {
    Message,
    Ctcp,
    Join,
    Notice,
    Invite,
    Nick,
}

#[derive(Debug)]
struct FloodCounter {
    timestamps: Vec<Instant>,
}

#[derive(Debug)]
pub struct FloodTracker {
    counters: HashMap<(String, FloodType), FloodCounter>,
    max_count: u32,
    window: Duration,
}

impl FloodTracker {
    pub fn new(max_count: u32, window_secs: u64) -> Self {
        Self {
            counters: HashMap::new(),
            max_count,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&mut self, source: &str, flood_type: FloodType) -> bool {
        let key = (source.to_ascii_lowercase(), flood_type);
        let now = Instant::now();

        let counter = self.counters.entry(key).or_insert_with(|| FloodCounter {
            timestamps: Vec::new(),
        });

        counter
            .timestamps
            .retain(|&t| now.duration_since(t) < self.window);
        counter.timestamps.push(now);

        counter.timestamps.len() > self.max_count as usize
    }

    pub fn cleanup(&mut self) {
        let now = Instant::now();
        let window = self.window;
        self.counters.retain(|_, counter| {
            counter
                .timestamps
                .retain(|&t| now.duration_since(t) < window);
            !counter.timestamps.is_empty()
        });
    }

    pub fn reset(&mut self, source: &str) {
        let lower = source.to_ascii_lowercase();
        self.counters.retain(|(s, _), _| *s != lower);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_no_flood_below_threshold() {
        let mut tracker = FloodTracker::new(3, 10);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
    }

    #[test]
    fn test_flood_at_threshold() {
        let mut tracker = FloodTracker::new(3, 10);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(tracker.check("Alice", FloodType::Message)); // 4th = flood
    }

    #[test]
    fn test_different_sources_tracked_separately() {
        let mut tracker = FloodTracker::new(2, 10);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Bob", FloodType::Message));
        assert!(!tracker.check("Bob", FloodType::Message));
        assert!(tracker.check("Alice", FloodType::Message)); // Alice floods
        assert!(tracker.check("Bob", FloodType::Message)); // Bob floods too
    }

    #[test]
    fn test_different_types_tracked_separately() {
        let mut tracker = FloodTracker::new(2, 10);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Ctcp));
        assert!(!tracker.check("Alice", FloodType::Ctcp));
        assert!(tracker.check("Alice", FloodType::Message)); // Message floods
        assert!(tracker.check("Alice", FloodType::Ctcp)); // Ctcp floods
    }

    #[test]
    fn test_window_expiry() {
        let mut tracker = FloodTracker::new(2, 1);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(tracker.check("Alice", FloodType::Message));

        thread::sleep(Duration::from_millis(1100));

        assert!(!tracker.check("Alice", FloodType::Message));
    }

    #[test]
    fn test_cleanup() {
        let mut tracker = FloodTracker::new(5, 1);
        tracker.check("Alice", FloodType::Message);
        tracker.check("Bob", FloodType::Ctcp);
        assert_eq!(tracker.counters.len(), 2);

        thread::sleep(Duration::from_millis(1100));
        tracker.cleanup();
        assert_eq!(tracker.counters.len(), 0);
    }

    #[test]
    fn test_reset() {
        let mut tracker = FloodTracker::new(5, 10);
        tracker.check("Alice", FloodType::Message);
        tracker.check("Alice", FloodType::Ctcp);
        tracker.check("Bob", FloodType::Message);
        assert_eq!(tracker.counters.len(), 3);

        tracker.reset("Alice");
        assert_eq!(tracker.counters.len(), 1);
        assert!(!tracker.check("Alice", FloodType::Message));
    }

    #[test]
    fn test_case_insensitive_source() {
        let mut tracker = FloodTracker::new(2, 10);
        assert!(!tracker.check("Alice", FloodType::Message));
        assert!(!tracker.check("ALICE", FloodType::Message));
        assert!(tracker.check("alice", FloodType::Message));
    }

    #[test]
    fn test_all_flood_types() {
        let types = [
            FloodType::Message,
            FloodType::Ctcp,
            FloodType::Join,
            FloodType::Notice,
            FloodType::Invite,
            FloodType::Nick,
        ];
        let mut tracker = FloodTracker::new(1, 10);
        for ft in &types {
            assert!(!tracker.check("test", *ft));
            assert!(tracker.check("test", *ft));
        }
    }
}
