import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useAuth } from "@/lib/auth"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Trash2, Plus, KeyRound, User } from "lucide-react"

const KNOWN_CREDENTIALS = [
  {
    name: "CLAUDE_CODE_OAUTH_TOKEN",
    description: "OAuth token for Claude Code CLI authentication",
  },
  {
    name: "ANTHROPIC_API_KEY",
    description: "Anthropic API key for direct API access",
  },
]

export function SettingsPage() {
  const { user, authEnabled } = useAuth()
  const queryClient = useQueryClient()
  const [newCredName, setNewCredName] = useState("")
  const [newCredValue, setNewCredValue] = useState("")
  const [editingCred, setEditingCred] = useState<string | null>(null)
  const [editValue, setEditValue] = useState("")

  const { data: credData } = useQuery({
    queryKey: ["credentials"],
    queryFn: api.getCredentials,
  })

  const setCred = useMutation({
    mutationFn: ({ name, value }: { name: string; value: string }) =>
      api.setCredential(name, value),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["credentials"] })
      setNewCredName("")
      setNewCredValue("")
      setEditingCred(null)
      setEditValue("")
    },
  })

  const deleteCred = useMutation({
    mutationFn: (name: string) => api.deleteCredential(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["credentials"] })
    },
  })

  const credentials = credData?.credentials ?? []
  const existingNames = new Set(credentials.map((c) => c.name))

  return (
    <div className="space-y-4 md:space-y-6">
      <div>
        <h1 className="text-xl font-bold tracking-tight md:text-2xl">Settings</h1>
        <p className="text-sm text-muted-foreground">
          Manage your profile and credentials.
        </p>
      </div>

      {/* Profile — only show when auth is enabled (not anonymous) */}
      {authEnabled && user && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base md:text-lg">
              <User className="h-4 w-4" />
              Profile
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-4">
              {user.github_avatar_url && (
                <img
                  src={user.github_avatar_url}
                  alt={user.github_login}
                  className="h-12 w-12 rounded-full"
                />
              )}
              <div>
                <p className="font-medium">{user.github_name ?? user.github_login}</p>
                <p className="text-sm text-muted-foreground">@{user.github_login}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Credentials */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base md:text-lg">
            <KeyRound className="h-4 w-4" />
            Credentials
          </CardTitle>
          <CardDescription>
            Credentials are encrypted and passed to agent sessions as environment variables.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Existing credentials */}
          {credentials.map((cred) => (
            <div
              key={cred.name}
              className="flex items-center gap-2 rounded-md border p-3"
            >
              <div className="flex-1">
                <p className="font-mono text-sm font-medium">{cred.name}</p>
                <p className="text-xs text-muted-foreground">
                  Updated {new Date(cred.updated_at).toLocaleDateString()}
                </p>
              </div>
              {editingCred === cred.name ? (
                <div className="flex items-center gap-2">
                  <Input
                    type="password"
                    placeholder="New value"
                    value={editValue}
                    onChange={(e) => setEditValue(e.target.value)}
                    className="w-48"
                  />
                  <Button
                    size="sm"
                    onClick={() =>
                      setCred.mutate({ name: cred.name, value: editValue })
                    }
                    disabled={!editValue || setCred.isPending}
                  >
                    Save
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      setEditingCred(null)
                      setEditValue("")
                    }}
                  >
                    Cancel
                  </Button>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      setEditingCred(cred.name)
                      setEditValue("")
                    }}
                  >
                    Update
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => deleteCred.mutate(cred.name)}
                    disabled={deleteCred.isPending}
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </div>
              )}
            </div>
          ))}

          {/* Quick-add known credentials */}
          {KNOWN_CREDENTIALS.filter((k) => !existingNames.has(k.name)).length >
            0 && (
            <div className="space-y-2">
              <p className="text-sm font-medium text-muted-foreground">
                Add credential
              </p>
              {KNOWN_CREDENTIALS.filter((k) => !existingNames.has(k.name)).map(
                (known) => (
                  <div
                    key={known.name}
                    className="flex items-center gap-2 rounded-md border border-dashed p-3"
                  >
                    <div className="flex-1">
                      <p className="font-mono text-sm">{known.name}</p>
                      <p className="text-xs text-muted-foreground">
                        {known.description}
                      </p>
                    </div>
                    {editingCred === `new-${known.name}` ? (
                      <div className="flex items-center gap-2">
                        <Input
                          type="password"
                          placeholder="Value"
                          value={editValue}
                          onChange={(e) => setEditValue(e.target.value)}
                          className="w-48"
                        />
                        <Button
                          size="sm"
                          onClick={() =>
                            setCred.mutate({
                              name: known.name,
                              value: editValue,
                            })
                          }
                          disabled={!editValue || setCred.isPending}
                        >
                          Save
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => {
                            setEditingCred(null)
                            setEditValue("")
                          }}
                        >
                          Cancel
                        </Button>
                      </div>
                    ) : (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => {
                          setEditingCred(`new-${known.name}`)
                          setEditValue("")
                        }}
                      >
                        <Plus className="mr-1 h-3 w-3" />
                        Add
                      </Button>
                    )}
                  </div>
                )
              )}
            </div>
          )}

          {/* Custom credential */}
          <div className="space-y-2 border-t pt-4">
            <p className="text-sm font-medium text-muted-foreground">
              Custom credential
            </p>
            <div className="flex items-center gap-2">
              <Input
                placeholder="ENV_VAR_NAME"
                value={newCredName}
                onChange={(e) => setNewCredName(e.target.value.toUpperCase())}
                className="w-48"
              />
              <Input
                type="password"
                placeholder="Value"
                value={newCredValue}
                onChange={(e) => setNewCredValue(e.target.value)}
                className="flex-1"
              />
              <Button
                size="sm"
                onClick={() =>
                  setCred.mutate({ name: newCredName, value: newCredValue })
                }
                disabled={!newCredName || !newCredValue || setCred.isPending}
              >
                <Plus className="mr-1 h-3 w-3" />
                Add
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
