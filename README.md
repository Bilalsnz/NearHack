# DreamFund — NEAR HACKATHON / Social Money

A public-goods funding pool where projects receive money in milestones. Successful verification releases the next tranche. If a project fails, its **unreleased allocation returns to the shared community pool**, not to the founder.

## Current status

Working scaffold with shared-pool contributions, resolver allocation, direct pledges, evidence submission, milestone payout, and fail-to-pool recycling. Testnet deployment and end-to-end transaction validation are still pending. Do not treat this repository as deployed until the testnet contract address and live transaction links are added below.

## Core demo

1. Connect a NEAR **testnet** wallet.
2. Create or open two projects.
3. Add testnet NEAR to the shared pool, then have the demo resolver allocate funds to projects. Direct project pledges are also supported.
4. Submit an evidence hash for a project's current milestone.
5. The demo resolver approves one milestone and the contract pays its tranche to the project owner.
6. The resolver fails the other project and its remaining locked funds return to the community pool.
7. Inspect the resulting transactions on NEAR testnet explorer.

## Honest limitations

- Milestone resolution is controlled by the contract owner for this hackathon demo. It is **not decentralized arbitration or a full DAO**.
- Projects support at most two milestones.
- Reputation counts successful milestone releases only.
- Funds already paid to a project owner cannot be clawed back. Only funds still held by the contract can return to the pool.
- Use testnet funds only.

## Contract

The Rust contract source is in `contract/`. Initial methods:

- `new()`
- `create_project(title, goal, milestones)`
- `fund_pool()` — payable; contributes to the shared pool (emits a `DREAMFUND_POOL_FUNDED` receipt log)
- `allocate_from_pool(project_id, amount)` — owner-only demo allocation from the shared pool (emits `DREAMFUND_POOL_ALLOCATED`)
- `pledge(project_id)` — payable direct project contribution
- `submit_evidence(project_id, evidence_hash)`
- `resolve_milestone(project_id, pass)` — emits payout or pool-recycling receipt logs
- `get_pool()`
- `get_project(project_id)`
- `get_projects()`
- `get_reputation(account)`
- `get_owner()`

## Frontend

Static HTML/CSS/JS at the repository root. The app reads the deployed account ID from `config.js`; the deployment workflow updates that file automatically.

## Deploy from a phone (GitHub Actions)

1. Create a dedicated NEAR **testnet** account with a full-access key and fund it with faucet NEAR.
2. In this repository open **Settings → Secrets and variables → Actions**.
3. Add repository variable `NEAR_TESTNET_ACCOUNT_ID` with the account ID (for example, `dreamfund-demo.testnet`).
4. Add repository secret `NEAR_TESTNET_PRIVATE_KEY` with that account's full-access private key. **Never paste the key into chat, an issue, or a commit.**
5. Open **Actions → Deploy DreamFund to NEAR testnet → Run workflow**.
6. The workflow builds the WASM, deploys it, initializes it, writes the public contract ID to `config.js`, and commits that config. Vercel then redeploys the frontend automatically.

Use a dedicated testnet account only. The key is needed by the workflow to deploy and initialize the contract; it is not used by the public website.

## Deployment checklist

- [ ] Build and deploy the Rust contract to a NEAR testnet account.
- [ ] Confirm `config.js` contains the deployed testnet contract ID.
- [ ] Connect with a testnet wallet.
- [ ] Run both pass and fail paths with real testnet transactions.
- [ ] Add explorer transaction links and the live demo URL here.
- [ ] Record the 90-second demo and publish the hackathon post.
