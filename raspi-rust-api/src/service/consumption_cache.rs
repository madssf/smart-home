use std::collections::VecDeque;

use log::error;
use tokio::sync::mpsc::Sender;

use crate::domain::LiveConsumption;
use crate::service::notifications::NotificationMessage;

// 24 per minute (2.5 sec intervals)
const MAX_CACHE_SIZE: i32 = 24 * 15;

#[derive(Debug)]
pub struct ConsumptionCache {
    consumption: VecDeque<LiveConsumption>,
    notification_sender: Sender<NotificationMessage>,
}

impl ConsumptionCache {
    pub fn new(notification_sender: Sender<NotificationMessage>) -> Self {
        Self {
            consumption: VecDeque::with_capacity(MAX_CACHE_SIZE as usize),
            notification_sender,
        }
    }

    pub async fn add(&mut self, value: LiveConsumption) {
        if self.consumption.len() as i32 == MAX_CACHE_SIZE {
            self.consumption = self
                .consumption
                .range(0..(MAX_CACHE_SIZE - 1) as usize)
                .copied()
                .collect()
        }
        self.consumption.push_front(value);
        let sent = self
            .notification_sender
            .send(NotificationMessage::Consumption {
                watt_usage: value.power,
            })
            .await;
        if let Err(send_error) = sent {
            error!("NotificationMessage SendError: {}", send_error)
        }
    }

    pub fn get_latest(&self, num: i32) -> Vec<&LiveConsumption> {
        if self.consumption.len() < num as usize {
            return self.consumption.iter().collect();
        }
        self.consumption.range(0..num as usize).collect()
    }

    pub fn get_all(&self) -> Vec<&LiveConsumption> {
        self.consumption.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::domain::LiveConsumption;
    use crate::service::consumption_cache::{ConsumptionCache, MAX_CACHE_SIZE};
    use crate::service::notifications::NotificationMessage;

    fn consumption_cache() -> ConsumptionCache {
        let (tx, _) = tokio::sync::mpsc::channel::<NotificationMessage>(1);
        ConsumptionCache::new(tx)
    }

    #[tokio::test]
    async fn should_discard_old_values_when_max_hit() {
        let mut cache = consumption_cache();
        for i in 0..=1000 {
            cache
                .add(LiveConsumption {
                    timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                    power: i,
                })
                .await;
        }
        assert_eq!(cache.consumption.len() as i32, MAX_CACHE_SIZE)
    }

    #[tokio::test]
    async fn should_return_latest_value() {
        let mut cache = consumption_cache();
        for i in 0..=1000 {
            cache
                .add(LiveConsumption {
                    timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                    power: i,
                })
                .await;
        }
        assert_eq!(
            cache.get_latest(1).get(0),
            Some(&&LiveConsumption {
                timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + 1000 * 5, 0),
                power: 1000,
            })
        )
    }

    #[tokio::test]
    async fn should_allow_getting_more_than_exists() {
        let mut cache = consumption_cache();
        for i in 0..=2 {
            cache
                .add(LiveConsumption {
                    timestamp: NaiveDateTime::from_timestamp(1_000_000_000 + i * 5, 0),
                    power: i,
                })
                .await;
        }
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
