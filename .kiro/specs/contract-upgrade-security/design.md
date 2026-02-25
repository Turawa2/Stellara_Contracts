# Design Document

## Overview

This design document specifies security enhancements for the Stellara smart contract upgrade governance system. The existing system provides multi-signature approval and timelock mechanisms, but requires additional safety features to prevent accidental or malicious upgrades. This enhancement adds comprehensive validation checks, simulation metadata storage, emergency halt mechanisms, improved multi-sig workflows with cooling-off periods, and enhanced audit trails.

The design builds upon the existing `GovernanceManager` in the shared library and extends it with new validation, halt, and approval revocation capabilities while maintaining backward compatibility with existing contracts.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Contract Layer (Trading, etc.)              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  propose_upgrade() → Enhanced Validation             │  │
│  │  approve_upgrade() → Cooling-off + Revocation        │  │
│  │  halt_upgrade() → Emergency Stop                     │  │
│  │  execute_upgrade() → Halt Check + Validation         │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Enhanced Governance Manager                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Validation Module                                    │  │
│  │  • Hash format validation                            │  │
│  │  • Contract address validation                       │  │
│  │  │  • Threshold validation                            │  │
│  │  • Timelock minimum enforcement                      │  │
│  │  • Approver uniqueness check                         │  │
│  │  • Version compatibility check                       │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Halt Module                                          │  │
│  │  • Emergency halt function                           │  │
│  │  • Halt status tracking                              │  │
│  │  • Resume with new timelock                          │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Approval Module                                      │  │
│  │  • Cooling-off period enforcement                    │  │
│  │  • Approval timestamp tracking                       │  │
│  │  • Approval revocation                               │  │
│  │  • Time-to-execution queries                         │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Metadata Module                                      │  │
│  │  • Simulation results storage                        │  │
│  │  • Version information                               │  │
│  │  • Compatibility flags                               │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    Event Emission Layer                      │
│  • ValidationFailedEvent                                     │
│  • ProposalHaltedEvent                                       │
│  • ProposalResumedEvent                                      │
│  • ApprovalRevokedEvent                                      │
│  • Enhanced existing events with metadata                    │
└─────────────────────────────────────────────────────────────┘
```

### Component Interaction Flow

```
1. Proposal Creation with Validation:
   Admin → propose_upgrade() → ValidationModule → Store Proposal → Emit Event

2. Approval with Cooling-off:
   Approver → approve_upgrade() → Check Cooling-off → Record Approval → Emit Event

3. Emergency Halt:
   Admin → halt_upgrade() → Update Status to Halted → Emit Event

4. Execution with Safety Checks:
   Executor → execute_upgrade() → Check Halt Status → Check Timelock → Execute
```

## Components and Interfaces

### 1. Enhanced UpgradeProposal Structure

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    // Existing fields
    pub id: u64,
    pub proposer: Address,
    pub new_contract_hash: Symbol,
    pub target_contract: Address,
    pub description: Symbol,
    pub approval_threshold: u32,
    pub approvers: Vec<Address>,
    pub approvals_count: u32,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub execution_time: u64,
    pub executed: bool,
    
    // New fields for security enhancements
    pub cooling_off_period: u64,           // Minimum time before first approval
    pub current_version: u32,              // Current contract version
    pub proposed_version: u32,             // Proposed contract version
    pub simulation_passed: bool,           // Whether simulation tests passed
    pub simulation_metadata: Symbol,       // Simulation results summary
    pub breaking_change: bool,             // Whether this is a breaking change
    pub halt_reason: Symbol,               // Reason if halted (empty if not halted)
    pub halted_by: Option<Address>,        // Who halted it (None if not halted)
    pub halted_at: u64,                    // When it was halted (0 if not halted)
}
```

### 2. Enhanced ProposalStatus Enum

```rust
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProposalStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Executed = 3,
    Cancelled = 4,
    Halted = 5,        // New status for emergency halts
}
```

### 3. Validation Module Interface

```rust
pub struct ValidationModule;

impl ValidationModule {
    /// Validate proposal parameters before creation
    pub fn validate_proposal_params(
        env: &Env,
        new_contract_hash: &Symbol,
        target_contract: &Address,
        approval_threshold: u32,
        approvers: &Vec<Address>,
        timelock_delay: u64,
        current_version: u32,
        proposed_version: u32,
    ) -> Result<(), GovernanceError>;
    
    /// Validate contract hash format
    fn validate_hash_format(hash: &Symbol) -> Result<(), GovernanceError>;
    
    /// Validate contract address exists
    fn validate_contract_address(env: &Env, address: &Address) -> Result<(), GovernanceError>;
    
    /// Validate approval threshold
    fn validate_threshold(threshold: u32, approver_count: u32) -> Result<(), GovernanceError>;
    
    /// Validate timelock meets minimum
    fn validate_timelock(timelock: u64, is_mainnet: bool) -> Result<(), GovernanceError>;
    
    /// Validate approvers are unique
    fn validate_approvers_unique(approvers: &Vec<Address>) -> Result<(), GovernanceError>;
    
    /// Validate version compatibility
    fn validate_version_compatibility(
        current: u32,
        proposed: u32,
    ) -> Result<(), GovernanceError>;
}
```

### 4. Halt Module Interface

```rust
pub struct HaltModule;

impl HaltModule {
    /// Emergency halt an approved proposal
    pub fn halt_proposal(
        env: &Env,
        proposal_id: u64,
        admin: Address,
        reason: Symbol,
    ) -> Result<(), GovernanceError>;
    
    /// Resume a halted proposal with new timelock
    pub fn resume_proposal(
        env: &Env,
        proposal_id: u64,
        admin: Address,
        new_timelock_delay: u64,
    ) -> Result<(), GovernanceError>;
    
    /// Check if proposal is halted
    pub fn is_halted(env: &Env, proposal_id: u64) -> bool;
}
```

### 5. Approval Module Interface

```rust
pub struct ApprovalModule;

impl ApprovalModule {
    /// Approve with cooling-off period check
    pub fn approve_with_cooling_off(
        env: &Env,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), GovernanceError>;
    
    /// Revoke an approval before execution
    pub fn revoke_approval(
        env: &Env,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), GovernanceError>;
    
    /// Get time remaining until execution possible
    pub fn get_time_to_execution(
        env: &Env,
        proposal_id: u64,
    ) -> Result<u64, GovernanceError>;
    
    /// Record approval timestamp
    fn record_approval_timestamp(
        env: &Env,
        proposal_id: u64,
        approver: &Address,
        timestamp: u64,
    );
    
    /// Check if cooling-off period has passed
    fn check_cooling_off_period(
        env: &Env,
        proposal: &UpgradeProposal,
    ) -> Result<(), GovernanceError>;
}
```

### 6. Enhanced Error Codes

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovernanceError {
    // Existing errors
    Unauthorized = 2001,
    InvalidProposal = 2002,
    InsufficientApprovals = 2003,
    TimelockNotExpired = 2004,
    ProposalNotApproved = 2005,
    InvalidThreshold = 2006,
    DuplicateApproval = 2007,
    ProposalNotFound = 2008,
    
    // New validation errors
    InvalidHashFormat = 2009,
    InvalidContractAddress = 2010,
    TimelockTooShort = 2011,
    DuplicateApprover = 2012,
    InvalidVersion = 2013,
    VersionNotIncreasing = 2014,
    
    // New halt errors
    ProposalHalted = 2015,
    CannotHaltExecuted = 2016,
    NotHalted = 2017,
    
    // New approval errors
    CoolingOffNotExpired = 2018,
    ApprovalNotFound = 2019,
    CannotRevokeAfterThreshold = 2020,
}
```

## Data Models

### Approval Timestamp Storage

```rust
// Storage key: ("appr_ts", proposal_id, approver) -> timestamp
// Maps each approval to its timestamp for tracking
```

### Approval Revocation Storage

```rust
// Storage key: ("appr_rev", proposal_id, approver) -> bool
// Tracks which approvals have been revoked
```

### Simulation Metadata Storage

```rust
// Stored within UpgradeProposal struct
// simulation_passed: bool
// simulation_metadata: Symbol (JSON-like string with results)
```

### Version Tracking

```rust
// Stored within UpgradeProposal struct
// current_version: u32
// proposed_version: u32
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property Reflection

After reviewing all testable criteria, I've identified the following consolidations to eliminate redundancy:

- Properties 1.1-1.6 can be consolidated into a single comprehensive validation property
- Properties 2.1, 2.2, 2.3 are all about metadata round-tripping and can be combined
- Properties 3.1, 3.2, 3.3 all relate to halt functionality and can be consolidated
- Properties 4.1, 4.2 both test cooling-off period enforcement
- Properties 6.1, 6.2, 6.3, 6.4, 6.6 all test event emission and can be consolidated

### Correctness Properties

Property 1: Proposal validation completeness
*For any* proposal creation attempt, if any validation rule fails (empty hash, invalid address, invalid threshold, insufficient timelock, duplicate approvers, or non-increasing version), then the proposal SHALL be rejected with the appropriate error code
**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 5.1**

Property 2: Metadata round-trip consistency
*For any* proposal created with metadata (version info, simulation results, breaking change flags), retrieving that proposal SHALL return all the same metadata values
**Validates: Requirements 2.1, 2.2, 2.3, 5.5**

Property 3: Simulation warning flag
*For any* proposal with failed simulation tests, the proposal SHALL have the warning flag set to true
**Validates: Requirements 2.4**

Property 4: Execution event completeness
*For any* proposal that is executed, an event SHALL be emitted containing all validation and simulation metadata
**Validates: Requirements 2.5**

Property 5: Halt prevents execution
*For any* proposal in Halted status, execution attempts SHALL fail even if timelock has expired and approval threshold is met
**Validates: Requirements 3.1, 3.2, 3.3**

Property 6: Halt event emission
*For any* halt operation, an event SHALL be emitted with halt reason, halting address, and timestamp
**Validates: Requirements 3.4, 6.6**

Property 7: Resume authorization
*For any* halted proposal, only the original proposer admin or a super-admin SHALL be able to resume it, and resume SHALL require a new timelock period
**Validates: Requirements 3.5, 3.6**

Property 8: Cooling-off period enforcement
*For any* proposal, approval attempts before the cooling-off period expires SHALL be rejected
**Validates: Requirements 4.1, 4.2**

Property 9: Approval timestamp recording
*For any* approval action, the system SHALL record the timestamp of that approval
**Validates: Requirements 4.3**

Property 10: Timelock calculation from final approval
*For any* proposal that reaches approval threshold, the execution time SHALL be calculated from the timestamp of the final approval plus the timelock delay
**Validates: Requirements 4.4**

Property 11: Time-to-execution query accuracy
*For any* proposal at any point in time, querying time remaining until execution SHALL return the correct value based on current time and execution time
**Validates: Requirements 4.5**

Property 12: Approval revocation before execution
*For any* approval that has not yet contributed to reaching the threshold, the approver SHALL be able to revoke it before proposal execution
**Validates: Requirements 4.6**

Property 13: Semantic versioning format
*For any* proposal with a version number, the version SHALL follow semantic versioning format (major.minor.patch)
**Validates: Requirements 5.3**

Property 14: Breaking change acknowledgment
*For any* proposal marked as a breaking change, the proposal metadata SHALL contain explicit acknowledgment
**Validates: Requirements 5.4**

Property 15: Governance action event emission
*For any* governance action (propose, approve, reject, execute, cancel, halt, resume), an event SHALL be emitted with complete details including actor, action, timestamp, and parameters
**Validates: Requirements 6.1, 6.2, 6.3, 6.4**

Property 16: Proposal history completeness
*For any* proposal, querying its history SHALL return a complete timeline of all actions taken on that proposal in chronological order
**Validates: Requirements 6.5**

## Error Handling

### Validation Errors

All validation errors will be caught during proposal creation and return specific error codes:

- `InvalidHashFormat`: Contract hash is empty or malformed
- `InvalidContractAddress`: Target contract address is invalid or doesn't exist
- `InvalidThreshold`: Threshold is zero or exceeds approver count
- `TimelockTooShort`: Timelock delay is below minimum (3600s testnet, 86400s mainnet)
- `DuplicateApprover`: Approver list contains duplicate addresses
- `InvalidVersion`: Version number is malformed
- `VersionNotIncreasing`: Proposed version is not greater than current version

### Halt Errors

- `ProposalHalted`: Attempted to execute a halted proposal
- `CannotHaltExecuted`: Attempted to halt an already executed proposal
- `NotHalted`: Attempted to resume a proposal that isn't halted

### Approval Errors

- `CoolingOffNotExpired`: Attempted to approve before cooling-off period
- `ApprovalNotFound`: Attempted to revoke non-existent approval
- `CannotRevokeAfterThreshold`: Attempted to revoke after threshold reached

### Error Recovery

1. **Validation Failures**: User must correct parameters and resubmit proposal
2. **Halt Situations**: Admin must investigate, then either resume with new timelock or cancel
3. **Approval Issues**: Approver must wait for cooling-off period or coordinate with other approvers

## Testing Strategy

### Unit Testing

Unit tests will cover:
- Individual validation functions with valid and invalid inputs
- Halt and resume state transitions
- Approval and revocation logic
- Timestamp recording and calculation
- Error code returns for each failure scenario

### Property-Based Testing

We will use the `proptest` crate for Rust property-based testing. Each correctness property will be implemented as a property-based test with at least 100 iterations.

**Property-Based Testing Library**: `proptest` (Rust)

**Test Configuration**: Minimum 100 iterations per property test

**Test Tagging**: Each property-based test will be tagged with a comment in this format:
```rust
// **Feature: contract-upgrade-security, Property 1: Proposal validation completeness**
```

**Key Property Tests**:

1. **Validation Completeness** (Property 1): Generate random proposal parameters including invalid ones, verify correct validation and error codes
2. **Metadata Round-trip** (Property 2): Generate random proposals with metadata, verify retrieval returns same data
3. **Halt Prevents Execution** (Property 5): Generate approved proposals, halt them, verify execution fails
4. **Cooling-off Enforcement** (Property 8): Generate proposals, attempt approvals at random times, verify cooling-off is enforced
5. **Approval Revocation** (Property 12): Generate proposals with approvals, revoke them, verify state is correct

### Integration Testing

Integration tests will verify:
- End-to-end proposal lifecycle with all new features
- Interaction between validation, halt, and approval modules
- Event emission for all governance actions
- Multi-contract scenarios (upgrading multiple contracts)

### Test Coverage Goals

- Unit test coverage: >90% of new code
- Property test coverage: All 16 correctness properties
- Integration test coverage: All major user workflows

## Security Considerations

### Attack Vectors and Mitigations

| Attack Vector | Mitigation |
|---------------|-----------|
| Bypass validation by manipulating parameters | All validation is enforced in contract code, immutable |
| Halt abuse by malicious admin | Only original proposer or super-admin can halt/resume |
| Approval before cooling-off | Enforced by timestamp checks in contract |
| Revoke approval after threshold | Revocation blocked once threshold reached |
| Execute halted proposal | Halt status checked before execution |
| Version downgrade attack | Version must be strictly increasing |
| Duplicate approver attack | Uniqueness validated at proposal creation |

### Threat Model

**In Scope (This design prevents)**:
- Invalid proposals from being created ✓
- Premature approvals during cooling-off ✓
- Execution of halted proposals ✓
- Version downgrades ✓
- Approval manipulation after threshold ✓

**Out of Scope (Require external measures)**:
- All approvers colluding to approve malicious upgrade
- Compromise of admin private keys
- Social engineering of governance participants
- Network-level attacks on Stellar/Soroban

### Security Best Practices

1. **Validation First**: All validation happens before state changes
2. **Fail Secure**: Default to rejection on any validation failure
3. **Immutable Checks**: Validation logic cannot be bypassed
4. **Audit Trail**: All actions emit events for monitoring
5. **Defense in Depth**: Multiple layers (validation, cooling-off, halt, timelock)

## Implementation Notes

### Storage Optimization

- Use persistent storage for proposals and approvals (long-lived data)
- Use instance storage for temporary validation state
- Minimize storage keys by using composite keys where possible

### Gas Optimization

- Batch validation checks to minimize iterations
- Cache frequently accessed data (current version, admin addresses)
- Use efficient data structures (Maps for O(1) lookups)

### Backward Compatibility

- New fields in `UpgradeProposal` have default values for existing proposals
- Existing contracts can upgrade to use new features without breaking changes
- Old proposal format can be migrated by setting new fields to defaults

### Deployment Strategy

1. Deploy enhanced shared governance library
2. Test on testnet with sample proposals
3. Upgrade existing contracts to use new library
4. Verify all existing functionality still works
5. Enable new features (validation, halt, cooling-off)
6. Monitor for issues and iterate

## Performance Considerations

### Expected Performance

- Proposal creation: ~1000 gas (with validation)
- Approval: ~500 gas (with cooling-off check)
- Halt: ~300 gas (status update)
- Execution: ~800 gas (with halt check)

### Scalability

- System supports unlimited proposals (limited by storage)
- Each proposal can have up to 100 approvers (practical limit)
- Cooling-off period: 1 hour minimum (configurable)
- Timelock period: 1 hour testnet, 24 hours mainnet minimum

## Monitoring and Observability

### Key Metrics

- Proposal creation rate
- Validation failure rate by error type
- Halt frequency and reasons
- Approval revocation rate
- Time from proposal to execution

### Alerts

- Alert on validation failure spike (possible attack)
- Alert on halt events (requires investigation)
- Alert on approval revocation (unusual behavior)
- Alert on execution failures (system issue)

### Logging

All events are emitted on-chain and can be indexed by:
- Backend indexer service
- Stellar Horizon API
- Custom monitoring tools

## Future Enhancements

### Potential Improvements

1. **Automated Simulation**: On-chain simulation execution before approval
2. **Reputation System**: Track governance participant behavior
3. **Weighted Voting**: Different approvers have different voting power
4. **Time-based Decay**: Approvals expire after certain time
5. **Multi-stage Approval**: Different thresholds for different proposal types

### Extensibility

The design is modular and can be extended with:
- Additional validation rules
- Custom halt conditions
- Alternative approval mechanisms
- Enhanced metadata schemas

---

**Design Version**: 1.0  
**Last Updated**: February 25, 2026  
**Status**: Ready for Implementation
