import { useEffect, useRef } from "react"
import { ScrollArea } from "@/components/ui/scroll-area"
import { useSessionLogs } from "@/lib/sse"

interface SessionLogStreamProps {
  sessionId: string
}

export function SessionLogStream({ sessionId }: SessionLogStreamProps) {
  const { logs } = useSessionLogs(sessionId)
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  return (
    <ScrollArea className="h-[400px] rounded-md border bg-black/50 p-4" ref={scrollRef}>
      <pre className="font-mono text-sm text-green-400 whitespace-pre-wrap">
        {logs || "Waiting for output..."}
      </pre>
    </ScrollArea>
  )
}
