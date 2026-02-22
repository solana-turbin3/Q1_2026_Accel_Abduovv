# Transfer Hook Vault

A whitelisted token vault using Token-2022's Transfer Hook extension. Only admin-approved users can hold and transfer the vault's token. Every transfer is validated on-chain by checking that the sender is whitelisted.

Uses **LiteSVM** for fast, in-process testing.

---

## Architecture

### Vault PDA
Stores the admin and mint for the vault.
```rust
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
```

### UserAccount PDA
Per-user whitelist entry tracking deposited balance.
```rust
pub struct UserAccount {
    pub account: Pubkey,
    pub amount: u64,
    pub bump: u8,
}
```

---

## Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize` | Creates vault PDA + Token-2022 mint with TransferHook, MetadataPointer, TokenMetadata |
| `add_user` / `remove_user` | Admin-only whitelist management |
| `init_extra_acc_meta` | Creates ExtraAccountMetaList for transfer hook account resolution |
| `transfer_hook` | Auto-called by Token-2022 on every transfer; validates sender is whitelisted |
| `deposit` | Updates ledger; paired with `transfer_checked` (user → vault) in same tx |
| `withdraw` | Approves user as delegate; paired with `transfer_checked` (vault → user) |

---

## Flow

```
1. initialize()           -> Vault PDA + Token-2022 mint with extensions
2. add_user(address)      -> Whitelist user
3. init_extra_acc_meta()  -> Setup ExtraAccountMetaList for hook
4. deposit() + transfer   -> User deposits tokens (atomic tx)
5. withdraw() + transfer  -> User withdraws tokens (atomic tx)
```

**Deposit/Withdraw Pattern:** Token transfers are paired with ledger updates in atomic transactions to avoid reentrancy errors from the transfer hook.

---

## LiteSVM Testing

Token-2022 and Associated Token Program are built into LiteSVM 0.9.1 — no `.so` fixtures needed.

```bash
make build
make test
```

---
