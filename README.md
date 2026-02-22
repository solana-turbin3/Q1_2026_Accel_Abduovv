# Q1 2026 Accel - Abduovv

Turbin3 Q1 Accelerated Builders Proof of Work

A collection of Solana programs demonstrating advanced Anchor development patterns, Token-2022 extensions, oracle integrations, and automated transaction scheduling.

---

## Projects

### 🏦 [escrow-litesvm](./escrow-litesvm/)
**Trustless Token Swap Escrow with LiteSVM Testing**

A trustless escrow mechanism for secure token swaps between two parties. Alice (maker) offers to exchange Token A for Token B, and Bob (taker) can accept to complete the swap atomically. Features a 5-day waiting period before the offer can be taken, ensuring security against instant trades.

**Key Features:**
- Maker deposits tokens into a program-controlled vault
- Taker can fulfill the trade after the 5-day waiting period
- Maker can refund at any time
- Tested using **LiteSVM** for fast, in-process testing without a local validator

---

### 🤖 [gpt-oracle](./gpt-oracle/)
**AI Oracle with Automated Scheduling**

A Solana program that queries an on-chain AI oracle (powered by [MagicBlock](https://magicblock.gg)) with automated scheduling via [TukTuk](https://www.tuktuk.fun). Initialize an AI agent with a system prompt, and TukTuk's crankers automatically fire queries on a schedule.

**Key Features:**
- CPI to MagicBlock's GPT oracle for off-chain AI processing
- TukTuk integration for permissionless automated scheduling
- Callback pattern for receiving LLM responses on-chain
- Deployed on devnet with live oracle and task queue

---

### 🎲 [magicblock-vrf](./magicblock-vrf/)
**Verifiable Random Function Integration**

Demonstrates VRF integration using the MagicBlock ephemeral rollup SDK to update on-chain state with verifiable randomness. Covers both base layer and ephemeral rollup implementations.

**Key Features:**
- Request randomness on Solana base layer
- Delegate accounts to ephemeral rollup for faster, cheaper execution
- Two-transaction pattern: request → oracle callback
- On-chain seed derivation from clock and payer pubkey

---

### 📦 [transfer-hook-vault](./transfer-hook-vault/)
**Whitelisted Token Vault with Transfer Hook**

A whitelisted token vault using Token-2022's Transfer Hook extension. Only admin-approved users can hold and transfer the vault's token. Every transfer is validated on-chain by the transfer hook.

**Key Features:**
- Token-2022 mint with TransferHook, MetadataPointer, and TokenMetadata extensions
- Whitelist enforcement via transfer hook
- Deposit/withdraw with atomic token transfers
- ExtraAccountMetaList for dynamic account resolution
- Tested with LiteSVM 0.9.1 (Token-2022 as built-in)

---

### 🔐 [tuktuk-escrow](./tuktuk-escrow/)
**Escrow with Automated Expiry Refunds**

A trustless token escrow with automated expiry refunds powered by TukTuk. If no taker fulfills the trade before expiry, TukTuk's crankers automatically call `auto_refund` to return the maker's tokens.

**Key Features:**
- Time-limited escrow with `expires_at` timestamp
- Permissionless `auto_refund` callable after expiry
- TukTuk integration for automated scheduling at exact timestamp
- Manual refund option for maker at any time
- End-to-end devnet tests with live TukTuk integration

---

### ✅ [whitelist-transfer-hook](./whitelist-transfer-hook/)
**Transfer Hook Access Control**

Implements the SPL Token-2022 Transfer Hook interface to enforce whitelist restrictions on token transfers. Only whitelisted addresses can transfer tokens with this hook enabled.

**Key Features:**
- O(1) whitelist lookups via per-address PDAs
- Token-2022 mint creation with TransferHook extension
- ExtraAccountMetaList for runtime PDA derivation
- Admin-controlled whitelist management
- Validation during every token transfer

---

### 📝 [persistent-todo-queue](./persistent-todo-queue/)
**CLI Todo with Persistent FIFO Queue**

A Rust CLI application demonstrating persistent storage using Borsh serialization. Tasks are stored in a FIFO queue that survives application restarts.

**Key Features:**
- Generic FIFO queue implementation
- Borsh binary serialization to `todos.bin`
- Add, list, and complete tasks
- Persistent storage across restarts

---

### 🔧 [generic-storage](./generic-storage/)
*Placeholder project - basic Rust project structure*

---

## Build & Test

Each project has its own build configuration. Common commands:

```bash
# Anchor projects (most programs)
cd <project-name>
anchor build
anchor test

# LiteSVM projects
cd escrow-litesvm transfer-hook-vault
make build
make test

# CLI project
cd persistent-todo-queue
cargo build --release
cargo run
```

---

## Technologies Used

| Technology | Usage |
|------------|-------|
| **Anchor** | Program framework for most projects |
| **LiteSVM** | In-process testing for escrow and transfer-hook-vault |
| **Token-2022** | Transfer Hook, MetadataPointer, TokenMetadata extensions |
| **TukTuk** | Permissionless crank scheduler for automation |
| **MagicBlock Oracle** | GPT oracle and VRF integration |
| **Borsh** | Binary serialization for persistent storage |

---

## Devnet Deployments

| Project | Program ID | Additional Accounts |
|---------|------------|---------------------|
| gpt-oracle | `8d6wKSQNNoqSu98EgLn5ZotmJMZHq8cgcfLGsiubUqZe` | Oracle: `LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab` |
| tuktuk-escrow | `92t1k1s6XLTzrFzKvHFRHVX8At6DuzP9BSzkXT33pHjA` | Task Queue: `UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2` |

---

## License

MIT
