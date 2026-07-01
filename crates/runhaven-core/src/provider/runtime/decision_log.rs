use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::provider::egress::{ProxyDecision, ThreadedAllowlistProxy};
use crate::provider::observability::write_provider_policy_log;
use crate::runtime::plans::AgentRunPlan;

pub(super) type DecisionKey = (String, u16, String, String, String);
pub(super) type DecisionCounts = BTreeMap<DecisionKey, usize>;

pub(super) struct ProviderDecisionLogStream {
    shutdown: Arc<AtomicBool>,
    handle: thread::JoinHandle<Result<()>>,
}

impl ProviderDecisionLogStream {
    pub(super) fn stop(self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);
        self.handle
            .join()
            .map_err(|_| anyhow::anyhow!("provider decision log stream panicked"))?
    }
}

pub(super) fn start_provider_decision_log_stream(
    plan: &AgentRunPlan,
    run_id: &str,
    proxy: ThreadedAllowlistProxy,
) -> ProviderDecisionLogStream {
    let plan = plan.clone();
    let run_id = run_id.to_string();
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let handle = thread::spawn(move || {
        let mut seen = DecisionCounts::new();
        while !thread_shutdown.load(Ordering::Relaxed) {
            flush_provider_decision_deltas(&plan, &run_id, &proxy, &mut seen)?;
            thread::sleep(Duration::from_millis(250));
        }
        flush_provider_decision_deltas(&plan, &run_id, &proxy, &mut seen)
    });
    ProviderDecisionLogStream { shutdown, handle }
}

fn flush_provider_decision_deltas(
    plan: &AgentRunPlan,
    run_id: &str,
    proxy: &ThreadedAllowlistProxy,
    seen: &mut DecisionCounts,
) -> Result<()> {
    let deltas = provider_decision_deltas(&proxy.policy_decisions(), seen);
    write_provider_policy_log(plan, &deltas, run_id)
}

pub(super) fn provider_decision_deltas(
    decisions: &[ProxyDecision],
    seen: &mut DecisionCounts,
) -> Vec<ProxyDecision> {
    decisions
        .iter()
        .filter_map(|decision| {
            let key = (
                decision.host.clone(),
                decision.port,
                decision.decision.clone(),
                decision.reason.clone(),
                decision.matched_rule.clone(),
            );
            let previous = seen.get(&key).copied().unwrap_or_default();
            if decision.count <= previous {
                return None;
            }
            seen.insert(key, decision.count);
            let mut delta = decision.clone();
            delta.count -= previous;
            Some(delta)
        })
        .collect()
}
