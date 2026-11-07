use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// Delivery with this id does not exist.
    NotFound = 1,
    /// pickup_bps exceeds 5000 (50 %).
    InvalidPickupBps = 2,
    /// expiry_ledger is not in the future.
    InvalidExpiry = 3,
    /// Caller is not the expected account.
    Unauthorised = 4,
    /// Delivery is in a state that does not allow this operation.
    InvalidState = 5,
    /// Wrong secret – SHA-256 does not match hash.
    WrongSecret = 6,
    /// Expiry ledger has not passed yet (too early to refund).
    NotExpired = 7,
    /// Amount must be positive.
    InvalidAmount = 8,
}
