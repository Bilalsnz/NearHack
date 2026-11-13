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
- `fund_pool()` — payable; contributes to the shared pool
- `allocate_from_pool(project_id, amount)` — owner-only demo allocation from the shared pool
- `pledge(project_id)` — payable direct project contribution
- `submit_evidence(project_id, evidence_hash)`
- `resolve_milestone(project_id, pass)`
- `get_pool()`
- `get_project(project_id)`
- `get_projects()`
- `get_reputation(account)`
- `get_owner()`

## Frontend

Static HTML/CSS/JS at the repository root. Configure `CONTRACT_ID` in `app.js` after deploying the contract, then deploy the static site (for example, with Vercel).

## Deployment checklist

- [ ] Build and deploy the Rust contract to a NEAR testnet account.
- [ ] Set the deployed contract ID in `app.js`.
- [ ] Connect with a testnet wallet.
- [ ] Run both pass and fail paths with real testnet transactions.
- [ ] Add explorer transaction links and the live demo URL here.
- [ ] Record the 90-second demo and publish the hackathon post.
