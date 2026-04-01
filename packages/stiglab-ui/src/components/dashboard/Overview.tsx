import { Server, Terminal, AlertCircle, CheckCircle } from "lucide-react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import type { Node, Session } from "@/lib/api"

interface OverviewProps {
  nodes: Node[]
  sessions: Session[]
}

export function Overview({ nodes, sessions }: OverviewProps) {
  const onlineNodes = nodes.filter((n) => n.status === "online").length
  const activeSessions = sessions.filter((s) =>
    ["running", "dispatched", "waiting_input"].includes(s.state)
  ).length
  const waitingInput = sessions.filter((s) => s.state === "waiting_input").length
  const completedSessions = sessions.filter((s) => s.state === "done").length

  const stats = [
    {
      title: "Nodes Online",
      value: `${onlineNodes}/${nodes.length}`,
      icon: Server,
      description: `${nodes.length - onlineNodes} offline`,
    },
    {
      title: "Active Sessions",
      value: activeSessions,
      icon: Terminal,
      description: "Currently running",
    },
    {
      title: "Waiting Input",
      value: waitingInput,
      icon: AlertCircle,
      description: "Needs attention",
      highlight: waitingInput > 0,
    },
    {
      title: "Completed",
      value: completedSessions,
      icon: CheckCircle,
      description: "Successfully done",
    },
  ]

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {stats.map((stat) => (
        <Card key={stat.title} className={stat.highlight ? "border-yellow-500/50" : ""}>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              {stat.title}
            </CardTitle>
            <stat.icon className={`h-4 w-4 ${stat.highlight ? "text-yellow-500" : "text-muted-foreground"}`} />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${stat.highlight ? "text-yellow-500" : ""}`}>
              {stat.value}
            </div>
            <p className="text-xs text-muted-foreground">{stat.description}</p>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
