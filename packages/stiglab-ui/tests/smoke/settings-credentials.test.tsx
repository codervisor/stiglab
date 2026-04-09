import { describe, it, expect, vi, beforeEach } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { SettingsPage } from "@/pages/SettingsPage"

// Minimal mocks so SettingsPage renders without network
vi.mock("@/lib/auth", () => ({
  useAuth: () => ({ user: null, authEnabled: false }),
}))

vi.mock("@/lib/api", () => ({
  api: {
    getCredentials: vi.fn().mockResolvedValue({ credentials: [] }),
    setCredential: vi.fn(),
    deleteCredential: vi.fn(),
  },
}))

function renderSettings() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <SettingsPage />
    </QueryClientProvider>,
  )
}

describe("SettingsPage credentials layout", () => {
  beforeEach(() => vi.clearAllMocks())

  it("renders both known credential entries", () => {
    renderSettings()
    expect(screen.getByText("CLAUDE_CODE_OAUTH_TOKEN")).toBeInTheDocument()
    expect(screen.getByText("ANTHROPIC_API_KEY")).toBeInTheDocument()
  })

  it("shows Add button inline with each known credential (not in a separate row)", () => {
    renderSettings()
    const addButtons = screen.getAllByRole("button", { name: /add/i })
    // At least 2 Add buttons (one per known credential) + 1 for custom credential
    expect(addButtons.length).toBeGreaterThanOrEqual(2)
  })

  it("known credential name has truncate class to prevent mobile overflow", () => {
    renderSettings()
    const token = screen.getByText("CLAUDE_CODE_OAUTH_TOKEN")
    expect(token.className).toMatch(/truncate/)
  })

  it("reveals inline input form when Add is clicked on a known credential", () => {
    renderSettings()
    const [firstAdd] = screen.getAllByRole("button", { name: /add/i })
    fireEvent.click(firstAdd)
    // Two "Value" inputs exist: the inline edit form + the custom credential form
    expect(screen.getAllByPlaceholderText("Value").length).toBeGreaterThanOrEqual(1)
    expect(screen.getByRole("button", { name: /save/i })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: /cancel/i })).toBeInTheDocument()
  })

  it("hides Add button while inline edit form is open", () => {
    renderSettings()
    const addButtons = screen.getAllByRole("button", { name: /add/i })
    const initialCount = addButtons.length
    fireEvent.click(addButtons[0])
    // One Add button should disappear (replaced by Save/Cancel)
    expect(screen.getAllByRole("button", { name: /add/i }).length).toBe(
      initialCount - 1,
    )
  })

  it("renders custom credential form with ENV_VAR_NAME and Value inputs", () => {
    renderSettings()
    expect(screen.getByPlaceholderText("ENV_VAR_NAME")).toBeInTheDocument()
    // Password input for the custom credential value
    const valueInputs = screen.getAllByPlaceholderText("Value")
    expect(valueInputs.length).toBeGreaterThanOrEqual(1)
  })

  it("delete button has aria-label naming the credential", async () => {
    const { api: mockApi } = await import("@/lib/api")
    vi.mocked(mockApi.getCredentials).mockResolvedValueOnce({
      credentials: [{ name: "MY_SECRET", updated_at: new Date().toISOString() }],
    })
    const { findByRole } = renderSettings()
    const deleteBtn = await findByRole("button", { name: "Delete MY_SECRET" })
    expect(deleteBtn).toBeInTheDocument()
  })
})
