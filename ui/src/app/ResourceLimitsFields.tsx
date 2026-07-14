import type { RunPlanRequest } from "@/commands/runhaven"
import { Input } from "@/components/ui/input"

const CPU_PATTERN = String.raw`[1-9][0-9]*(\.[0-9]+)?`
const MEMORY_PATTERN = String.raw`[1-9][0-9]*[KkMmGgTtPp]?`
const CPU_FORMAT = new RegExp(`^(?:${CPU_PATTERN})$`)
const MEMORY_FORMAT = new RegExp(`^(?:${MEMORY_PATTERN})$`)

export function ResourceLimitsFields({
  cpus,
  memory,
  onChange,
}: {
  cpus: string
  memory: string
  onChange: (patch: Partial<RunPlanRequest>) => void
}) {
  const cpuInvalid = !CPU_FORMAT.test(cpus)
  const memoryInvalid = !MEMORY_FORMAT.test(memory)

  return (
    <div className="grid gap-3 sm:grid-cols-2">
      <div className="flex flex-col gap-1.5">
        <label htmlFor="runhaven-cpu-limit" className="text-sm font-medium">
          CPU cores
        </label>
        <Input
          id="runhaven-cpu-limit"
          inputMode="decimal"
          pattern={CPU_PATTERN}
          required
          title="Enter a CPU count such as 4 or 2.5."
          aria-describedby={cpuInvalid ? "cpu-help cpu-error" : "cpu-help"}
          aria-invalid={cpuInvalid || undefined}
          value={cpus}
          onChange={(event) => onChange({ cpus: event.currentTarget.value })}
        />
        <span
          id="cpu-help"
          className="text-xs font-normal text-muted-foreground"
        >
          Use a number such as 4 or 2.5.
        </span>
        {cpuInvalid ? (
          <span
            id="cpu-error"
            className="text-xs font-normal text-error-text"
          >
            Enter CPU cores as a number, such as 4 or 2.5.
          </span>
        ) : null}
      </div>
      <div className="flex flex-col gap-1.5">
        <label htmlFor="runhaven-memory-limit" className="text-sm font-medium">
          Memory
        </label>
        <Input
          id="runhaven-memory-limit"
          inputMode="text"
          pattern={MEMORY_PATTERN}
          required
          title="Enter memory such as 4g for 4 GB or 4096m."
          aria-describedby={
            memoryInvalid ? "memory-help memory-error" : "memory-help"
          }
          aria-invalid={memoryInvalid || undefined}
          value={memory}
          onChange={(event) => onChange({ memory: event.currentTarget.value })}
        />
        <span
          id="memory-help"
          className="text-xs font-normal text-muted-foreground"
        >
          Use a number and unit, such as 4g or 4096m.
        </span>
        {memoryInvalid ? (
          <span
            id="memory-error"
            className="text-xs font-normal text-error-text"
          >
            Enter a value such as 4g or 4096m.
          </span>
        ) : null}
      </div>
    </div>
  )
}
