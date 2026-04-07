import { SessionTable } from "@/components/sessions/SessionTable"
import { useSessions } from "@/hooks/useSessions"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function SessionsPage() {
  const { data, isLoading } = useSessions()
  const sessions = data?.sessions ?? []

  return (
    <div className="space-y-4 md:space-y-6">
      <div>
        <h1 className="text-xl font-bold tracking-tight md:text-2xl">Sessions</h1>
        <p className="text-sm text-muted-foreground">
          View and manage agent sessions.
        </p>
      </div>

      <Card>
        <CardHeader className="px-4 md:px-6">
          <CardTitle className="text-base md:text-lg">All Sessions</CardTitle>
        </CardHeader>
        <CardContent className="px-4 md:px-6">
          {isLoading ? (
            <p className="py-8 text-center text-muted-foreground">Loading...</p>
          ) : (
            <SessionTable sessions={sessions} />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
