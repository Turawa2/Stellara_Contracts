#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_validation_hash_format() {
    let env = Env::default();
    let empty_hash = symbol_short!("");
    let valid_hash = symbol_short!("QmHash");
    
    // Empty hash should fail
    assert!(ValidationModule::validate_hash_format(&empty_hash).is_err());
    
    // Valid hash should pass
    assert!(ValidationModule::validate_hash_format(&valid_hash).is_ok());
}

#[test]
fn test_validation_threshold() {
    // Valid threshold
    assert!(ValidationModule::validate_threshold(2, 3).is_ok());
    
    // Zero threshold should fail
    assert!(ValidationModule::validate_threshold(0, 3).is_err());
    
    // Threshold exceeding approver count should fail
    assert!(ValidationModule::validate_threshold(4, 3).is_err());
}

#[test]
fn test_validation_timelock() {
    // Valid timelock (1 hour)
    assert!(ValidationModule::validate_timelock(3600).is_ok());
    
    // Too short timelock should fail
    assert!(ValidationModule::validate_timelock(1800).is_err());
}

#[test]
fn test_validation_version_compatibility() {
    // Valid version increment
    assert!(ValidationModule::validate_version_compatibility(1, 2).is_ok());
    
    // Same version should fail
    assert!(ValidationModule::validate_version_compatibility(1, 1).is_err());
    
    // Downgrade should fail
    assert!(ValidationModule::validate_version_compatibility(2, 1).is_err());
}

#[test]
fn test_validation_approvers_unique() {
    let env = Env::default();
    
    // Unique approvers
    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    let addr3 = Address::generate(&env);
    
    let mut unique_approvers = soroban_sdk::Vec::new(&env);
    unique_approvers.push_back(addr1.clone());
    unique_approvers.push_back(addr2.clone());
    unique_approvers.push_back(addr3.clone());
    
    assert!(ValidationModule::validate_approvers_unique(&unique_approvers).is_ok());
    
    // Duplicate approvers
    let mut duplicate_approvers = soroban_sdk::Vec::new(&env);
    duplicate_approvers.push_back(addr1.clone());
    duplicate_approvers.push_back(addr2.clone());
    duplicate_approvers.push_back(addr1.clone()); // Duplicate
    
    assert!(ValidationModule::validate_approvers_unique(&duplicate_approvers).is_err());
}

#[test]
fn test_proposal_lifecycle_with_validation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let executor = Address::generate(&env);
    let target_contract = Address::generate(&env);
    
    // Set up roles
    let roles_key = symbol_short!("roles");
    let mut roles = soroban_sdk::Map::new(&env);
    roles.set(admin.clone(), GovernanceRole::Admin);
    roles.set(approver1.clone(), GovernanceRole::Approver);
    roles.set(approver2.clone(), GovernanceRole::Approver);
    roles.set(executor.clone(), GovernanceRole::Executor);
    env.storage().persistent().set(&roles_key, &roles);
    
    // Create proposal with valid parameters
    let mut approvers = soroban_sdk::Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());
    
    let proposal_id = GovernanceManager::propose_upgrade(
        &env,
        admin.clone(),
        symbol_short!("QmHash"),
        target_contract.clone(),
        symbol_short!("Test"),
        2,
        approvers,
        3600,
    ).unwrap();
    
    assert_eq!(proposal_id, 1);
    
    // Verify proposal was created
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Pending);
    assert_eq!(proposal.approvals_count, 0);
}

#[test]
fn test_halt_and_resume_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let target_contract = Address::generate(&env);
    
    // Set up roles
    let roles_key = symbol_short!("roles");
    let mut roles = soroban_sdk::Map::new(&env);
    roles.set(admin.clone(), GovernanceRole::Admin);
    roles.set(approver1.clone(), GovernanceRole::Approver);
    roles.set(approver2.clone(), GovernanceRole::Approver);
    env.storage().persistent().set(&roles_key, &roles);
    
    // Create and approve proposal
    let mut approvers = soroban_sdk::Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());
    
    let proposal_id = GovernanceManager::propose_upgrade(
        &env,
        admin.clone(),
        symbol_short!("QmHash"),
        target_contract.clone(),
        symbol_short!("Test"),
        2,
        approvers,
        3600,
    ).unwrap();
    
    // Advance time past cooling-off period
    env.ledger().with_mut(|li| {
        li.timestamp = 3700;
    });
    
    // Approve proposal
    GovernanceManager::approve_proposal(&env, proposal_id, approver1.clone()).unwrap();
    GovernanceManager::approve_proposal(&env, proposal_id, approver2.clone()).unwrap();
    
    // Verify approved
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Approved);
    
    // Halt the proposal
    HaltModule::halt_proposal(&env, proposal_id, admin.clone(), symbol_short!("Security")).unwrap();
    
    // Verify halted
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Halted);
    assert!(HaltModule::is_halted(&env, proposal_id));
    
    // Resume the proposal
    HaltModule::resume_proposal(&env, proposal_id, admin.clone(), 3600).unwrap();
    
    // Verify resumed to approved status
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Approved);
    assert!(!HaltModule::is_halted(&env, proposal_id));
}

#[test]
fn test_cooling_off_period_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let target_contract = Address::generate(&env);
    
    // Set up roles
    let roles_key = symbol_short!("roles");
    let mut roles = soroban_sdk::Map::new(&env);
    roles.set(admin.clone(), GovernanceRole::Admin);
    roles.set(approver1.clone(), GovernanceRole::Approver);
    env.storage().persistent().set(&roles_key, &roles);
    
    // Create proposal
    let mut approvers = soroban_sdk::Vec::new(&env);
    approvers.push_back(approver1.clone());
    
    let proposal_id = GovernanceManager::propose_upgrade(
        &env,
        admin.clone(),
        symbol_short!("QmHash"),
        target_contract.clone(),
        symbol_short!("Test"),
        1,
        approvers,
        3600,
    ).unwrap();
    
    // Try to approve immediately (should fail due to cooling-off)
    let result = GovernanceManager::approve_proposal(&env, proposal_id, approver1.clone());
    assert!(result.is_err());
    
    // Advance time past cooling-off period
    env.ledger().with_mut(|li| {
        li.timestamp = 3700;
    });
    
    // Now approval should succeed
    let result = GovernanceManager::approve_proposal(&env, proposal_id, approver1.clone());
    assert!(result.is_ok());
}

#[test]
fn test_approval_revocation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let target_contract = Address::generate(&env);
    
    // Set up roles
    let roles_key = symbol_short!("roles");
    let mut roles = soroban_sdk::Map::new(&env);
    roles.set(admin.clone(), GovernanceRole::Admin);
    roles.set(approver1.clone(), GovernanceRole::Approver);
    roles.set(approver2.clone(), GovernanceRole::Approver);
    env.storage().persistent().set(&roles_key, &roles);
    
    // Create proposal
    let mut approvers = soroban_sdk::Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());
    
    let proposal_id = GovernanceManager::propose_upgrade(
        &env,
        admin.clone(),
        symbol_short!("QmHash"),
        target_contract.clone(),
        symbol_short!("Test"),
        2,
        approvers,
        3600,
    ).unwrap();
    
    // Advance time past cooling-off period
    env.ledger().with_mut(|li| {
        li.timestamp = 3700;
    });
    
    // Approve with first approver
    GovernanceManager::approve_proposal(&env, proposal_id, approver1.clone()).unwrap();
    
    // Verify approval count
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.approvals_count, 1);
    
    // Revoke approval
    ApprovalModule::revoke_approval(&env, proposal_id, approver1.clone()).unwrap();
    
    // Verify approval count decreased
    let proposal = GovernanceManager::get_proposal(&env, proposal_id).unwrap();
    assert_eq!(proposal.approvals_count, 0);
}
