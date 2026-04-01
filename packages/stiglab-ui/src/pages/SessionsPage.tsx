import { SessionTable } from "@/components/sessions/SessionTable"
import { useSessions } from "@/hooks/useSessions"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function SessionsPage() {
  const { data, isLoading } = useSessions()
  const sessions = data?.sessions ?? []

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Sessions</h1>
        <p className="text-muted-foreground">
          View and manage agent sessions.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">All Sessions</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-muted-foreground py-8 text-center">Loading...</p>
          ) : (
            <SessionTable sessions={sessions} />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
