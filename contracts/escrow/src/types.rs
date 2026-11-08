use soroban_sdk::{contractevent, contracttype, Address, Env};

/// Status of a delivery.
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    /// Waiting for the courier to confirm pickup.
    Created,
    /// Courier confirmed pickup; pickup fraction already released.
    PickedUp,
    /// Recipient confirmed receipt; remainder released to courier.
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
    /// Account that receives the parcel and confirms receipt.
    pub recipient: Address,
    /// SAC / token contract address.
    pub token: Address,
    /// Total amount locked (token's smallest unit).
    pub amount: i128,
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
    pub recipient: Address,
    pub amount: i128,
}

/// Emitted when the courier confirms pickup.
#[contractevent]
pub struct PickupConfirmed {
    pub id: u64,
    pub pickup_released: i128,
}

/// Emitted when the recipient confirms receipt and the remainder is released.
#[contractevent]
pub struct ReceiptConfirmed {
    pub id: u64,
    pub remainder_released: i128,
}

/// Emitted when the sender reclaims funds after expiry.
#[contractevent]
pub struct DeliveryRefunded {
    pub id: u64,
    pub returned: i128,
}

/// Emit a `DeliveryCreated` event.
pub fn emit_created(
    env: &Env,
    id: u64,
    sender: Address,
    courier: Address,
    recipient: Address,
    amount: i128,
) {
    env.events().publish_event(&DeliveryCreated {
        id,
        sender,
        courier,
        recipient,
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

/// Emit a `ReceiptConfirmed` event.
pub fn emit_receipt(env: &Env, id: u64, remainder_released: i128) {
    env.events().publish_event(&ReceiptConfirmed {
        id,
        remainder_released,
    });
}

/// Emit a `DeliveryRefunded` event.
pub fn emit_refunded(env: &Env, id: u64, returned: i128) {
    env.events()
        .publish_event(&DeliveryRefunded { id, returned });
}
