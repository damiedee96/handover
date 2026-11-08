use soroban_sdk::{contractevent, contracttype, Address, BytesN, Env};

/// Status of a delivery.
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    /// Waiting for the courier to confirm pickup.
    Created,
    /// Courier confirmed pickup; pickup fraction already released.
    PickedUp,
    /// Delivery completed; remainder released.
    Completed,
    /// Refunded to sender after expiry.
    Refunded,
}

/// On-chain record for a single delivery.
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub struct Delivery {
    /// Account that created and funded the delivery.
    pub sender: Address,
    /// Account that picks up and delivers the parcel.
    pub courier: Address,
    /// SAC / token contract address.
    pub token: Address,
    /// Total amount locked (token's smallest unit).
    pub amount: i128,
    /// SHA-256 of the delivery secret (32 bytes).
    pub secret_hash: BytesN<32>,
    /// Basis-points (0-5000) released on pickup.
    pub pickup_bps: u32,
    /// Ledger sequence after which the sender may refund.
    pub expiry_ledger: u32,
    /// Current lifecycle state.
    pub status: Status,
    /// Amount already released so far.
    pub released: i128,
}

// ─── Contract events ─────────────────────────────────────────────────────────

/// Emitted when a delivery is created and funded.
#[contractevent]
pub struct DeliveryCreated {
    pub id: u64,
    pub sender: Address,
    pub courier: Address,
    pub amount: i128,
}

/// Emitted when the courier confirms pickup.
#[contractevent]
pub struct PickupConfirmed {
    pub id: u64,
    pub pickup_released: i128,
}

/// Emitted when the delivery is completed and the remainder released.
#[contractevent]
pub struct DeliveryCompleted {
    pub id: u64,
    pub total_released: i128,
}

/// Emitted when the sender reclaims funds after expiry.
#[contractevent]
pub struct DeliveryRefunded {
    pub id: u64,
    pub returned: i128,
}

/// Emit a `DeliveryCreated` event.
pub fn emit_created(env: &Env, id: u64, sender: Address, courier: Address, amount: i128) {
    env.events().publish_event(&DeliveryCreated {
        id,
        sender,
        courier,
        amount,
    });
}

/// Emit a `PickupConfirmed` event.
pub fn emit_pickup(env: &Env, id: u64, pickup_released: i128) {
    env.events().publish_event(&PickupConfirmed {
        id,
        pickup_released,
    });
}

/// Emit a `DeliveryCompleted` event.
pub fn emit_completed(env: &Env, id: u64, total_released: i128) {
    env.events()
        .publish_event(&DeliveryCompleted { id, total_released });
}

/// Emit a `DeliveryRefunded` event.
pub fn emit_refunded(env: &Env, id: u64, returned: i128) {
    env.events()
        .publish_event(&DeliveryRefunded { id, returned });
}
