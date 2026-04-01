import { useParams } from "react-router-dom"
import { useSession } from "@/hooks/useSessions"
import { SessionStateBadge } from "@/components/sessions/SessionStateBadge"
import { SessionLogStream } from "@/components/sessions/SessionLogStream"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"
import { formatDistanceToNow } from "@/lib/utils"

export function SessionDetailPage() {
  const { id } = useParams<{ id: string }>()
  const { data, isLoading } = useSession(id!)

  if (isLoading) {
    return <p className="text-muted-foreground py-8 text-center">Loading...</p>
  }

  const session = data?.session
  if (!session) {
    return <p className="text-muted-foreground py-8 text-center">Session not found</p>
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <h1 className="text-2xl font-bold tracking-tight font-mono">
          {session.id.slice(0, 8)}
        </h1>
        <SessionStateBadge state={session.state} />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Session Details</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Task ID:</span>
              <p className="font-mono">{session.task_id.slice(0, 8)}</p>
            </div>
            <div>
              <span className="text-muted-foreground">Node ID:</span>
              <p className="font-mono">{session.node_id.slice(0, 8)}</p>
            </div>
            <div>
              <span className="text-muted-foreground">Created:</span>
              <p>{formatDistanceToNow(session.created_at)}</p>
            </div>
            <div>
              <span className="text-muted-foreground">Updated:</span>
              <p>{formatDistanceToNow(session.updated_at)}</p>
            </div>
          </div>

          <Separator />

          <div>
            <span className="text-sm text-muted-foreground">Prompt:</span>
            <p className="mt-1 rounded-md bg-muted p-3 text-sm">{session.prompt}</p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Output</CardTitle>
        </CardHeader>
        <CardContent>
          <SessionLogStream sessionId={session.id} />
        </CardContent>
      </Card>
    </div>
  )
}
