use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env};

use crate::{
    error::Error,
    storage,
    types::{emit_completed, emit_created, emit_pickup, emit_refunded, Delivery, Status},
};

/// Compute SHA-256 of `data` and return 32-byte result.
fn sha256(env: &Env, data: &Bytes) -> BytesN<32> {
    env.crypto().sha256(data).into()
}

/// Transfer `amount` of `token` from `from` to `to`.
fn transfer(env: &Env, token: &Address, from: &Address, to: &Address, amount: i128) {
    token::Client::new(env, token).transfer(from, to, &amount);
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Create a new delivery and lock `amount` tokens in the contract.
    ///
    /// Returns the numeric delivery id.
    #[allow(clippy::too_many_arguments)]
    pub fn create_delivery(
        env: Env,
        sender: Address,
        courier: Address,
        token: Address,
        amount: i128,
        secret_hash: BytesN<32>,
        pickup_bps: u32,
        expiry_ledger: u32,
    ) -> Result<u64, Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if pickup_bps > 5_000 {
            return Err(Error::InvalidPickupBps);
        }
        if expiry_ledger <= env.ledger().sequence() {
            return Err(Error::InvalidExpiry);
        }

        sender.require_auth();

        transfer(
            &env,
            &token,
            &sender,
            &env.current_contract_address(),
            amount,
        );

        let id = storage::next_id(&env);
        let delivery = Delivery {
            sender: sender.clone(),
            courier: courier.clone(),
            token,
            amount,
            secret_hash,
            pickup_bps,
            expiry_ledger,
            status: Status::Created,
            released: 0,
        };
        storage::save_delivery(&env, id, &delivery);

        emit_created(&env, id, sender, courier, amount);
        Ok(id)
    }

    /// Called by the courier once they physically collect the parcel.
    ///
    /// Releases `pickup_bps / 10_000 * amount` tokens to the courier immediately.
    /// Rejected if the delivery is already expired.
    pub fn confirm_pickup(env: Env, id: u64) -> Result<(), Error> {
        let mut delivery = storage::load_delivery(&env, id).ok_or(Error::NotFound)?;

        delivery.courier.require_auth();

        if delivery.status != Status::Created {
            return Err(Error::InvalidState);
        }

        // Reject pickup after the delivery window has closed.
        if env.ledger().sequence() > delivery.expiry_ledger {
            return Err(Error::InvalidState);
        }

        let pickup_amount = (delivery.amount as u128)
            .checked_mul(delivery.pickup_bps as u128)
            .unwrap_or(0)
            / 10_000;
        let pickup_amount = pickup_amount as i128;

        if pickup_amount > 0 {
            transfer(
                &env,
                &delivery.token,
                &env.current_contract_address(),
                &delivery.courier,
                pickup_amount,
            );
        }

        delivery.released += pickup_amount;
        delivery.status = Status::PickedUp;
        storage::save_delivery(&env, id, &delivery);

        emit_pickup(&env, id, pickup_amount);
        Ok(())
    }

    /// Called by the courier with the plaintext secret after hand-off.
    ///
    /// Verifies SHA-256(secret) == secret_hash, then releases the remaining balance.
    pub fn complete_delivery(env: Env, id: u64, secret: Bytes) -> Result<(), Error> {
        let mut delivery = storage::load_delivery(&env, id).ok_or(Error::NotFound)?;

        delivery.courier.require_auth();

        if delivery.status != Status::PickedUp {
            return Err(Error::InvalidState);
        }

        if env.ledger().sequence() > delivery.expiry_ledger {
            return Err(Error::InvalidState);
        }

        let hash: BytesN<32> = sha256(&env, &secret);
        if hash != delivery.secret_hash {
            return Err(Error::WrongSecret);
        }

        let remaining = delivery.amount - delivery.released;
        if remaining > 0 {
            transfer(
                &env,
                &delivery.token,
                &env.current_contract_address(),
                &delivery.courier,
                remaining,
            );
        }

        delivery.released += remaining;
        delivery.status = Status::Completed;
        storage::save_delivery(&env, id, &delivery);

        emit_completed(&env, id, delivery.released);
        Ok(())
    }

    /// Called by the sender after `expiry_ledger` to reclaim unreleased funds.
    pub fn refund(env: Env, id: u64) -> Result<(), Error> {
        let mut delivery = storage::load_delivery(&env, id).ok_or(Error::NotFound)?;

        delivery.sender.require_auth();

        if env.ledger().sequence() <= delivery.expiry_ledger {
            return Err(Error::NotExpired);
        }

        if delivery.status == Status::Completed || delivery.status == Status::Refunded {
            return Err(Error::InvalidState);
        }

        let unreleased = delivery.amount - delivery.released;
        if unreleased > 0 {
            transfer(
                &env,
                &delivery.token,
                &env.current_contract_address(),
                &delivery.sender,
                unreleased,
            );
        }

        delivery.status = Status::Refunded;
        storage::save_delivery(&env, id, &delivery);

        emit_refunded(&env, id, unreleased);
        Ok(())
    }

    /// Read the delivery record.
    pub fn get_delivery(env: Env, id: u64) -> Result<Delivery, Error> {
        storage::load_delivery(&env, id).ok_or(Error::NotFound)
    }
}
