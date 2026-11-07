#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Bytes, BytesN, Env,
};

use crate::{
    contract::{EscrowContract, EscrowContractClient},
    error::Error,
    types::Status,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_hash(env: &Env, secret: &[u8]) -> BytesN<32> {
    let h: BytesN<32> = env
        .crypto()
        .sha256(&Bytes::from_slice(env, secret))
        .into();
    h
}

struct Setup {
    env: Env,
    contract: Address,
    token: Address,
    sender: Address,
    courier: Address,
}

impl Setup {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let courier = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);

        let contract = env.register(EscrowContract, ());
        Setup { env, contract, token, sender, courier }
    }

    fn client(&self) -> EscrowContractClient<'_> {
        EscrowContractClient::new(&self.env, &self.contract)
    }

    fn advance(&self, n: u32) {
        let info = self.env.ledger().get();
        self.env.ledger().set(LedgerInfo {
            sequence_number: info.sequence_number + n,
            ..info
        });
    }

    fn token_balance(&self, addr: &Address) -> i128 {
        TokenClient::new(&self.env, &self.token).balance(addr)
    }

    fn expiry(&self) -> u32 {
        self.env.ledger().get().sequence_number + 100
    }
}

/// Helper: create a delivery and return its id.
fn create_default(s: &Setup, secret_hash: BytesN<32>, pickup_bps: u32) -> u64 {
    let expiry = s.expiry();
    s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &100_000,
        &secret_hash,
        &pickup_bps,
        &expiry,
    )
}

// ---------------------------------------------------------------------------
// 1. Happy-path: full flow
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_full_flow() {
    let s = Setup::new();
    let secret = b"correct-horse-battery-staple";
    let hash = make_hash(&s.env, secret);

    let id = create_default(&s, hash, 2000); // 20 %
    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 20_000);

    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(s.token_balance(&s.courier), 100_000);
    assert_eq!(s.token_balance(&s.sender), 900_000);
}

// ---------------------------------------------------------------------------
// 2. Zero pickup_bps
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_zero_pickup() {
    let s = Setup::new();
    let secret = b"zero-pickup";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 0);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 0);

    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(s.token_balance(&s.courier), 100_000);
}

// ---------------------------------------------------------------------------
// 3. Max pickup_bps (5000)
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_max_pickup_bps() {
    let s = Setup::new();
    let secret = b"max-pickup";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 5000);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 50_000);

    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(s.token_balance(&s.courier), 100_000);
}

// ---------------------------------------------------------------------------
// 4. Rounding (floor division)
// ---------------------------------------------------------------------------
#[test]
fn test_rounding() {
    let s = Setup::new();
    let secret = b"rounding";
    let hash = make_hash(&s.env, secret);
    // amount=10_001, pickup_bps=1000 → pickup = floor(10001 * 1000 / 10000) = 1000
    let id = s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &10_001,
        &hash,
        &1000,
        &s.expiry(),
    );

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 1_000);

    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(s.token_balance(&s.courier), 10_001);
}

// ---------------------------------------------------------------------------
// 5. Wrong secret
// ---------------------------------------------------------------------------
#[test]
fn test_wrong_secret() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"real-secret");
    let id = create_default(&s, hash, 1000);
    s.client().confirm_pickup(&id);

    let result = s
        .client()
        .try_complete_delivery(&id, &Bytes::from_slice(&s.env, b"wrong-secret"));
    assert_eq!(result, Err(Ok(Error::WrongSecret)));
}

// ---------------------------------------------------------------------------
// 6. Double-completion rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_completion() {
    let s = Setup::new();
    let secret = b"double";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 500);

    s.client().confirm_pickup(&id);
    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));

    let result = s
        .client()
        .try_complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 7. Double-pickup rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_pickup() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"dp");
    let id = create_default(&s, hash, 500);
    s.client().confirm_pickup(&id);

    let result = s.client().try_confirm_pickup(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 8. Early refund rejected
// ---------------------------------------------------------------------------
#[test]
fn test_early_refund() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"early");
    let id = create_default(&s, hash, 0);

    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::NotExpired)));
}

// ---------------------------------------------------------------------------
// 9. Refund after expiry returns full balance
// ---------------------------------------------------------------------------
#[test]
fn test_refund_after_expiry() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"refund");
    let id = create_default(&s, hash, 0);

    s.advance(101);
    s.client().refund(&id);
    assert_eq!(s.token_balance(&s.sender), 1_000_000);
    assert_eq!(s.token_balance(&s.courier), 0);
}

// ---------------------------------------------------------------------------
// 10. Refund after partial pickup returns remainder only
// ---------------------------------------------------------------------------
#[test]
fn test_refund_after_pickup() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"partial-refund");
    let id = create_default(&s, hash, 3000); // 30 %

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 30_000);

    s.advance(101);
    s.client().refund(&id);
    assert_eq!(s.token_balance(&s.sender), 970_000);
    assert_eq!(s.token_balance(&s.courier), 30_000);
}

// ---------------------------------------------------------------------------
// 11. Double-refund rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_refund() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"dr");
    let id = create_default(&s, hash, 0);

    s.advance(101);
    s.client().refund(&id);

    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 12. Refund on completed delivery rejected
// ---------------------------------------------------------------------------
#[test]
fn test_refund_on_completed() {
    let s = Setup::new();
    let secret = b"roc";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 0);

    s.client().confirm_pickup(&id);
    s.client().complete_delivery(&id, &Bytes::from_slice(&s.env, secret));

    s.advance(101);
    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 13. Complete before pickup rejected
// ---------------------------------------------------------------------------
#[test]
fn test_complete_before_pickup() {
    let s = Setup::new();
    let secret = b"cbp";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 0);

    let result = s
        .client()
        .try_complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 14. Expired delivery cannot be completed
// ---------------------------------------------------------------------------
#[test]
fn test_expired_completion() {
    let s = Setup::new();
    let secret = b"expired";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 1000);

    s.client().confirm_pickup(&id);
    s.advance(101); // past expiry

    let result = s
        .client()
        .try_complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 15. Invalid pickup_bps (> 5000) rejected
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_pickup_bps() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"inv");
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &100_000,
        &hash,
        &5001,
        &s.expiry(),
    );
    assert_eq!(result, Err(Ok(Error::InvalidPickupBps)));
}

// ---------------------------------------------------------------------------
// 16. Invalid expiry (at or before current ledger)
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_expiry() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"exp");
    let current = s.env.ledger().get().sequence_number;
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &100_000,
        &hash,
        &0,
        &current, // == current, not > current
    );
    assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
}

// ---------------------------------------------------------------------------
// 17. Zero amount rejected
// ---------------------------------------------------------------------------
#[test]
fn test_zero_amount() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"zero");
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &0,
        &hash,
        &0,
        &s.expiry(),
    );
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

// ---------------------------------------------------------------------------
// 18. Negative amount rejected
// ---------------------------------------------------------------------------
#[test]
fn test_negative_amount() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"neg");
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &-1,
        &hash,
        &0,
        &s.expiry(),
    );
    assert_eq!(result, Err(Ok(Error::InvalidAmount)));
}

// ---------------------------------------------------------------------------
// 19. get_delivery returns NotFound for unknown id
// ---------------------------------------------------------------------------
#[test]
fn test_get_delivery_not_found() {
    let s = Setup::new();
    let result = s.client().try_get_delivery(&999);
    assert_eq!(result, Err(Ok(Error::NotFound)));
}

// ---------------------------------------------------------------------------
// 20. get_delivery reflects state transitions
// ---------------------------------------------------------------------------
#[test]
fn test_get_delivery_state_transitions() {
    let s = Setup::new();
    let secret = b"state";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 1000);

    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::Created);
    assert_eq!(d.released, 0);

    s.client().confirm_pickup(&id);
    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::PickedUp);
    assert_eq!(d.released, 10_000);

    s.client()
        .complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::Completed);
    assert_eq!(d.released, 100_000);
}

// ---------------------------------------------------------------------------
// 21. Auth failure: courier role required for confirm_pickup
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_confirm_pickup() {
    let env = Env::default();
    // Use full mock for setup.
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);

    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let hash = make_hash(&env, b"auth-pickup");
    let expiry = env.ledger().get().sequence_number + 100;
    let id = client.create_delivery(
        &sender,
        &courier,
        &token,
        &100_000,
        &hash,
        &500,
        &expiry,
    );

    // Clear all mocked auths: require_auth() for the courier will now fail.
    env.mock_auths(&[]);
    let result = client.try_confirm_pickup(&id);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 22. Auth failure: sender auth required for create_delivery
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_create() {
    let env = Env::default();
    // No auths mocked at all.
    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let hash = make_hash(&env, b"auth-create");
    let expiry = env.ledger().get().sequence_number + 100;
    let result = client.try_create_delivery(
        &sender,
        &courier,
        &token,
        &100_000,
        &hash,
        &500,
        &expiry,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 23. Auth failure: sender role required for refund
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let hash = make_hash(&env, b"auth-refund");
    let expiry = env.ledger().get().sequence_number + 50;
    let id = client.create_delivery(
        &sender,
        &courier,
        &token,
        &100_000,
        &hash,
        &0,
        &expiry,
    );

    env.ledger().set(LedgerInfo {
        sequence_number: expiry + 1,
        ..env.ledger().get()
    });

    // Clear all mocked auths: require_auth() for the sender will now fail.
    env.mock_auths(&[]);
    let result = client.try_refund(&id);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 24. Multiple independent deliveries
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_deliveries() {
    let s = Setup::new();
    let secret_a = b"secret-a";
    let secret_b = b"secret-b";
    let hash_a = make_hash(&s.env, secret_a);
    let hash_b = make_hash(&s.env, secret_b);

    let id_a = create_default(&s, hash_a, 1000);
    let id_b = create_default(&s, hash_b, 2000);

    // Complete A.
    s.client().confirm_pickup(&id_a);
    s.client()
        .complete_delivery(&id_a, &Bytes::from_slice(&s.env, secret_a));

    // B is still Created.
    let d_b = s.client().get_delivery(&id_b);
    assert_eq!(d_b.status, Status::Created);

    // Complete B.
    s.client().confirm_pickup(&id_b);
    s.client()
        .complete_delivery(&id_b, &Bytes::from_slice(&s.env, secret_b));

    assert_eq!(s.token_balance(&s.courier), 200_000);
}

// ---------------------------------------------------------------------------
// 25. Token balances throughout lifecycle
// ---------------------------------------------------------------------------
#[test]
fn test_token_balances_throughout() {
    let s = Setup::new();
    let secret = b"balance-check";
    let hash = make_hash(&s.env, secret);
    let id = create_default(&s, hash, 1500); // 15 %

    assert_eq!(s.token_balance(&s.sender), 900_000);
    assert_eq!(s.token_balance(&s.contract), 100_000);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 15_000);
    assert_eq!(s.token_balance(&s.contract), 85_000);

    s.client()
        .complete_delivery(&id, &Bytes::from_slice(&s.env, secret));
    assert_eq!(s.token_balance(&s.courier), 100_000);
    assert_eq!(s.token_balance(&s.contract), 0);
}

// ---------------------------------------------------------------------------
// 26. Empty secret that matches its own hash
// ---------------------------------------------------------------------------
#[test]
fn test_complete_with_empty_secret() {
    let s = Setup::new();
    let empty: &[u8] = b"";
    let hash = make_hash(&s.env, empty);
    let id = create_default(&s, hash, 0);

    s.client().confirm_pickup(&id);
    s.client()
        .complete_delivery(&id, &Bytes::from_slice(&s.env, empty));
    assert_eq!(s.token_balance(&s.courier), 100_000);
}

// ---------------------------------------------------------------------------
// 27. Refund at exactly expiry+1 succeeds
// ---------------------------------------------------------------------------
#[test]
fn test_refund_exactly_at_expiry_plus_one() {
    let s = Setup::new();
    let hash = make_hash(&s.env, b"exact");
    let expiry = s.env.ledger().get().sequence_number + 50;
    let id = s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.token,
        &100_000,
        &hash,
        &0,
        &expiry,
    );

    s.env.ledger().set(LedgerInfo {
        sequence_number: expiry + 1,
        ..s.env.ledger().get()
    });

    s.client().refund(&id);
    assert_eq!(s.token_balance(&s.sender), 1_000_000);
}
