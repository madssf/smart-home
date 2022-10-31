use std::collections::VecDeque;

use crate::domain::LiveConsumption;

const MAX_CACHE_SIZE: i32 = 100;

#[derive(Debug)]
pub struct ConsumptionCache {
    consumption: VecDeque<LiveConsumption>,
}

impl ConsumptionCache {
    pub fn new() -> Self {
        Self {
            consumption: VecDeque::with_capacity(MAX_CACHE_SIZE as usize),
        }
    }

    pub fn add(&mut self, value: LiveConsumption) {
        if self.consumption.len() as i32 == MAX_CACHE_SIZE {
            self.consumption = self
                .consumption
                .range(0..(MAX_CACHE_SIZE - 1) as usize)
                .copied()
                .collect()
        }
        self.consumption.push_front(value)
    }

    pub fn get_latest(&self, num: i32) -> Vec<&LiveConsumption> {
        if self.consumption.len() < num as usize {
            return self.consumption.iter().collect();
        }
        self.consumption.range(0..num as usize).collect()
    }
}

impl Default for ConsumptionCache {
    fn default() -> Self {
        ConsumptionCache::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::domain::LiveConsumption;
    use crate::service::consumption_cache::{ConsumptionCache, MAX_CACHE_SIZE};

    #[test]
    fn should_discard_old_values_when_max_hit() {
        let mut cache = ConsumptionCache::default();
        (0..=1000).for_each(|i| {
            cache.add(LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                power: i,
            })
        });
        assert_eq!(cache.consumption.len() as i32, MAX_CACHE_SIZE)
    }

    #[test]
    fn should_return_latest_value() {
        let mut cache = ConsumptionCache::default();
        (0..=1000).for_each(|i| {
            cache.add(LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                power: i,
            })
        });
        assert_eq!(
            cache.get_latest(1).get(0),
            Some(&&LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + 1000 * 5, 0),
                power: 1000,
            })
        )
    }

    #[test]
    fn should_allow_getting_more_than_exists() {
        let mut cache = ConsumptionCache::default();
        (0..=2).for_each(|i| {
            cache.add(LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                power: i,
            })
        });
        assert_eq!(cache.get_latest(100).get(50), None);
        dbg!(&cache);
        assert_eq!(
            cache.get_latest(3).get(2),
            Some(&&LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000, 0),
                power: 0
            })
        )
    }
}
