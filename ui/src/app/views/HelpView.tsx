import type { ReactNode } from "react"

import { Button } from "@/components/ui/button"

export function HelpView({
  backLabel,
  onBack,
}: {
  backLabel: string
  onBack: () => void
}) {
  return (
    <article className="mx-auto flex w-full max-w-3xl flex-col gap-6 py-1 sm:py-3">
      <header className="flex flex-col gap-1.5">
        <h1 className="font-heading text-3xl leading-none font-semibold tracking-[-0.025em]">
          RunHaven Help
        </h1>
        <p className="max-w-2xl text-base leading-relaxed text-muted-foreground">
          Run coding assistants without giving them unrestricted access to your
          Mac.
        </p>
        <Button
          type="button"
          variant="outline"
          className="mt-2 self-start"
          onClick={onBack}
        >
          Back to {backLabel}
        </Button>
      </header>

      <HelpSection title="Start safely">
        <ol className="list-decimal space-y-2 pl-5">
          <li>Open Home and choose the assistant account you already use.</li>
          <li>Choose the folder that contains your project.</li>
          <li>Keep the recommended protected copy and network settings.</li>
          <li>Choose Start session, review the Mac dialog, then confirm.</li>
        </ol>
      </HelpSection>

      <HelpSection title="Choose an assistant">
        <p>
          Choose Claude, Codex, Antigravity, or Copilot based on the account you
          already have. You can go back or cancel before anything starts.
        </p>
      </HelpSection>

      <HelpSection title="Find a previous session">
        <p>
          Open History to see sessions that need you and sessions that have
          ended. Select a session to read its status, output, and next action.
        </p>
      </HelpSection>

      <HelpSection title="Fix a setup problem">
        <p>
          Open System when RunHaven cannot start a session. It shows the
          problem, what it affects, and the safest available next action.
        </p>
      </HelpSection>

      <HelpSection title="How RunHaven protects your project">
        <ul className="list-disc space-y-2 pl-5">
          <li>A protected copy keeps assistant changes separate for review.</li>
          <li>Assistant services only is the default network setting.</li>
          <li>
            Your Mac home folder, browser profiles, and raw credentials stay out.
          </li>
          <li>Less-safe choices show a warning and require explicit approval.</li>
        </ul>
      </HelpSection>
    </article>
  )
}

function HelpSection({
  title,
  children,
}: {
  title: string
  children: ReactNode
}) {
  return (
    <section className="flex flex-col gap-2">
      <h2 className="font-heading text-xl font-semibold">{title}</h2>
      <div className="max-w-2xl leading-relaxed text-muted-foreground">
        {children}
      </div>
    </section>
  )
}
