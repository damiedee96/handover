use soroban_sdk::{contract, contractimpl, token, Address, Env};

use crate::{
    error::Error,
    storage,
    types::{emit_created, emit_pickup, emit_receipt, emit_refunded, Delivery, Status},
};

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
    /// `recipient` is the person who will receive the parcel and must call
    /// `confirm_receipt` to release the remainder to the courier.
    ///
    /// Returns the numeric delivery id.
    #[allow(clippy::too_many_arguments)]
    pub fn create_delivery(
        env: Env,
        sender: Address,
        courier: Address,
        recipient: Address,
        token: Address,
        amount: i128,
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
            recipient: recipient.clone(),
            token,
            amount,
            pickup_bps,
            expiry_ledger,
            status: Status::Created,
            released: 0,
        };
        storage::save_delivery(&env, id, &delivery);

        emit_created(&env, id, sender, courier, recipient, amount);
        Ok(id)
    }

    /// Called by the courier once they physically collect the parcel.
    ///
    /// Releases `pickup_bps / 10_000 * amount` tokens to the courier.
    /// Rejected if the delivery has already expired.
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

    /// Called by the RECIPIENT after the parcel is physically in their hands.
    ///
    /// Requires the recipient's authorization.  Releases the remaining balance
    /// to the courier.  Only valid when status is PickedUp and before expiry.
    ///
    /// Security note: this separates the ability to trigger payment (recipient)
    /// from the ability to receive payment (courier).  The courier alone cannot
    /// call this function and therefore cannot steal the escrow by calling
    /// confirm_pickup followed immediately by confirm_receipt.
    pub fn confirm_receipt(env: Env, id: u64) -> Result<(), Error> {
        let mut delivery = storage::load_delivery(&env, id).ok_or(Error::NotFound)?;

        // Only the recipient may confirm receipt.
        delivery.recipient.require_auth();

        if delivery.status != Status::PickedUp {
            return Err(Error::InvalidState);
        }

        if env.ledger().sequence() > delivery.expiry_ledger {
            return Err(Error::InvalidState);
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

        emit_receipt(&env, id, remaining);
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
