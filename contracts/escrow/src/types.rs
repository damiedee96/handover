use soroban_sdk::{contracttype, Address, BytesN};

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
