import type { Node } from "@/lib/api"
import { NodeStatusBadge } from "./NodeStatusBadge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { formatDistanceToNow } from "@/lib/utils"

interface NodeTableProps {
  nodes: Node[]
}

export function NodeTable({ nodes }: NodeTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Hostname</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Sessions</TableHead>
          <TableHead>Last Heartbeat</TableHead>
          <TableHead>Registered</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {nodes.length === 0 && (
          <TableRow>
            <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
              No nodes registered
            </TableCell>
          </TableRow>
        )}
        {nodes.map((node) => (
          <TableRow key={node.id}>
            <TableCell className="font-medium">{node.name}</TableCell>
            <TableCell className="text-muted-foreground">{node.hostname}</TableCell>
            <TableCell>
              <NodeStatusBadge status={node.status} />
            </TableCell>
            <TableCell>
              <span className="font-mono text-sm">
                {node.active_sessions}/{node.max_sessions}
              </span>
            </TableCell>
            <TableCell className="text-muted-foreground text-sm">
              {formatDistanceToNow(node.last_heartbeat)}
            </TableCell>
            <TableCell className="text-muted-foreground text-sm">
              {formatDistanceToNow(node.registered_at)}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
