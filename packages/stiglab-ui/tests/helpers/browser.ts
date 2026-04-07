import { execSync, type ExecSyncOptions } from "node:child_process";

const EXEC_OPTIONS: ExecSyncOptions = {
  encoding: "utf-8",
  timeout: 30_000,
  stdio: ["pipe", "pipe", "pipe"],
};

/**
 * Wrapper around the agent-browser CLI for deterministic e2e tests.
 * Each instance manages a single browser session against a target URL.
 */
export class Browser {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  /** Run an agent-browser CLI command and return stdout. */
  private run(args: string): string {
    const cmd = `npx agent-browser ${args}`;
    try {
      return execSync(cmd, EXEC_OPTIONS) as string;
    } catch (err) {
      const e = err as { stderr?: string; stdout?: string };
      throw new Error(
        `agent-browser command failed: ${cmd}\nstderr: ${e.stderr ?? ""}\nstdout: ${e.stdout ?? ""}`,
      );
    }
  }

  /** Navigate to a path relative to the base URL. */
  open(path = "/"): string {
    return this.run(`open "${this.baseUrl}${path}"`);
  }

  /** Get an accessibility snapshot of the current page. */
  snapshot(): string {
    return this.run("snapshot");
  }

  /** Get an interactive accessibility snapshot with element refs. */
  interactiveSnapshot(): string {
    return this.run("snapshot -i");
  }

  /** Click on an element by its ref (e.g., "@e1"). */
  click(ref: string): string {
    return this.run(`click "${ref}"`);
  }

  /** Fill a text field by its ref with a value. */
  fill(ref: string, value: string): string {
    return this.run(`fill "${ref}" "${value}"`);
  }

  /** Take a screenshot and return the file path. */
  screenshot(outputPath?: string): string {
    const args = outputPath ? `screenshot --output "${outputPath}"` : "screenshot";
    return this.run(args);
  }

  /** Wait for text to appear on the page. */
  waitForText(text: string, timeoutMs = 10_000): string {
    return this.run(`wait --text "${text}" --timeout ${timeoutMs}`);
  }

  /** Wait for a specific URL pattern. */
  waitForUrl(pattern: string, timeoutMs = 10_000): string {
    return this.run(`wait --url "${pattern}" --timeout ${timeoutMs}`);
  }

  /** Get the current page title. */
  title(): string {
    return this.run("get title").trim();
  }

  /** Get the current page URL. */
  url(): string {
    return this.run("get url").trim();
  }

  /** Get text content of the page or a specific element. */
  getText(ref?: string): string {
    return this.run(ref ? `get text "${ref}"` : "get text");
  }

  /** Execute JavaScript in the browser context. */
  evaluate(js: string): string {
    return this.run(`evaluate "${js.replace(/"/g, '\\"')}"`);
  }

  /** Close the browser. */
  close(): string {
    return this.run("close");
  }
}

/**
 * Create a Browser instance pointed at the dev server.
 * Defaults to STIGLAB_TEST_URL or http://localhost:5173.
 */
export function createBrowser(baseUrl?: string): Browser {
  const url = baseUrl ?? process.env.STIGLAB_TEST_URL ?? "http://localhost:5173";
  return new Browser(url);
}
