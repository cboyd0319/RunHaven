import type {
  AgentProfile,
  AgentReadiness,
  RunPlanRequest,
} from "@/commands/runhaven";
import { Check } from "lucide-react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

import { projectAccessLabel, projectName } from "./launchSummary";
import { agentLabel } from "./sessionIdentity";
import type { LaunchStep } from "./types";

type NewSessionProgressProps = {
  step: LaunchStep;
  request: RunPlanRequest;
  selectedAgent: AgentProfile | undefined;
  readiness: AgentReadiness | undefined;
  onEditAgent: () => void;
  onEditProject: () => void;
};

const STEP_INDEX: Record<LaunchStep, number> = {
  agent: 0,
  project: 1,
  review: 2,
};

export function NewSessionProgress({
  step,
  request,
  selectedAgent,
  readiness,
  onEditAgent,
  onEditProject,
}: NewSessionProgressProps) {
  const current = STEP_INDEX[step];
  const steps = [
    {
      label: "Choose assistant",
      summary: selectedAgent
        ? `Complete · ${agentLabel(selectedAgent.name)}${readiness ? ` · ${readiness.label}` : ""}`
        : "Choose the assistant you use.",
      onEdit: onEditAgent,
      editLabel: "Edit assistant",
    },
    {
      label: "Choose project",
      summary: request.workspacePath
        ? `Complete · ${projectName(request.workspacePath)} · ${projectAccessLabel(request)}`
        : "Choose the project folder for this session.",
      onEdit: onEditProject,
      editLabel: "Edit project",
    },
    {
      label: "Review and start",
      summary: "Confirm access before anything starts.",
      onEdit: null,
      editLabel: "",
    },
  ];

  return (
    <ol aria-label="Session setup progress" className="@container grid gap-1">
      {steps.map((item, index) => {
        const state =
          index < current
            ? "complete"
            : index === current
              ? "current"
              : "upcoming";
        return (
          <li
            key={item.label}
            aria-current={state === "current" ? "step" : undefined}
            data-step-state={state}
            className={cn(
              "grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-center gap-3 rounded-lg border-l-2 px-3 py-2.5 transition-colors @[34rem]:grid-cols-[auto_minmax(0,1fr)_auto]",
              state === "current" && "border-primary",
              state === "complete" && "border-transparent",
              state === "upcoming" &&
                "border-transparent text-muted-foreground",
            )}
          >
            <span
              aria-hidden="true"
              className={cn(
                "flex size-6 shrink-0 items-center justify-center rounded-full border text-xs font-medium",
                state === "current" && "border-primary text-primary",
                state === "complete" && "border-success/40 text-success",
                state === "upcoming" && "border-border",
              )}
            >
              {state === "complete" ? (
                <Check className="size-3.5" />
              ) : (
                index + 1
              )}
            </span>
            <span className="min-w-0">
              <span className="block text-base font-medium text-foreground">
                {item.label}
              </span>
              <span className="block break-words text-sm text-muted-foreground">
                {item.summary}
              </span>
            </span>
            {state === "complete" && item.onEdit ? (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                aria-label={item.editLabel}
                className="col-start-2 justify-self-start @[34rem]:col-start-3 @[34rem]:row-start-1 @[34rem]:justify-self-end"
                onClick={item.onEdit}
              >
                {item.editLabel}
              </Button>
            ) : null}
          </li>
        );
      })}
    </ol>
  );
}
