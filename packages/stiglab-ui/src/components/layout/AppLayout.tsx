import type { ReactNode } from "react"
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "./AppSidebar"
import { BottomNav } from "./BottomNav"
import { Separator } from "@/components/ui/separator"
import { ThemeToggle } from "./ThemeToggle"
import { Terminal } from "lucide-react"
import { Link } from "react-router-dom"

export function AppLayout({ children }: { children: ReactNode }) {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        {/* Mobile header */}
        <header className="flex h-12 items-center justify-between border-b px-4 md:hidden">
          <Link to="/" className="flex items-center gap-2">
            <Terminal className="h-5 w-5 text-blue-500" />
            <span className="text-base font-semibold">Stiglab</span>
          </Link>
          <ThemeToggle />
        </header>
        {/* Desktop header */}
        <header className="hidden h-14 items-center gap-2 border-b px-6 md:flex">
          <SidebarTrigger />
          <Separator orientation="vertical" className="h-6" />
        </header>
        <main className="flex-1 p-4 pb-20 md:p-6 md:pb-6">
          {children}
        </main>
      </SidebarInset>
      <BottomNav />
    </SidebarProvider>
  )
}
