# DreamFund — NEAR HACKATHON / Social Money

A public-goods funding pool where projects receive money in milestones. Successful verification releases the next tranche. If a project fails, its **unreleased allocation returns to the shared community pool**, not to the founder.

## Current status

Initial project scaffold. The contract and browser UI are being wired together. Do not treat this repository as deployed until the testnet contract address and live transaction links are added below.

## Core demo

1. Connect a NEAR **testnet** wallet.
2. Create or open two projects.
3. Pledge testnet NEAR to each project.
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
- `pledge(project_id)` — payable
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
