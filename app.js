import { connect, keyStores, utils } from "https://esm.sh/near-api-js@5.1.1";
import { setupWalletSelector, actionCreators } from "https://esm.sh/@near-wallet-selector/core@10.1.4";
import { setupMeteorWalletApp } from "https://esm.sh/@near-wallet-selector/meteor-wallet-app@10.1.4";
import { setupMeteorWallet } from "https://esm.sh/@near-wallet-selector/meteor-wallet@10.1.4";
import { setupMyNearWallet } from "https://esm.sh/@near-wallet-selector/my-near-wallet@10.1.4";
import { setupSender } from "https://esm.sh/@near-wallet-selector/sender@10.1.4";
import { setupHereWallet } from "https://esm.sh/@near-wallet-selector/here-wallet@10.1.4";
import { setupNightly } from "https://esm.sh/@near-wallet-selector/nightly@10.1.4";

// Replace this after deploying contract/src/lib.rs to NEAR testnet.
const CONTRACT_ID = String(window.DREAMFUND_CONTRACT_ID || "").trim();
const NETWORK = {
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.fastnear.com",
  walletUrl: "https://wallet.testnet.near.org",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://testnet.nearblocks.io"
};
const GAS = "30000000000000";
let near;
let selector;
let wallet;
let activeWalletId = null;
let accountId = null;
let resolverAccount = null;
let toastTimer;

const $ = (id) => document.getElementById(id);
const loader = $("loader-wrap");
const template = $("card-template");

function showLoader(message = "Working on chain…") {
  $("loader-text").textContent = message;
  loader.classList.remove("hidden");
}
function hideLoader() { loader.classList.add("hidden"); }
function toast(message) {
  const el = $("toast");
  el.textContent = message;
  el.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.remove("show"), 3600);
}
function nearToYocto(value) {
  const result = utils.format.parseNearAmount(String(value));
  if (!result) throw new Error("Enter a valid NEAR amount");
  return result;
}
function formatNear(value) {
  try { return (Number(BigInt(value || "0")) / 1e24).toLocaleString(undefined, { maximumFractionDigits: 3 }); }
  catch { return "0"; }
}
function ensureConfigured() {
  if (!CONTRACT_ID || CONTRACT_ID.startsWith("REPLACE_")) {
    throw new Error("DreamFund is waiting for its NEAR testnet contract deployment. The contract address will be loaded from config.js automatically.");
  }
}
function updateWalletUI() {
  $("btn-connect").textContent = accountId ? "Disconnect" : "Connect NEAR Wallet";
  $("wallet-account").textContent = accountId || "";
}
async function initWallet() {
  ensureConfigured();

  // Public reads must not depend on browser wallet storage or wallet-selector
  // initialization. An in-memory key store is sufficient for read-only RPC.
  try {
    near = await connect({
      ...NETWORK,
      keyStore: new keyStores.InMemoryKeyStore()
    });
    await refresh();
  } catch (error) {
    console.error("Public NEAR RPC initialization failed:", error);
    $("pool-balance").innerHTML = "Unavailable <small>NEAR</small>";
    $("project-count").textContent = "—";
    $("my-rep").textContent = "—";
    $("project-grid").innerHTML =
      '<div class="empty-state">Could not reach NEAR Testnet right now. Tap Refresh to retry. Wallet connection is a separate step.</div>';
    toast("NEAR Testnet data could not load: " + error.message);
  }

  // Wallet connection is initialized independently. A wallet module failure
  // must never prevent the public portfolio from loading.
  try {
    selector = await setupWalletSelector({
      network: NETWORK,
      modules: [
        setupMeteorWalletApp({ contractId: CONTRACT_ID }),
        setupMeteorWallet(),
        setupMyNearWallet({ walletUrl: "https://testnet.mynearwallet.com" }),
        setupSender(),
        setupHereWallet(),
        setupNightly()
      ]
    });
    const state = selector.store.getState();
    const savedAccounts = state.accounts || [];
    const selectedWalletId = state.selectedWalletId;
    if (savedAccounts.length && selectedWalletId) {
      wallet = await selector.wallet(selectedWalletId);
      activeWalletId = selectedWalletId;
      accountId = savedAccounts[0].accountId;
    }
  } catch (error) {
    console.warn("Wallet connectors unavailable in this browser:", error);
  }

  updateWalletUI();
  if (accountId && near) await refresh();
}
async function connectWallet() {
  if (!selector) throw new Error("Wallet connections are unavailable in this browser. You can still browse public projects.");
  if (accountId && wallet) {
    await wallet.signOut();
    wallet = null;
    activeWalletId = null;
    accountId = null;
    updateWalletUI();
    await refresh();
    return;
  }
  ensureConfigured();
  // Reuse the last wallet choice. When there is no previous choice, try
  // Meteor Wallet App first (the common path when DreamFund is opened inside
  // Meteor). Only show the wallet picker if that direct connection fails.
  const state = selector.store.getState();
  const preferredWalletId = state.selectedWalletId || "meteor-wallet-app";
  const choices = [
    ["Meteor Wallet App (open inside Meteor)", "meteor-wallet-app"],
    ["Meteor Wallet browser extension", "meteor-wallet"],
    ["My NEAR Wallet", "my-near-wallet"],
    ["Sender Wallet", "sender"],
    ["HERE Wallet", "here-wallet"],
    ["Nightly Wallet", "nightly"]
  ];
  const tryConnect = async (walletId) => {
    const candidate = await selector.wallet(walletId);
    const accounts = await candidate.signIn({ contractId: CONTRACT_ID, methodNames: [] });
    const selected = accounts?.[0] || (await candidate.getAccounts())[0];
    if (!selected?.accountId) throw new Error("The wallet did not return an account.");
    wallet = candidate;
    activeWalletId = walletId;
    accountId = selected.accountId;
  };

  try {
    await tryConnect(preferredWalletId);
  } catch (directError) {
    console.info("Preferred wallet connection did not complete; offering wallet choices.", directError);
    const menu = choices.map(([label], i) => `${i + 1}. ${label}`).join("\n");
    const answer = prompt("Meteor could not connect automatically. Choose your NEAR wallet:\n\n" + menu + "\n\nEnter the wallet number:");
    if (answer === null) return;
    const choice = Number(answer);
    if (!Number.isInteger(choice) || choice < 1 || choice > choices.length) {
      throw new Error("Choose a wallet by entering a number from 1 to " + choices.length + ".");
    }
    await tryConnect(choices[choice - 1][1]);
  }
  updateWalletUI();
  await refresh();
}
async function view(methodName, args = {}) {
  const provider = near.connection.provider;
  return provider.query({
    request_type: "call_function",
    account_id: CONTRACT_ID,
    method_name: methodName,
    args_base64: btoa(unescape(encodeURIComponent(JSON.stringify(args)))),
    finality: "optimistic"
  }).then((r) => JSON.parse(Bufferless.decode(r.result)));
}
const Bufferless = {
  decode(bytes) {
    const decoded = new TextDecoder().decode(new Uint8Array(bytes));
    return decoded;
  }
};
async function call(methodName, args = {}, deposit = "0") {
  ensureConfigured();
  if (!wallet || !accountId) throw new Error("Connect your Meteor testnet wallet first");
  return wallet.signAndSendTransaction({
    signerId: accountId,
    receiverId: CONTRACT_ID,
    actions: [actionCreators.functionCall(methodName, args, BigInt(GAS), BigInt(deposit))]
  });
}
async function refresh() {
  if (!CONTRACT_ID || CONTRACT_ID.startsWith("REPLACE_")) {
    $("pool-balance").innerHTML = "Not deployed <small>yet</small>";
    $("project-count").textContent = "—";
    $("my-rep").textContent = "—";
    $("project-grid").innerHTML = '<div class="empty-state">The NEAR testnet contract has not been deployed yet.</div>';
    return;
  }
  try {
    // Re-establish the public RPC connection on manual refresh if startup
    // encountered a transient RPC or browser-storage problem.
    if (!near) {
      near = await connect({
        ...NETWORK,
        keyStore: new keyStores.InMemoryKeyStore()
      });
    }
    const [pool, projects, resolver] = await Promise.all([
      view("get_pool"),
      view("get_projects"),
      view("get_owner")
    ]);
    resolverAccount = resolver;
    $("pool-balance").innerHTML = `${formatNear(pool)} <small>NEAR</small>`;
    $("project-count").textContent = projects.length;
    $("my-rep").textContent = Boolean(accountId)
      ? await view("get_reputation", { account: accountId })
      : "—";
    await renderProjects(projects);
  } catch (error) {
    console.error("Could not refresh public contract data:", error);
    if (!near) {
      $("pool-balance").innerHTML = "Unavailable <small>NEAR</small>";
      $("project-count").textContent = "—";
    }
    $("project-grid").innerHTML = '<div class="empty-state">Could not load NEAR Testnet data. Check your connection and tap Refresh to retry. This does not mean your wallet or funds are disconnected.</div>';
    toast("Could not load contract data: " + error.message);
  }
}
async function renderProjects(projects) {
  const grid = $("project-grid");
  grid.innerHTML = "";
  if (!projects.length) {
    grid.innerHTML = '<div class="empty-state">No projects yet. Be the first to propose a public good.</div>';
    return;
  }
  for (const project of projects) {
    const node = template.content.firstElementChild.cloneNode(true);
    const card = node.querySelector(".project-card");
    card.dataset.id = project.id;
    node.querySelector(".project-id").textContent = "PROJECT #" + String(project.id).padStart(2, "0");
    node.querySelector(".project-title").textContent = project.title;
    node.querySelector(".project-meta").textContent = `${formatNear(project.raised)} / ${formatNear(project.goal)} NEAR raised · ${formatNear(project.paid)} paid`;
    const badge = node.querySelector(".status-badge");
    badge.textContent = project.status.toUpperCase().replace("_", " ");
    const pct = Math.min(100, Number(BigInt(project.raised || "0") * 100n / BigInt(project.goal || "1")));
    node.querySelector(".progress-fill").style.width = pct + "%";
    node.querySelector(".milestones").innerHTML = project.milestones.map((m, i) =>
      `<div class="milestone-row"><span>${escapeHtml(m.label)}</span><strong>${formatNear(m.amount)} N</strong></div>`
    ).join("");
    node.querySelector(".evidence-display").textContent = project.evidence_hash ? "Evidence: " + project.evidence_hash : "No evidence submitted for current milestone";
    node.querySelector(".owner-line").textContent = "Founder: " + project.owner;
    const founderRep = await view("get_reputation", { account: project.owner });
    node.querySelector(".reputation-line").textContent = "Verified deliveries: " + founderRep;
    const isOwner = Boolean(accountId) && accountId === project.owner;
    const isFounder = isOwner;
    const isResolver = Boolean(accountId) && accountId === resolverAccount;
    const voteCounts = await view("get_milestone_votes", { project_id: Number(project.id) });
    const myVote = Boolean(accountId) ? await view("get_my_milestone_vote", { project_id: Number(project.id), voter: accountId }) : null;
    node.querySelector(".btn-pledge").onclick = () => pledge(project.id);
    node.querySelector(".btn-allocate").onclick = () => allocateFromPool(project.id);
    node.querySelector(".btn-evidence").onclick = () => submitEvidence(project.id);
    node.querySelector(".btn-vote-yes").onclick = () => voteMilestone(project.id, true);
    node.querySelector(".btn-vote-no").onclick = () => voteMilestone(project.id, false);
    node.querySelector(".vote-display").textContent = `Community votes: ${voteCounts[0]} approve · ${voteCounts[1]} reject · ${voteCounts[2]}/5 cast`;
    node.querySelector(".btn-evidence").hidden = !isOwner || project.status !== "in_progress";
    const votingOpen = project.status === "in_progress" && Boolean(project.evidence_hash) && !isFounder && Boolean(accountId) && voteCounts[2] < 5 && myVote === null;
    node.querySelector(".btn-vote-yes").hidden = !votingOpen;
    node.querySelector(".btn-vote-no").hidden = !votingOpen;
    if (myVote !== null) node.querySelector(".vote-display").textContent += myVote ? " · You voted approve" : " · You voted reject";
    node.querySelector(".btn-pledge").hidden = !Boolean(accountId) || !["funding", "in_progress"].includes(project.status);
    node.querySelector(".btn-allocate").hidden = !isResolver || !["funding", "in_progress"].includes(project.status);
    node.querySelector(".explorer-link").href = NETWORK.explorerUrl + "/address/" + encodeURIComponent(CONTRACT_ID);
    grid.appendChild(node);
  }
}
function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, (c) => ({ "&":"&amp;", "<":"&lt;", ">":"&gt;", '"':"&quot;", "'":"&#39;" }[c]));
}
async function fundPool() {
  const amount = prompt("How much NEAR would you like to add to the shared community pool?");
  if (!amount) return;
  try {
    showLoader("Adding funds to the community pool…");
    const tx = await call("fund_pool", {}, nearToYocto(amount));
    toast("Community pool funded · " + tx.transaction.hash.slice(0, 10) + "…");
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
}
async function allocateFromPool(id) {
  const amount = prompt("How much NEAR should the community pool allocate to this project?");
  if (!amount) return;
  try {
    showLoader("Allocating community funds…");
    const tx = await call("allocate_from_pool", { project_id: Number(id), amount: nearToYocto(amount) });
    toast("Pool allocation confirmed · " + tx.transaction.hash.slice(0, 10) + "…");
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
}
async function pledge(id) {
  const amount = prompt("How much NEAR would you like to pledge?");
  if (!amount) return;
  try {
    showLoader("Sending your pledge to NEAR…");
    const tx = await call("pledge", { project_id: Number(id) }, nearToYocto(amount));
    toast("Pledge confirmed · " + tx.transaction.hash.slice(0, 10) + "…");
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
}
async function submitEvidence(id) {
  const raw = prompt("Paste a proof hash, URL, or short evidence note:");
  if (!raw) return;
  try {
    let hash = raw.trim();
    if (!/^0x[a-fA-F0-9]{64}$/.test(hash) && !/^[a-fA-F0-9]{64}$/.test(hash)) {
      const bytes = new TextEncoder().encode(hash);
      const digest = await crypto.subtle.digest("SHA-256", bytes);
      hash = "0x" + Array.from(new Uint8Array(digest), b => b.toString(16).padStart(2, "0")).join("");
    }
    showLoader("Anchoring evidence hash…");
    const tx = await call("submit_evidence", { project_id: Number(id), evidence_hash: hash });
    toast("Evidence submitted · " + tx.transaction.hash.slice(0, 10) + "…");
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
}
async function voteMilestone(id, approve) {
  const choice = approve ? "APPROVE this milestone" : "REJECT this milestone";
  if (!confirm(`Cast your one-time community vote to ${choice}? Five eligible accounts are required to decide.`)) return;
  try {
    showLoader("Recording your community vote…");
    const tx = await call("vote_milestone", { project_id: Number(id), approve });
    toast("Vote recorded on NEAR · " + tx.transaction.hash.slice(0, 10) + "…");
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
}
$("create-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const title = $("title").value.trim();
    const goal = Number($("goal").value);
    const m1Amount = Number($("m1-amount").value);
    const m2Amount = Number($("m2-amount").value || 0);
    if (m1Amount + m2Amount > goal) throw new Error("Milestone amounts cannot exceed the goal");
    const milestones = [{ label: $("m1-label").value.trim(), amount: nearToYocto(m1Amount) }];
    if ($("m2-label").value.trim() && m2Amount > 0) milestones.push({ label: $("m2-label").value.trim(), amount: nearToYocto(m2Amount) });
    showLoader("Creating project on NEAR…");
    const tx = await call("create_project", { title, goal: nearToYocto(goal), milestones });
    toast("Project created · " + tx.transaction.hash.slice(0, 10) + "…");
    event.target.reset();
    $("m2-amount").value = "0";
    await refresh();
  } catch (e) { toast(e.message); }
  finally { hideLoader(); }
});
$("btn-connect").addEventListener("click", () => connectWallet().catch(e => toast(e.message)));
$("btn-fund-pool").addEventListener("click", () => fundPool());
$("btn-refresh").addEventListener("click", () => refresh());
initWallet().catch((e) => { console.error(e); toast("Wallet setup failed: " + e.message); });
