#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::*, Address, Env, token};

    fn create_test_env() -> (Env, AcademyVestingContractClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AcademyVestingContract);
        let client = AcademyVestingContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reward_token_id = env.register_stellar_asset_contract(issuer);
        let governance = Address::generate(&env);

        (env, client, admin, reward_token_id, governance)
    }

    #[test]
    fn test_contract_initialization() {
        let (_env, client, admin, reward_token, governance) = create_test_env();

        client.init(&admin, &reward_token, &governance);

        // Verify info is stored correctly
        let (stored_admin, stored_token, stored_gov) = client.get_info();

        assert_eq!(stored_admin, admin);
        assert_eq!(stored_token, reward_token);
        assert_eq!(stored_gov, governance);
    }

    #[test]
    fn test_contract_cannot_be_initialized_twice() {
        let (_env, client, admin, reward_token, governance) = create_test_env();

        // First initialization should succeed
        client.init(&admin, &reward_token, &governance);

        // Second initialization should fail
        let result = client.try_init(&admin, &reward_token, &governance);

        assert!(result.is_err());
    }

    #[test]
    fn test_grant_vesting_schedule() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        // Grant vesting schedule
        let grant_id = client.grant_vesting(
            &admin,
            &beneficiary,
            &1000,
            &100,   // start_time
            &60,    // cliff
            &3600,  // duration
        );

        assert_eq!(grant_id, 1);

        // Retrieve and verify schedule
        let schedule = client.get_vesting(&grant_id);

        assert_eq!(schedule.beneficiary, beneficiary);
        assert_eq!(schedule.amount, 1000);
        assert_eq!(schedule.start_time, 100);
        assert_eq!(schedule.cliff, 60);
        assert_eq!(schedule.duration, 3600);
        assert!(!schedule.claimed);
        assert!(!schedule.revoked);
    }

    #[test]
    fn test_grant_multiple_schedules() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary1 = Address::generate(&env);
        let beneficiary2 = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        // Grant first schedule
        let grant_id1 = client.grant_vesting(&admin, &beneficiary1, &1000, &100, &60, &3600);

        // Grant second schedule
        let grant_id2 = client.grant_vesting(&admin, &beneficiary2, &2000, &200, &120, &7200);

        // IDs should be sequential
        assert_eq!(grant_id1, 1);
        assert_eq!(grant_id2, 2);
    }

    #[test]
    fn test_grant_with_invalid_schedule() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        // Test: negative amount
        let result = client.try_grant_vesting(&admin, &beneficiary, &-1000, &100, &60, &3600);
        assert!(result.is_err());

        // Test: cliff > duration
        let result = client.try_grant_vesting(&admin, &beneficiary, &1000, &100, &5000, &3600);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_admin_cannot_grant() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let non_admin = Address::generate(&env);
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        // Non-admin tries to grant
        let result = client.try_grant_vesting(&non_admin, &beneficiary, &1000, &100, &60, &3600);
        assert!(result.is_err());
    }

    #[test]
    fn test_vesting_calculation_before_start() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 1000u64;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &start_time, &300, &3600);

        // Mock ledger time to before start
        env.ledger().set_timestamp(start_time - 100);

        let vested = client.get_vested_amount(&grant_id);
        assert_eq!(vested, 0);
    }

    #[test]
    fn test_vesting_calculation_before_cliff() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 1000u64;
        let cliff = 300u64;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &start_time, &cliff, &3600);

        // Mock ledger time to after start but before cliff
        env.ledger().set_timestamp(start_time + cliff - 50);

        let vested = client.get_vested_amount(&grant_id);
        assert_eq!(vested, 0);
    }

    #[test]
    fn test_vesting_calculation_after_cliff() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 1000u64;
        let cliff = 300u64;
        let duration = 3600u64;
        let amount = 1000i128;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &amount, &start_time, &cliff, &duration);

        // Mock ledger time to exactly at cliff
        env.ledger().set_timestamp(start_time + cliff);

        let vested = client.get_vested_amount(&grant_id);
        assert_eq!(vested, 0);
    }

    #[test]
    fn test_vesting_calculation_fully_vested() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 1000u64;
        let cliff = 300u64;
        let duration = 3600u64;
        let amount = 1000i128;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &amount, &start_time, &cliff, &duration);

        // Mock ledger time to after full duration
        env.ledger().set_timestamp(start_time + duration + 1000);

        let vested = client.get_vested_amount(&grant_id);
        assert_eq!(vested, amount);
    }

    #[test]
    fn test_vesting_calculation_partial() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 0u64;
        let cliff = 100u64;
        let duration = 1000u64;
        let amount = 1000i128;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &amount, &start_time, &cliff, &duration);

        // Mock ledger time to 50% through vesting (after cliff)
        // Time: cliff + (duration - cliff) / 2 = 100 + 450 = 550
        env.ledger().set_timestamp(start_time + cliff + 450);

        let vested = client.get_vested_amount(&grant_id);
        // Should be exactly 500: (1000 * 450) / 900 = 500
        assert_eq!(vested, 500);
    }

    #[test]
    fn test_claim_not_vested() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 1000u64;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &start_time, &300, &3600);

        // Try to claim before vesting
        env.ledger().set_timestamp(start_time - 100);

        let result = client.try_claim(&grant_id, &beneficiary);
        assert!(result.is_err());
    }

    #[test]
    fn test_claim_flow_and_double_claim_protection() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let start_time = 0u64;
        let cliff = 100u64;
        let duration = 1000u64;
        let amount = 1000i128;

        let grant_id = client.grant_vesting(&admin, &beneficiary, &amount, &start_time, &cliff, &duration);

        // Set time to fully vested
        env.ledger().set_timestamp(start_time + duration + 1);

        // Setup token balance - Case 1: Insufficient balance
        let token_admin = token::StellarAssetClient::new(&env, &reward_token);
        token_admin.mint(&client.address, &100); // Only 100 tokens

        let result = client.try_claim(&grant_id, &beneficiary);
        assert!(result.is_err());

        // Case 2: Successful claim
        token_admin.mint(&client.address, &900); // Now has 1000
        let claimed = client.claim(&grant_id, &beneficiary);
        assert_eq!(claimed, amount);

        // Case 3: Double claim protection
        let result2 = client.try_claim(&grant_id, &beneficiary);
        assert!(result2.is_err());
    }

    #[test]
    fn test_claim_revoked_schedule() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &0, &100, &3600);

        // Allow revocation delay
        env.ledger().set_timestamp(4000);
        client.revoke(&grant_id, &admin, &3600);

        // Try to claim revoked schedule
        let result = client.try_claim(&grant_id, &beneficiary);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_constraints() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);

        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &0, &100, &3600);

        // Case 1: Invalid timelock (< 1 hour)
        let result = client.try_revoke(&grant_id, &admin, &100);
        assert!(result.is_err());

        // Case 2: Not enough time elapsed
        env.ledger().set_timestamp(500);
        let result2 = client.try_revoke(&grant_id, &admin, &3600);
        assert!(result2.is_err());

        // Case 3: Successful revoke
        env.ledger().set_timestamp(4000);
        client.revoke(&grant_id, &admin, &3600);

        // Case 4: Cannot revoke twice
        let result3 = client.try_revoke(&grant_id, &admin, &3600);
        assert!(result3.is_err());
    }

    #[test]
    fn test_claim_wrong_beneficiary() {
        let (env, client, admin, reward_token, governance) = create_test_env();
        let beneficiary = Address::generate(&env);
        let other = Address::generate(&env);

        client.init(&admin, &reward_token, &governance);
        let grant_id = client.grant_vesting(&admin, &beneficiary, &1000, &0, &100, &3600);

        env.ledger().set_timestamp(2000);

        let result = client.try_claim(&grant_id, &other);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_grant() {
        let (_env, client, admin, reward_token, governance) = create_test_env();
        client.init(&admin, &reward_token, &governance);

        let result = client.try_get_vesting(&999);
        assert!(result.is_err());

        let result2 = client.try_get_vested_amount(&999);
        assert!(result2.is_err());
    }
}
