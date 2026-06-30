import { CircleAlert, Network } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"

import type { DiagnosticsState } from "./types"

export function DiagnosticsCard({
  diagnostics,
  loading,
  error,
  onLoad,
}: {
  diagnostics: DiagnosticsState | null
  loading: boolean
  error: string
  onLoad: () => Promise<void>
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle role="heading" aria-level={2}>
          Trust checks
        </CardTitle>
        <CardDescription>
          Secret-free auth and network checks for RunHaven runs.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button type="button" variant="outline" onClick={() => void onLoad()}>
          {loading ? "Loading diagnostics..." : "Load diagnostics"}
        </Button>
        {error ? (
          <Alert variant="destructive">
            <CircleAlert />
            <AlertTitle>Diagnostics unavailable</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
        {diagnostics ? (
          <Tabs defaultValue="auth">
            <TabsList>
              <TabsTrigger value="auth">Auth</TabsTrigger>
              <TabsTrigger value="network">Network</TabsTrigger>
            </TabsList>
            <TabsContent value="auth" className="mt-3">
              <h2 className="mb-2 text-base font-medium">Auth broker status</h2>
              <div className="flex flex-col gap-2 text-sm">
                <p className="text-muted-foreground">
                  Runtime: {diagnostics.authStatus.runtime}
                </p>
                {diagnostics.authStatus.profiles.map((profile) => (
                  <p key={profile.name}>
                    {profile.name}: {profile.status}
                  </p>
                ))}
                {diagnostics.authLog.entries.length === 0 ? (
                  <p className="text-muted-foreground">
                    No auth broker decisions recorded.
                  </p>
                ) : null}
              </div>
            </TabsContent>
            <TabsContent value="network" className="mt-3">
              {diagnostics.egressLog.entries.length === 0 ? (
                <Empty>
                  <EmptyHeader>
                    <EmptyMedia variant="icon">
                      <Network />
                    </EmptyMedia>
                    <EmptyTitle>No provider egress decisions recorded.</EmptyTitle>
                    <EmptyDescription>
                      RunHaven has not recorded provider network decisions yet.
                    </EmptyDescription>
                  </EmptyHeader>
                </Empty>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Profile</TableHead>
                      <TableHead>Host</TableHead>
                      <TableHead>Decision</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {diagnostics.egressLog.entries.map((entry) => (
                      <TableRow key={`${entry.runId}-${entry.host}`}>
                        <TableCell>{entry.profile}</TableCell>
                        <TableCell>{entry.host}</TableCell>
                        <TableCell>{entry.decision}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </TabsContent>
          </Tabs>
        ) : null}
      </CardContent>
    </Card>
  )
}
