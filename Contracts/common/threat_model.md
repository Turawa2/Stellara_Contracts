# Threat Model – Stellara Soroban Contracts

## Assets

- Admin authority
- Token balances
- Contract configuration

## Threat Actors

- Malicious users
- Compromised admin key
- External contracts calling via cross-contract calls

## Attack Vectors

### 1. Unauthorized Admin Access

Mitigation:

- Enforce `require_auth()` on all admin functions

### 2. Reentrancy

Mitigation:

- State updates occur BEFORE external calls
- No callback-based logic

### 3. Integer Overflow / Underflow

Mitigation:

- Use checked arithmetic
- Panic on overflow

### 4. Privilege Escalation

Mitigation:

- Immutable admin unless explicitly changed
- Admin change requires auth from current admin
