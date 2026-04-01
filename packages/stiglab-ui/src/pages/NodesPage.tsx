import { NodeTable } from "@/components/nodes/NodeTable"
import { useNodes } from "@/hooks/useNodes"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function NodesPage() {
  const { data, isLoading } = useNodes()
  const nodes = data?.nodes ?? []

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Nodes</h1>
        <p className="text-muted-foreground">
          Manage registered agent nodes.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Registered Nodes</CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-muted-foreground py-8 text-center">Loading...</p>
          ) : (
            <NodeTable nodes={nodes} />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
