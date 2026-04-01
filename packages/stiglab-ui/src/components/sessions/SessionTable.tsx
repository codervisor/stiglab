import { Link } from "react-router-dom"
import type { Session } from "@/lib/api"
import { SessionStateBadge } from "./SessionStateBadge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { formatDistanceToNow } from "@/lib/utils"

interface SessionTableProps {
  sessions: Session[]
}

export function SessionTable({ sessions }: SessionTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>ID</TableHead>
          <TableHead>Node</TableHead>
          <TableHead>State</TableHead>
          <TableHead>Prompt</TableHead>
          <TableHead>Created</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sessions.length === 0 && (
          <TableRow>
            <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
              No sessions yet
            </TableCell>
          </TableRow>
        )}
        {sessions.map((session) => (
          <TableRow key={session.id}>
            <TableCell>
              <Link
                to={`/sessions/${session.id}`}
                className="font-mono text-sm text-blue-500 hover:underline"
              >
                {session.id.slice(0, 8)}
              </Link>
            </TableCell>
            <TableCell className="text-muted-foreground">{session.node_id.slice(0, 8)}</TableCell>
            <TableCell>
              <SessionStateBadge state={session.state} />
            </TableCell>
            <TableCell className="max-w-[300px] truncate text-sm">
              {session.prompt.slice(0, 80)}
            </TableCell>
            <TableCell className="text-muted-foreground text-sm">
              {formatDistanceToNow(session.created_at)}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
