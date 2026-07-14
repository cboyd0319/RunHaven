import type { AgentProfile, RunPlanRequest } from "@/commands/runhaven";
import { CircleAlert, ShieldCheck } from "lucide-react";

import { authScopeLabels, networkLabels } from "./labels";
import { projectAccessLabel, projectName } from "./launchSummary";
import { agentLabel } from "./sessionIdentity";

export function LaunchReview({
  request,
  selectedAgent,
}: {
  request: RunPlanRequest;
  selectedAgent: AgentProfile | undefined;
}) {
  const assistant = selectedAgent
    ? agentLabel(selectedAgent.name)
    : "This assistant";
  const project = request.workspacePath
    ? projectName(request.workspacePath)
    : "this project";
  const originalFilesChange = !request.worktree && !request.readOnlyWorkspace;

  return (
    <div className="flex flex-col gap-5">
      <section
        aria-labelledby="launch-boundary-summary"
        className="flex items-start gap-3 border-y py-4"
      >
        {originalFilesChange ? (
          <CircleAlert
            className="mt-0.5 size-5 shrink-0 text-warning"
            aria-hidden
          />
        ) : (
          <ShieldCheck
            className="mt-0.5 size-5 shrink-0 text-primary"
            aria-hidden
          />
        )}
        <div className="grid gap-1">
          <h3 id="launch-boundary-summary" className="text-base font-semibold">
            Review this session's access
          </h3>
          <p className="text-base leading-relaxed text-muted-foreground">
            {boundarySummary(request, assistant, project)}
          </p>
        </div>
      </section>

      <dl className="divide-y text-base">
        <SummaryRow label="Assistant" value={assistant} />
        <SummaryRow label="Project" value={project} />
        <SummaryRow
          label="Project access"
          value={projectAccessLabel(request)}
        />
        <SummaryRow
          label="Network access"
          value={networkLabels[request.networkMode]}
        />
        <SummaryRow
          label="Sign-in sharing"
          value={authScopeLabels[request.authScope]}
        />
        {request.apiKeyBrokerEnv ? (
          <SummaryRow
            label="API key access"
            value={`${request.apiKeyBrokerEnv} from this Mac`}
          />
        ) : null}
      </dl>

      <p className="text-sm leading-relaxed text-muted-foreground">
        A Mac confirmation will show this access again before anything starts.
      </p>
    </div>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[minmax(8rem,0.45fr)_minmax(0,1fr)] gap-4 py-2.5">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="min-w-0 text-right font-medium">{value}</dd>
    </div>
  );
}

function boundarySummary(
  request: RunPlanRequest,
  assistant: string,
  project: string,
): string {
  if (request.worktree) {
    return `RunHaven will give ${assistant} a protected copy of ${project}. Your original files stay unchanged while the assistant works.`;
  }
  if (request.readOnlyWorkspace) {
    return `RunHaven will let ${assistant} read ${project}. The assistant can’t change the original files.`;
  }
  return `RunHaven will let ${assistant} change the original files in ${project}. The Mac confirmation will warn you again.`;
}
