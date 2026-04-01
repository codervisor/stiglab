use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use stiglab_core::{AgentMessage, Task};

use super::process::SessionProcess;

pub struct SessionManager {
    max_sessions: u32,
    agent_command: String,
    outbound_tx: mpsc::UnboundedSender<AgentMessage>,
    sessions: Arc<RwLock<HashMap<String, SessionProcess>>>,
    active_count: Arc<AtomicU32>,
}

impl SessionManager {
    pub fn new(
        max_sessions: u32,
        agent_command: String,
        outbound_tx: mpsc::UnboundedSender<AgentMessage>,
    ) -> Self {
        SessionManager {
            max_sessions,
            agent_command,
            outbound_tx,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_count: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn active_count_handle(&self) -> Arc<AtomicU32> {
        self.active_count.clone()
    }

    pub async fn spawn_session(&mut self, task: Task, session_id: String) {
        let count = self.active_count.load(Ordering::Relaxed);
        if count >= self.max_sessions {
            tracing::warn!(
                "at capacity ({}/{}), rejecting task {}",
                count,
                self.max_sessions,
                task.id
            );
            let _ = self.outbound_tx.send(AgentMessage::SessionFailed {
                session_id,
                error: "node at capacity".to_string(),
            });
            return;
        }

        match SessionProcess::spawn(
            &task,
            &session_id,
            &self.agent_command,
            self.outbound_tx.clone(),
        )
        .await
        {
            Ok(process) => {
                self.active_count.fetch_add(1, Ordering::Relaxed);
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), process);
                drop(sessions);

                // Spawn a task to wait for completion
                let sessions = self.sessions.clone();
                let active_count = self.active_count.clone();
                let outbound_tx = self.outbound_tx.clone();
                let sid = session_id.clone();

                tokio::spawn(async move {
                    let success = {
                        let mut sessions = sessions.write().await;
                        if let Some(ref mut proc) = sessions.get_mut(&sid) {
                            proc.wait().await.unwrap_or(false)
                        } else {
                            false
                        }
                    };

                    // Clean up
                    {
                        let mut sessions = sessions.write().await;
                        sessions.remove(&sid);
                    }
                    active_count.fetch_sub(1, Ordering::Relaxed);

                    if success {
                        let _ = outbound_tx.send(AgentMessage::SessionCompleted {
                            session_id: sid,
                            output: String::new(),
                        });
                    } else {
                        let _ = outbound_tx.send(AgentMessage::SessionFailed {
                            session_id: sid,
                            error: "process exited with non-zero status".to_string(),
                        });
                    }
                });
            }
            Err(e) => {
                tracing::error!("failed to spawn session: {e}");
                let _ = self.outbound_tx.send(AgentMessage::SessionFailed {
                    session_id,
                    error: e.to_string(),
                });
            }
        }
    }

    pub async fn cancel_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(ref mut proc) = sessions.get_mut(session_id) {
            proc.kill();
        }
    }

    pub async fn send_input(&self, session_id: &str, input: &str) {
        let sessions = self.sessions.read().await;
        if let Some(proc) = sessions.get(session_id) {
            if let Err(e) = proc.send_input(input) {
                tracing::error!("failed to send input to session {session_id}: {e}");
            }
        }
    }
}
