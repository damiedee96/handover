#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use crate::{
    contract::{EscrowContract, EscrowContractClient},
    error::Error,
    types::Status,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct Setup {
    env: Env,
    contract: Address,
    token: Address,
    sender: Address,
    courier: Address,
    recipient: Address,
}

impl Setup {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let sender = Address::generate(&env);
        let courier = Address::generate(&env);
        let recipient = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);

        let contract = env.register(EscrowContract, ());
        Setup {
            env,
            contract,
            token,
            sender,
            courier,
            recipient,
        }
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
fn create_default(s: &Setup, pickup_bps: u32) -> u64 {
    let expiry = s.expiry();
    s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &100_000,
        &pickup_bps,
        &expiry,
    )
}

/// Helper: return count of events from the last invocation.
fn event_count(env: &Env) -> usize {
    env.events().all().events().len()
}

// ---------------------------------------------------------------------------
// 1. Happy-path: full flow (20 % pickup)
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_full_flow() {
    let s = Setup::new();
    let id = create_default(&s, 2000); // 20 %

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 20_000);

    s.client().confirm_receipt(&id);
    assert_eq!(s.token_balance(&s.courier), 100_000);
    assert_eq!(s.token_balance(&s.sender), 900_000);
}

// ---------------------------------------------------------------------------
// 2. Zero pickup_bps – all released on receipt
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_zero_pickup() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 0);

    s.client().confirm_receipt(&id);
    assert_eq!(s.token_balance(&s.courier), 100_000);
}

// ---------------------------------------------------------------------------
// 3. Max pickup_bps (5000 = 50 %)
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_max_pickup_bps() {
    let s = Setup::new();
    let id = create_default(&s, 5000);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 50_000);

    s.client().confirm_receipt(&id);
    assert_eq!(s.token_balance(&s.courier), 100_000);
}

// ---------------------------------------------------------------------------
// 4. Rounding – floor division
// ---------------------------------------------------------------------------
#[test]
fn test_rounding() {
    let s = Setup::new();
    // amount=10_001, pickup_bps=1000 → floor(10001 * 1000 / 10000) = 1000
    let id = s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &10_001,
        &1000,
        &s.expiry(),
    );

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 1_000);

    s.client().confirm_receipt(&id);
    assert_eq!(s.token_balance(&s.courier), 10_001);
}

// ---------------------------------------------------------------------------
// 5. Double-receipt rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_receipt() {
    let s = Setup::new();
    let id = create_default(&s, 500);

    s.client().confirm_pickup(&id);
    s.client().confirm_receipt(&id);

    let result = s.client().try_confirm_receipt(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 6. Double-pickup rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_pickup() {
    let s = Setup::new();
    let id = create_default(&s, 500);
    s.client().confirm_pickup(&id);

    let result = s.client().try_confirm_pickup(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 7. Early refund rejected
// ---------------------------------------------------------------------------
#[test]
fn test_early_refund() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::NotExpired)));
}

// ---------------------------------------------------------------------------
// 8. Refund after expiry returns full balance (no pickup yet)
// ---------------------------------------------------------------------------
#[test]
fn test_refund_after_expiry() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    s.advance(101);
    s.client().refund(&id);
    assert_eq!(s.token_balance(&s.sender), 1_000_000);
    assert_eq!(s.token_balance(&s.courier), 0);
}

// ---------------------------------------------------------------------------
// 9. Refund after partial pickup returns remainder only
// ---------------------------------------------------------------------------
#[test]
fn test_refund_after_pickup() {
    let s = Setup::new();
    let id = create_default(&s, 3000); // 30 %

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 30_000);

    s.advance(101);
    s.client().refund(&id);
    assert_eq!(s.token_balance(&s.sender), 970_000);
    assert_eq!(s.token_balance(&s.courier), 30_000);
}

// ---------------------------------------------------------------------------
// 10. Double-refund rejected
// ---------------------------------------------------------------------------
#[test]
fn test_double_refund() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    s.advance(101);
    s.client().refund(&id);

    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 11. Refund on completed delivery rejected
// ---------------------------------------------------------------------------
#[test]
fn test_refund_on_completed() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    s.client().confirm_pickup(&id);
    s.client().confirm_receipt(&id);

    s.advance(101);
    let result = s.client().try_refund(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 12. confirm_receipt before pickup rejected
// ---------------------------------------------------------------------------
#[test]
fn test_receipt_before_pickup() {
    let s = Setup::new();
    let id = create_default(&s, 0);

    let result = s.client().try_confirm_receipt(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 13. confirm_receipt after expiry rejected
// ---------------------------------------------------------------------------
#[test]
fn test_receipt_after_expiry() {
    let s = Setup::new();
    let id = create_default(&s, 1000);

    s.client().confirm_pickup(&id);
    s.advance(101); // past expiry

    let result = s.client().try_confirm_receipt(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 14. confirm_pickup after expiry rejected
// ---------------------------------------------------------------------------
#[test]
fn test_pickup_after_expiry_rejected() {
    let s = Setup::new();
    let id = create_default(&s, 1000);

    s.advance(101);

    let result = s.client().try_confirm_pickup(&id);
    assert_eq!(result, Err(Ok(Error::InvalidState)));
}

// ---------------------------------------------------------------------------
// 15. Invalid pickup_bps (> 5000) rejected
// ---------------------------------------------------------------------------
#[test]
fn test_invalid_pickup_bps() {
    let s = Setup::new();
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &100_000,
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
    let current = s.env.ledger().get().sequence_number;
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &100_000,
        &0,
        &current,
    );
    assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
}

// ---------------------------------------------------------------------------
// 17. Zero amount rejected
// ---------------------------------------------------------------------------
#[test]
fn test_zero_amount() {
    let s = Setup::new();
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &0,
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
    let result = s.client().try_create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &-1,
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
    let id = create_default(&s, 1000);

    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::Created);
    assert_eq!(d.released, 0);

    s.client().confirm_pickup(&id);
    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::PickedUp);
    assert_eq!(d.released, 10_000);

    s.client().confirm_receipt(&id);
    let d = s.client().get_delivery(&id);
    assert_eq!(d.status, Status::Completed);
    assert_eq!(d.released, 100_000);
}

// ---------------------------------------------------------------------------
// 21. Auth failure: courier auth required for confirm_pickup
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_confirm_pickup() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);

    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 100;
    let id = client.create_delivery(
        &sender, &courier, &recipient, &token, &100_000, &500, &expiry,
    );

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
    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 100;
    let result = client.try_create_delivery(
        &sender, &courier, &recipient, &token, &100_000, &500, &expiry,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 23. Auth failure: sender auth required for refund
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 50;
    let id = client.create_delivery(&sender, &courier, &recipient, &token, &100_000, &0, &expiry);
    env.ledger().set(LedgerInfo {
        sequence_number: expiry + 1,
        ..env.ledger().get()
    });

    env.mock_auths(&[]);
    let result = client.try_refund(&id);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 24. Auth failure: recipient auth required for confirm_receipt
// ---------------------------------------------------------------------------
#[test]
fn test_auth_failure_confirm_receipt() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 100;
    let id = client.create_delivery(
        &sender, &courier, &recipient, &token, &100_000, &500, &expiry,
    );
    client.confirm_pickup(&id);

    // Clear all auths – recipient's auth is now absent.
    env.mock_auths(&[]);
    let result = client.try_confirm_receipt(&id);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 25. Auth failure: sender cannot call confirm_receipt (wrong role)
// ---------------------------------------------------------------------------
#[test]
fn test_sender_cannot_confirm_receipt() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 100;
    let id = client.create_delivery(
        &sender, &courier, &recipient, &token, &100_000, &500, &expiry,
    );
    client.confirm_pickup(&id);

    // Only sender's auth is present; contract requires recipient's auth.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &sender,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract,
            fn_name: "confirm_receipt",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);
    let result = client.try_confirm_receipt(&id);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 26. Exploit sequence rejected: courier cannot confirm_pickup then
//     immediately call confirm_receipt to steal the escrow
// ---------------------------------------------------------------------------
#[test]
fn test_exploit_courier_cannot_self_complete() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let courier = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&sender, &1_000_000);
    let contract = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract);

    let expiry = env.ledger().get().sequence_number + 100;
    let id = client.create_delivery(
        &sender, &courier, &recipient, &token, &100_000, &2000, &expiry,
    );

    // Courier picks up (gets 20 000).
    client.confirm_pickup(&id);
    assert_eq!(TokenClient::new(&env, &token).balance(&courier), 20_000);

    // Courier attempts to trigger confirm_receipt using only their own auth.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &courier,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract,
            fn_name: "confirm_receipt",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);
    let result = client.try_confirm_receipt(&id);

    // Must be rejected – courier cannot release their own remainder.
    assert!(result.is_err());

    // Courier balance stays at 20 000 – remainder not released.
    assert_eq!(TokenClient::new(&env, &token).balance(&courier), 20_000);
}

// ---------------------------------------------------------------------------
// 27. Multiple independent deliveries
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_deliveries() {
    let s = Setup::new();
    let id_a = create_default(&s, 1000);
    let id_b = create_default(&s, 2000);

    // Complete A.
    s.client().confirm_pickup(&id_a);
    s.client().confirm_receipt(&id_a);

    // B is still Created.
    let d_b = s.client().get_delivery(&id_b);
    assert_eq!(d_b.status, Status::Created);

    // Complete B.
    s.client().confirm_pickup(&id_b);
    s.client().confirm_receipt(&id_b);

    assert_eq!(s.token_balance(&s.courier), 200_000);
}

// ---------------------------------------------------------------------------
// 28. Token balances throughout lifecycle
// ---------------------------------------------------------------------------
#[test]
fn test_token_balances_throughout() {
    let s = Setup::new();
    let id = create_default(&s, 1500); // 15 %

    assert_eq!(s.token_balance(&s.sender), 900_000);
    assert_eq!(s.token_balance(&s.contract), 100_000);

    s.client().confirm_pickup(&id);
    assert_eq!(s.token_balance(&s.courier), 15_000);
    assert_eq!(s.token_balance(&s.contract), 85_000);

    s.client().confirm_receipt(&id);
    assert_eq!(s.token_balance(&s.courier), 100_000);
    assert_eq!(s.token_balance(&s.contract), 0);
}

// ---------------------------------------------------------------------------
// 29. Refund at exactly expiry+1 succeeds
// ---------------------------------------------------------------------------
#[test]
fn test_refund_exactly_at_expiry_plus_one() {
    let s = Setup::new();
    let expiry = s.env.ledger().get().sequence_number + 50;
    let id = s.client().create_delivery(
        &s.sender,
        &s.courier,
        &s.recipient,
        &s.token,
        &100_000,
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

// ---------------------------------------------------------------------------
// 30-33. Events emitted on every state change
// ---------------------------------------------------------------------------

#[test]
fn test_event_created() {
    let s = Setup::new();
    let id = create_default(&s, 1000);
    assert!(
        event_count(&s.env) > 0,
        "expected event after create_delivery"
    );
    let _ = id;
}

#[test]
fn test_event_pickedup() {
    let s = Setup::new();
    let id = create_default(&s, 1000);
    s.client().confirm_pickup(&id);
    assert!(
        event_count(&s.env) > 0,
        "expected event after confirm_pickup"
    );
}

#[test]
fn test_event_receipt() {
    let s = Setup::new();
    let id = create_default(&s, 0);
    s.client().confirm_pickup(&id);
    s.client().confirm_receipt(&id);
    assert!(
        event_count(&s.env) > 0,
        "expected event after confirm_receipt"
    );
}

#[test]
fn test_event_refunded() {
    let s = Setup::new();
    let id = create_default(&s, 0);
    s.advance(101);
    s.client().refund(&id);
    assert!(event_count(&s.env) > 0, "expected event after refund");
}
