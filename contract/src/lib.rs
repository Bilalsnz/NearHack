use near_sdk::{
    env, near, AccountId, NearToken, PanicOnDefault, Promise,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    json_types::U128,
    serde::{Deserialize, Serialize},
};

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Milestone {
    pub label: String,
    pub amount: u128,
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Project {
    pub id: u64,
    pub owner: AccountId,
    pub title: String,
    pub goal: u128,
    pub raised: u128,
    pub paid: u128,
    pub milestone_index: u8,
    pub milestones: Vec<Milestone>,
    pub status: String,
    pub evidence_hash: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MilestoneView { pub label: String, pub amount: U128 }

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectView {
    pub id: u64, pub owner: AccountId, pub title: String,
    pub goal: U128, pub raised: U128, pub paid: U128,
    pub milestone_index: u8, pub milestones: Vec<MilestoneView>,
    pub status: String, pub evidence_hash: Option<String>,
    pub failure_reason: Option<String>,
}

impl From<Project> for ProjectView {
    fn from(p: Project) -> Self {
        Self {
            id: p.id, owner: p.owner, title: p.title,
            goal: U128(p.goal), raised: U128(p.raised), paid: U128(p.paid),
            milestone_index: p.milestone_index,
            milestones: p.milestones.into_iter().map(|m| MilestoneView {
                label: m.label, amount: U128(m.amount),
            }).collect(),
            status: p.status, evidence_hash: p.evidence_hash,
            failure_reason: p.failure_reason,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MilestoneInput { pub label: String, pub amount: U128 }

#[derive(BorshDeserialize, BorshSerialize, Clone, Default)]
pub struct VoteTally { pub approve: u8, pub reject: u8, pub total: u8 }

const MILESTONE_VOTE_QUORUM: u8 = 5;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct DreamFund {
    owner: AccountId,
    pool_balance: u128,
    next_project_id: u64,
    projects: LookupMap<u64, Project>,
    project_ids: Vec<u64>,
    reputation: LookupMap<AccountId, u64>,
}

#[near]
impl DreamFund {
    /// Deploy with an optional attached deposit to seed the public pool.
    #[init]
    #[payable]
    pub fn new() -> Self {
        Self {
            owner: env::predecessor_account_id(),
            pool_balance: env::attached_deposit().as_yoctonear(),
            next_project_id: 0,
            projects: LookupMap::new(b"p"),
            project_ids: Vec::new(),
            reputation: LookupMap::new(b"r"),
        }
    }

    pub fn create_project(&mut self, title: String, goal: U128, milestones: Vec<MilestoneInput>) -> u64 {
        assert!(!title.trim().is_empty(), "Title is required");
        assert!(goal.0 > 0, "Goal must be greater than zero");
        assert!(!milestones.is_empty() && milestones.len() <= 2, "Use one or two milestones");
        for m in milestones.iter() {
            assert!(!m.label.trim().is_empty(), "Milestone label is required");
            assert!(m.amount.0 > 0, "Milestone amount must be positive");
        }
        let total: u128 = milestones.iter().map(|m| m.amount.0).sum();
        assert!(total > 0 && total <= goal.0, "Milestone total must be > 0 and <= goal");
        let id = self.next_project_id;
        self.next_project_id += 1;
        let project = Project {
            id, owner: env::predecessor_account_id(), title, goal: goal.0,
            raised: 0, paid: 0, milestone_index: 0,
            milestones: milestones.into_iter().map(|m| Milestone { label: m.label, amount: m.amount.0 }).collect(),
            status: "funding".to_string(), evidence_hash: None, failure_reason: None,
        };
        self.projects.insert(&id, &project);
        self.project_ids.push(id);
        id
    }

    /// Add NEAR to the unallocated shared community pool.
    #[payable]
    pub fn fund_pool(&mut self) {
        let amount = env::attached_deposit().as_yoctonear();
        assert!(amount > 0, "Attach a positive NEAR amount");
        self.pool_balance = self.pool_balance.checked_add(amount).expect("Pool balance overflow");
        env::log_str(&format!("DREAMFUND_POOL_FUNDED amount={}", amount));
    }

    /// The demo resolver assigns community-pool funds to a project's locked allocation.
    pub fn allocate_from_pool(&mut self, project_id: u64, amount: U128) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only the demo resolver can allocate pool funds");
        assert!(amount.0 > 0, "Allocation must be positive");
        assert!(self.pool_balance >= amount.0, "Insufficient community pool balance");
        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert!(project.status == "funding" || project.status == "in_progress", "Project is not accepting funds");
        let new_raised = project.raised.checked_add(amount.0).expect("Project raised overflow");
        assert!(new_raised <= project.goal, "Allocation exceeds project goal");
        self.pool_balance -= amount.0;
        project.raised = new_raised;
        project.status = "in_progress".to_string();
        env::log_str(&format!("DREAMFUND_POOL_ALLOCATED project={} amount={}", project_id, amount.0));
        self.projects.insert(&project_id, &project);
    }

    /// Attach NEAR to pledge directly to a project's locked allocation.
    #[payable]
    pub fn pledge(&mut self, project_id: u64) {
        let amount = env::attached_deposit().as_yoctonear();
        assert!(amount > 0, "Attach a positive NEAR amount");
        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert!(project.status == "funding" || project.status == "in_progress", "Project is not accepting pledges");
        let new_raised = project.raised.checked_add(amount).expect("Project raised overflow");
        assert!(new_raised <= project.goal, "Pledge exceeds project goal");
        project.raised = new_raised;
        if project.status == "funding" { project.status = "in_progress".to_string(); }
        self.projects.insert(&project_id, &project);
    }

    pub fn submit_evidence(&mut self, project_id: u64, evidence_hash: String) {
        assert!(!evidence_hash.trim().is_empty(), "Evidence hash is required");
        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert_eq!(env::predecessor_account_id(), project.owner, "Only the project owner can submit evidence");
        assert_eq!(project.status, "in_progress", "Project is not active");
        assert!((project.milestone_index as usize) < project.milestones.len(), "No milestone remains");
        project.evidence_hash = Some(evidence_hash);
        self.projects.insert(&project_id, &project);
    }

    /// One vote per non-founder account. Five accounts trigger majority resolution.
    pub fn vote_milestone(&mut self, project_id: u64, approve: bool) -> bool {
        let voter = env::predecessor_account_id();
        let project = self.projects.get(&project_id).expect("Project not found");
        assert_eq!(project.status, "in_progress", "Project is not active");
        assert!(project.evidence_hash.is_some(), "Evidence must be submitted first");
        assert!((project.milestone_index as usize) < project.milestones.len(), "No milestone remains");
        assert_ne!(voter, project.owner, "Project owners cannot vote on their own milestones");

        let index = project.milestone_index;
        let mut votes: LookupMap<(u64, u8, AccountId), bool> = LookupMap::new(b"mv");
        let mut tallies: LookupMap<(u64, u8), VoteTally> = LookupMap::new(b"mt");
        let vote_key = (project_id, index, voter.clone());
        assert!(votes.get(&vote_key).is_none(), "You have already voted on this milestone");
        let mut tally = tallies.get(&(project_id, index)).unwrap_or_default();
        assert!(tally.total < MILESTONE_VOTE_QUORUM, "Voting is already closed");
        if approve { tally.approve += 1; } else { tally.reject += 1; }
        tally.total += 1;
        votes.insert(&vote_key, &approve);
        tallies.insert(&(project_id, index), &tally);
        env::log_str(&format!("DREAMFUND_MILESTONE_VOTE project={} milestone={} approve={} total={}", project_id, index, approve, tally.total));
        if tally.total == MILESTONE_VOTE_QUORUM {
            self.finalize_milestone(project_id, project, tally.approve > tally.reject);
            true
        } else { false }
    }

    pub fn get_milestone_votes(&self, project_id: u64) -> (u8, u8, u8) {
        let project = self.projects.get(&project_id).expect("Project not found");
        let tallies: LookupMap<(u64, u8), VoteTally> = LookupMap::new(b"mt");
        let tally = tallies.get(&(project_id, project.milestone_index)).unwrap_or_default();
        (tally.approve, tally.reject, tally.total)
    }

    pub fn get_my_milestone_vote(&self, project_id: u64, voter: AccountId) -> Option<bool> {
        let project = self.projects.get(&project_id).expect("Project not found");
        let votes: LookupMap<(u64, u8, AccountId), bool> = LookupMap::new(b"mv");
        votes.get(&(project_id, project.milestone_index, voter))
    }

    fn finalize_milestone(&mut self, project_id: u64, mut project: Project, pass: bool) {
        let index = project.milestone_index as usize;
        if pass {
            let tranche = project.milestones[index].amount;
            let available = project.raised.saturating_sub(project.paid);
            assert!(available >= tranche, "Project has not raised enough for this tranche");
            project.paid += tranche;
            project.milestone_index += 1;
            project.evidence_hash = None;
            let rep = self.reputation.get(&project.owner).unwrap_or(0);
            self.reputation.insert(&project.owner, &(rep + 1));
            if (project.milestone_index as usize) >= project.milestones.len() {
                project.status = "complete".to_string();
                let recycled = project.raised.saturating_sub(project.paid);
                self.pool_balance = self.pool_balance.checked_add(recycled).expect("Pool balance overflow");
                if recycled > 0 { env::log_str(&format!("DREAMFUND_POOL_RECYCLED project={} amount={} reason=completed_surplus", project_id, recycled)); }
            }
            let recipient = project.owner.clone();
            self.projects.insert(&project_id, &project);
            env::log_str(&format!("DREAMFUND_MILESTONE_PAID project={} owner={} amount={}", project_id, recipient, tranche));
            Promise::new(recipient).transfer(NearToken::from_yoctonear(tranche));
        } else {
            let remaining = project.raised.saturating_sub(project.paid);
            self.pool_balance = self.pool_balance.checked_add(remaining).expect("Pool balance overflow");
            env::log_str(&format!("DREAMFUND_POOL_RECYCLED project={} amount={} reason=community_vote_rejected", project_id, remaining));
            project.status = "failed".to_string();
            project.failure_reason = Some("Milestone rejected by community vote".to_string());
            project.evidence_hash = None;
            self.projects.insert(&project_id, &project);
        }
    }

    pub fn get_pool(&self) -> U128 { U128(self.pool_balance) }
    pub fn get_project(&self, project_id: u64) -> Option<ProjectView> {
        self.projects.get(&project_id).map(|p| ProjectView::from(p.clone()))
    }
    pub fn get_projects(&self) -> Vec<ProjectView> {
        self.project_ids.iter().filter_map(|id| self.projects.get(id).map(ProjectView::from)).collect()
    }
    pub fn get_reputation(&self, account: AccountId) -> u64 { self.reputation.get(&account).unwrap_or(0) }
    pub fn get_owner(&self) -> AccountId { self.owner.clone() }
}
