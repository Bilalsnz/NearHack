use near_sdk::{
    env, near, AccountId, NearToken, PanicOnDefault, Promise,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    json_types::U128,
    serde::{Deserialize, Serialize},
};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Milestone {
    pub label: String,
    pub amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Project {
    pub id: u64,
    pub owner: AccountId,
    pub title: String,
    pub goal: U128,
    pub raised: U128,
    pub paid: U128,
    pub milestone_index: u8,
    pub milestones: Vec<Milestone>,
    pub status: String,
    pub evidence_hash: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MilestoneInput {
    pub label: String,
    pub amount: U128,
}

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

    pub fn create_project(
        &mut self,
        title: String,
        goal: U128,
        milestones: Vec<MilestoneInput>,
    ) -> u64 {
        assert!(!title.trim().is_empty(), "Title is required");
        assert!(goal.0 > 0, "Goal must be greater than zero");
        assert!(!milestones.is_empty() && milestones.len() <= 2, "Use one or two milestones");

        let total: u128 = milestones.iter().map(|m| m.amount.0).sum();
        assert!(total > 0 && total <= goal.0, "Milestone total must be > 0 and <= goal");

        let id = self.next_project_id;
        self.next_project_id += 1;

        let project = Project {
            id,
            owner: env::predecessor_account_id(),
            title,
            goal,
            raised: U128(0),
            paid: U128(0),
            milestone_index: 0,
            milestones: milestones.into_iter().map(|m| Milestone {
                label: m.label,
                amount: m.amount,
            }).collect(),
            status: "funding".to_string(),
            evidence_hash: None,
            failure_reason: None,
        };

        self.projects.insert(&id, &project);
        self.project_ids.push(id);
        id
    }

    #[payable]
    pub fn pledge(&mut self, project_id: u64) {
        let amount = env::attached_deposit().as_yoctonear();
        assert!(amount > 0, "Attach a positive NEAR amount");

        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert!(project.status == "funding" || project.status == "in_progress", "Project is not accepting pledges");
        assert!(project.raised.0 + amount <= project.goal.0, "Pledge exceeds project goal");

        project.raised = U128(project.raised.0 + amount);
        if project.status == "funding" {
            project.status = "in_progress".to_string();
        }
        self.projects.insert(&project_id, &project);
    }

    pub fn submit_evidence(&mut self, project_id: u64, evidence_hash: String) {
        assert!(!evidence_hash.trim().is_empty(), "Evidence hash is required");
        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert_eq!(env::predecessor_account_id(), project.owner, "Only the project owner can submit evidence");
        assert!(project.status == "in_progress", "Project is not active");
        assert!((project.milestone_index as usize) < project.milestones.len(), "No milestone remains");
        project.evidence_hash = Some(evidence_hash);
        self.projects.insert(&project_id, &project);
    }

    pub fn resolve_milestone(&mut self, project_id: u64, pass: bool) {
        assert_eq!(env::predecessor_account_id(), self.owner, "Only the demo resolver can resolve milestones");
        let mut project = self.projects.get(&project_id).expect("Project not found");
        assert_eq!(project.status, "in_progress", "Project is not active");
        assert!(project.evidence_hash.is_some(), "Evidence must be submitted first");

        let index = project.milestone_index as usize;
        assert!(index < project.milestones.len(), "No milestone remains");

        if pass {
            let tranche = project.milestones[index].amount.0;
            let available = project.raised.0.saturating_sub(project.paid.0);
            assert!(available >= tranche, "Project has not raised enough for this tranche");

            project.paid = U128(project.paid.0 + tranche);
            project.milestone_index += 1;
            project.evidence_hash = None;

            let rep = self.reputation.get(&project.owner).unwrap_or(0);
            self.reputation.insert(&project.owner, &(rep + 1));

            if (project.milestone_index as usize) >= project.milestones.len() {
                project.status = "complete".to_string();
                let surplus = project.raised.0.saturating_sub(project.paid.0);
                self.pool_balance += surplus;
            }

            let recipient = project.owner.clone();
            self.projects.insert(&project_id, &project);
            Promise::new(recipient).transfer(NearToken::from_yoctonear(tranche));
        } else {
            let remaining = project.raised.0.saturating_sub(project.paid.0);
            self.pool_balance += remaining;
            project.status = "failed".to_string();
            project.failure_reason = Some("Milestone rejected by the demo resolver".to_string());
            project.evidence_hash = None;
            self.projects.insert(&project_id, &project);
        }
    }

    pub fn get_pool(&self) -> U128 {
        U128(self.pool_balance)
    }

    pub fn get_project(&self, project_id: u64) -> Option<Project> {
        self.projects.get(&project_id)
    }

    pub fn get_projects(&self) -> Vec<Project> {
        self.project_ids.iter().filter_map(|id| self.projects.get(id)).collect()
    }

    pub fn get_reputation(&self, account: AccountId) -> u64 {
        self.reputation.get(&account).unwrap_or(0)
    }
}
