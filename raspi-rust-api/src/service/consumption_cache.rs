use std::collections::VecDeque;

use crate::domain::LiveConsumption;

const MAX_CACHE_SIZE: i32 = 100;

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

    pub fn get_latest(&self) -> Option<&LiveConsumption> {
        self.consumption.front()
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
            cache.get_latest(),
            Some(&LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + 1000 * 5, 0),
                power: 1000,
            })
        )
    }
}
