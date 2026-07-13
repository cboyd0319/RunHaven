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
      <div className="flex flex-col gap-1.5">
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
      </div>

      <HelpSection title="Start with recommended settings">
        <ol className="list-decimal space-y-2 pl-5">
          <li>Open New session and choose the assistant account you already use.</li>
          <li>Choose the folder that contains your project.</li>
          <li>Keep the recommended protected copy and network settings.</li>
          <li>Choose Start session, review the Mac dialog, then confirm.</li>
        </ol>
      </HelpSection>

      <HelpSection title="Choose an assistant">
        <div className="space-y-2">
          <p>
            Claude, Codex, Antigravity, and Copilot use generative AI. They can
            make mistakes. Review their output and file changes before you use
            them.
          </p>
          <p>
            When an assistant can use the internet, it can send your messages and
            project content it reads to the provider’s services. The provider
            handles that data under your account and the provider’s terms.
            RunHaven does not control whether the provider stores this content
            or uses it for training. That depends on your account settings and
            the provider’s terms.
          </p>
          <p>
            Choose an assistant based on the account you already have. You can go
            back before anything starts.
          </p>
        </div>
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
      <h2 className="font-heading text-base font-semibold">{title}</h2>
      <div className="max-w-2xl leading-relaxed text-muted-foreground">
        {children}
      </div>
    </section>
  )
}
