use anyhow::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use stiglab_core::{AgentMessage, SessionState, Task};

pub struct SessionProcess {
    child: Child,
    stdin_tx: Option<mpsc::UnboundedSender<String>>,
}

impl SessionProcess {
    pub async fn spawn(
        task: &Task,
        session_id: &str,
        agent_command: &str,
        outbound_tx: mpsc::UnboundedSender<AgentMessage>,
    ) -> Result<Self> {
        let mut cmd = Command::new(agent_command);
        cmd.arg("--print").arg(&task.prompt);

        if let Some(ref dir) = task.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped());

        let mut child = cmd.spawn()?;

        let session_id = session_id.to_string();

        // Handle stdin
        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
        if let Some(mut stdin) = child.stdin.take() {
            tokio::spawn(async move {
                while let Some(input) = stdin_rx.recv().await {
                    if stdin.write_all(input.as_bytes()).await.is_err() {
                        break;
                    }
                    if stdin.write_all(b"\n").await.is_err() {
                        break;
                    }
                    let _ = stdin.flush().await;
                }
            });
        }

        // Handle stdout streaming
        if let Some(stdout) = child.stdout.take() {
            let tx = outbound_tx.clone();
            let sid = session_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Detect waiting for input patterns
                    if line.contains("waiting for input")
                        || line.contains("? ")
                        || line.contains("[Y/n]")
                    {
                        let _ = tx.send(AgentMessage::SessionStateChanged {
                            session_id: sid.clone(),
                            state: SessionState::WaitingInput,
                        });
                    }

                    let _ = tx.send(AgentMessage::SessionOutput {
                        session_id: sid.clone(),
                        chunk: format!("{line}\n"),
                    });
                }
            });
        }

        // Handle stderr
        if let Some(stderr) = child.stderr.take() {
            let tx = outbound_tx.clone();
            let sid = session_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send(AgentMessage::SessionOutput {
                        session_id: sid.clone(),
                        chunk: format!("[stderr] {line}\n"),
                    });
                }
            });
        }

        // Notify running state
        let _ = outbound_tx.send(AgentMessage::SessionStateChanged {
            session_id: session_id.clone(),
            state: SessionState::Running,
        });

        Ok(SessionProcess {
            child,
            stdin_tx: Some(stdin_tx),
        })
    }

    pub fn send_input(&self, input: &str) -> Result<()> {
        if let Some(ref tx) = self.stdin_tx {
            tx.send(input.to_string())?;
        }
        Ok(())
    }

    pub async fn wait(&mut self) -> Result<bool> {
        let status = self.child.wait().await?;
        Ok(status.success())
    }

    pub fn kill(&mut self) {
        let _ = self.child.start_kill();
    }
}
