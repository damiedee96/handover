use soroban_sdk::{contracttype, Env};

use crate::types::Delivery;

/// Storage key variants.
#[contracttype]
pub enum DataKey {
    /// Maps u64 delivery id → Delivery.
    Delivery(u64),
    /// Monotonically increasing counter; next id to allocate.
    NextId,
}

/// Bump the delivery TTL so it stays alive for at least `min_ledgers` beyond now.
const DELIVERY_LIFETIME_THRESHOLD: u32 = 17_280; // ~1 day at 5-s ledgers
const DELIVERY_BUMP_AMOUNT: u32 = 17_280 * 30; // bump to ~30 days

pub fn next_id(env: &Env) -> u64 {
    let key = DataKey::NextId;
    let id: u64 = env.storage().instance().get(&key).unwrap_or(0u64);
    env.storage().instance().set(&key, &(id + 1));
    env.storage()
        .instance()
        .extend_ttl(DELIVERY_LIFETIME_THRESHOLD, DELIVERY_BUMP_AMOUNT);
    id
}

pub fn save_delivery(env: &Env, id: u64, delivery: &Delivery) {
    let key = DataKey::Delivery(id);
    env.storage().persistent().set(&key, delivery);
    env.storage()
        .persistent()
        .extend_ttl(&key, DELIVERY_LIFETIME_THRESHOLD, DELIVERY_BUMP_AMOUNT);
}

pub fn load_delivery(env: &Env, id: u64) -> Option<Delivery> {
    let key = DataKey::Delivery(id);
    env.storage().persistent().get(&key)
}
