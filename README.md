# DreamFund

**Community-powered, milestone-based crowdfunding for public goods on NEAR.**

DreamFund helps communities support public-good projects through transparent funding, measurable milestones, evidence submission, and community voting. Project funds are released milestone by milestone only after the required evidence is submitted and the community approves the milestone.

If a milestone is rejected, the project's unpaid funds still held by the contract return to the shared community pool.

## Live testnet deployment

- **Application:** https://dreamfund-ten.vercel.app/
- **Contract:** [0x_primee.testnet](https://testnet.nearblocks.io/address/0x_primee.testnet)
- **Network:** NEAR Testnet
- **Contract implementation:** `contract/src/lib.rs`

The application can display public contract state without a connected wallet. A NEAR testnet wallet is required for transactions such as creating projects, contributing to the pool, pledging, submitting evidence, allocating pool funds, and voting.

## How DreamFund works

1. **Contribute to the community pool** with `fund_pool()`, or pledge directly to a project with `pledge(project_id)`.
2. **Propose a public-good project** with `create_project(title, goal, milestones)`. Each project defines a funding goal and one or two measurable payment milestones.
3. **Allocate pool funds when needed.** The contract owner can assign available shared-pool funds to a project with `allocate_from_pool(project_id, amount)`. Direct pledges go straight to the project's locked funding balance.
4. **Submit milestone evidence.** The project owner submits a non-empty evidence value using `submit_evidence(project_id, evidence_hash)`. The frontend hashes ordinary evidence input with SHA-256 when it is not already a 32-byte hexadecimal hash.
5. **Community members vote.** Five eligible NEAR accounts cast an approval or rejection vote using `vote_milestone(project_id, approve)`.
6. **The contract resolves the milestone automatically** when the fifth vote is recorded:
   - **Approved:** a strict majority of approval votes releases the current milestone tranche to the project owner. The owner's reputation score increases by one.
   - **Rejected:** a strict majority of rejection votes marks the project as failed and returns its remaining unpaid raised funds to the shared pool.
7. **Verify activity on-chain** by inspecting contract state and transaction receipts on the NEAR Testnet explorer.

## Voting rules

- **Five-account quorum:** exactly five eligible NEAR accounts must vote for a milestone to be resolved.
- **Automatic resolution:** the contract evaluates the result as soon as the fifth vote is recorded.
- **Majority decision:** approval or rejection must receive more than half of the five votes.
- **Project-owner exclusion:** a project owner cannot vote on their own project's milestones.
- **One vote per account per milestone:** an account cannot submit a second vote for the same milestone. The next milestone has its own vote record.
- **Evidence required:** voting is available only after the project owner has submitted evidence for the active milestone.
- **Pending votes:** the contract does not currently impose a voting deadline. A milestone remains pending until five eligible accounts have voted.
- **Account-based eligibility:** voting eligibility is enforced by NEAR account ID. The contract does not verify that each account represents a different human.

## Funding and milestone behavior

- Projects can define one or two milestones.
- Each milestone must have a non-empty label and a positive tranche amount.
- The sum of milestone tranches must not exceed the project's funding goal.
- A project must have enough raised, unpaid funds to cover a tranche before an approved milestone can be paid.
- Funds already transferred to a project owner cannot be clawed back by the contract.
- When a project is rejected, its remaining unpaid raised funds are returned to the shared pool.
- When a project completes, any raised amount remaining after its milestone payments is recycled into the shared pool.
- Reputation increases after a successful milestone release. It is a project activity signal, not an identity-verification mechanism.

## Contract methods

| Method | Purpose |
| --- | --- |
| `new()` | Initializes the contract; an attached deposit seeds the shared pool. |
| `create_project(title, goal, milestones)` | Creates a project with a funding goal and one or two milestones. |
| `fund_pool()` | Adds a payable contribution to the shared community pool. |
| `allocate_from_pool(project_id, amount)` | Assigns shared-pool funds to a project. This action is restricted to the contract owner in the current implementation. |
| `pledge(project_id)` | Adds a payable contribution directly to a project's funding balance. |
| `submit_evidence(project_id, evidence_hash)` | Allows the project owner to submit evidence for the active milestone. |
| `vote_milestone(project_id, approve)` | Records one vote per account per milestone; the fifth vote triggers majority resolution. |
| `get_milestone_votes(project_id)` | Returns approval, rejection, and total vote counts for the active milestone. |
| `get_my_milestone_vote(project_id, voter)` | Returns the connected account's recorded vote for the active milestone when queried with that account ID. |
| `get_pool()` | Reads the shared-pool balance. |
| `get_project(project_id)` | Reads an individual project. |
| `get_projects()` | Reads the project list. |
| `get_reputation(account)` | Reads a project's owner's reputation score. |
| `get_owner()` | Reads the contract owner account. |

The contract emits receipt logs for pool funding, pool allocation, votes, milestone payouts, and recycled funds. Transaction receipts provide the on-chain record of each state transition.

## Repository structure

- `contract/src/lib.rs` — NEAR Rust smart contract.
- `app.js`, `index.html`, `styles.css` — web application.
- `config.js` — public deployed contract configuration.
- `.github/workflows/` — testnet contract deployment workflow.

## Development and deployment

The deployment workflow uses GitHub Actions variables and secrets:

- `NEAR_TESTNET_ACCOUNT_ID` — account used for contract deployment.
- `NEAR_TESTNET_PRIVATE_KEY` — full-access testnet key used by deployment automation.

Never commit or share a private key or wallet seed phrase. The public website does not require the deployment key. Use a dedicated testnet account and testnet funds.

## Demo verification checklist

- [ ] Read the live pool balance and project list from `get_pool()` and `get_projects()`.
- [ ] Fund the shared pool and capture the transaction hash.
- [ ] Create projects with one- and two-milestone configurations.
- [ ] Pledge directly to a project or allocate shared-pool funds as the contract owner.
- [ ] Submit evidence for an active milestone.
- [ ] Record five eligible account votes and capture the approval payout receipt.
- [ ] Record a rejection vote sequence and capture the pool-recycling receipt and before/after pool balances.
- [ ] Include relevant NEAR Testnet explorer transaction links in the demo materials.

**Network notice:** DreamFund is currently deployed on NEAR Testnet. Do not send mainnet funds.

**Implementation status:** The repository contains the contract methods and frontend voting flow described above. Full five-account approval and rejection journeys should be described as demonstrated only after their corresponding live testnet transactions and receipts have been captured.
